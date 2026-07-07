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
    cargo clippy --locked --workspace --all-targets -- -D warnings

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

# ── iOS (lands with M0-iOS; names mirror intrada) ──────────────────────

ios:
    @echo "M0-iOS not scaffolded yet — see docs/specs/mvp-plan.md (M0)" && exit 1

ios-run:
    @echo "M0-iOS not scaffolded yet — see docs/specs/mvp-plan.md (M0)" && exit 1

ios-gen:
    @echo "M0-iOS not scaffolded yet — see docs/specs/mvp-plan.md (M0)" && exit 1

ios-logs:
    @echo "M0-iOS not scaffolded yet — see docs/specs/mvp-plan.md (M0)" && exit 1
