# Changes task runner. `just` lists recipes; `just ci` mirrors CI locally
# (gitleaks is skipped with a warning if the binary isn't installed).

default:
    @just --list

# One-time per clone: install git hooks (pre-push runs the fast gates)
setup:
    git config core.hooksPath .githooks
    @echo "hooks installed — pre-push now runs fmt + clippy + test"

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

clippy:
    cargo clippy --locked --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --locked --workspace

deny:
    cargo deny check

# Broken links + merge-conflict markers — same script CI's docs job runs
links:
    python3 .github/scripts/check_doc_links.py

# Secrets scan — mirrors CI's gitleaks job when the binary is installed
gitleaks:
    @if command -v gitleaks >/dev/null 2>&1; then \
        gitleaks detect --redact --exit-code=1; \
    else \
        echo "warning: gitleaks not installed (brew install gitleaks) — secrets scan will only run in CI"; \
    fi

# Fast gate mirror — the pre-push hook runs this
pre-push: fmt-check clippy test

# Full CI mirror — run before opening a PR
ci: fmt-check clippy test deny links gitleaks
    @echo "all local gates green"

# ── iOS (names mirror intrada) ──────────────────────────────────────────

# Open the app in Xcode (regenerates bindings first if the core changed).
ios: _ios-sync
    cd ios && xcodegen generate
    xed ios/Changes.xcodeproj

# Build + launch on a simulator (regenerates bindings if the core changed).
ios-run: _ios-sync
    cd ios && xcodegen generate
    bash scripts/ios-run-sim.sh

# Signed Release .ipa -> TestFlight (needs env from docs/RELEASING.md).
testflight: ios-typegen (ios-package "release")
    cd ios && xcodegen generate
    rm -f ios/generated/.gen-stamp
    bundle exec fastlane ios beta

# Stream app logs from the booted sim, filtered to our subsystem.
ios-logs:
    xcrun simctl spawn booted log stream --predicate 'subsystem == "com.changes.app"'

# Force a full regenerate of both Swift packages + refresh the change-stamp.
ios-gen: ios-typegen (ios-package "debug")
    @mkdir -p ios/generated
    @just _ios-src-hash > ios/generated/.gen-stamp
    @echo "✓ bindings regenerated"

# Build the app for the simulator without launching (CI's build step).
ios-build: _ios-sync
    cd ios && xcodegen generate
    cd ios && xcodebuild build -project Changes.xcodeproj -scheme Changes \
        -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' \
        -derivedDataPath build/dd -clonedSourcePackagesDirPath build/spm -quiet \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO

# Build + run the ChangesTests suite on a dedicated simulator (created on
# first use, reused after; never hijacks a booted sim — CLAUDE.md rule).
ios-test: _ios-sync
    #!/usr/bin/env bash
    set -euo pipefail
    cd ios
    xcodegen generate
    UDID=$(xcrun simctl list devices --json | python3 -c "
    import json, sys
    devices = json.load(sys.stdin)['devices']
    print(next((d['udid'] for ds in devices.values() for d in ds
                if d['name'] == 'changes-test-sim'), ''))
    ")
    # Runtime pinned to CI's (Xcode 26.5) so snapshot renders agree.
    [ -n "$UDID" ] || UDID=$(xcrun simctl create changes-test-sim "iPhone 16" "iOS26.5")
    xcodebuild test -project Changes.xcodeproj -scheme Changes -sdk iphonesimulator \
        -destination "id=$UDID" -derivedDataPath build/dd \
        -clonedSourcePackagesDirPath build/spm -quiet \
        COMPILER_INDEX_STORE_ENABLE=NO CODE_SIGNING_ALLOWED=NO

# Facet typegen → ios/generated/SharedTypes.
ios-typegen:
    # Pre-clean so a renamed/removed type can't leave an orphan Swift file.
    rm -rf ios/generated/SharedTypes
    cargo run -p changes-ffi --bin codegen --features codegen -- --output-dir ios/generated

# cargo-swift → ios/generated/ChangesCoreFFI (CoreFFI + RustFramework.xcframework).
ios-package profile="debug":
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/changes-ffi
    if [ "{{profile}}" = "release" ]; then rel="--release"; else rel=""; fi
    cargo swift package --name ChangesCoreFFI --platforms ios --lib-type static --features uniffi $rel --accept-all
    rm -rf ../../ios/generated/ChangesCoreFFI
    mkdir -p ../../ios/generated
    mv ChangesCoreFFI ../../ios/generated/ChangesCoreFFI
    # cargo-swift 0.9.0 nests modulemap+header one level too deep; the
    # xcframework Info.plist declares HeadersPath=Headers, so canImport
    # fails. Move them up (same fix as intrada).
    xcf=../../ios/generated/ChangesCoreFFI/RustFramework.xcframework
    moved=0
    for slice in "$xcf"/*/; do
        hd="$slice/headers"
        if [ -d "$hd/RustFramework" ]; then
            mv "$hd/RustFramework/"* "$hd/"; rmdir "$hd/RustFramework"; moved=1
        fi
    done
    [ "$moved" = 1 ] || echo "⚠️  cargo-swift header layout changed — verify canImport(changes_ffiFFI)"
    echo "✓ ios/generated/ChangesCoreFFI"

_ios-src-hash:
    @find crates/changes-core/src crates/changes-ffi/src \
         crates/changes-core/Cargo.toml crates/changes-ffi/Cargo.toml \
         -type f -exec shasum {} \; | shasum | cut -d' ' -f1

_ios-sync:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=ios/generated/.gen-stamp
    current=$(just _ios-src-hash)
    if [ ! -d ios/generated/ChangesCoreFFI ] || [ ! -d ios/generated/SharedTypes ] \
       || [ "$(cat "$stamp" 2>/dev/null)" != "$current" ]; then
        echo "↻ core changed (or no bindings) — regenerating…"
        just ios-gen
    else
        echo "✓ bindings up to date"
    fi
