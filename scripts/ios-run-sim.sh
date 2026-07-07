#!/usr/bin/env bash
# Build the iOS app, launch it on a simulator, and screenshot.
# Assumes the Xcode project + generated packages already exist (the `ios-run`
# just recipe runs `_ios-sync` + xcodegen first).
#
# Usage: scripts/ios-run-sim.sh [screenshot-path]
set -euo pipefail

cd "$(dirname "$0")/../ios"

APP_ID="com.changes.app"
DD="build/dd"
SHOT="${1:-/tmp/changes-ios.png}"
mkdir -p "$(dirname "$SHOT")"

# Pick a simulator deterministically: prefer SIM_DEVICE (default iPhone 16)
# on the highest installed iOS; fall back to the newest available iPhone.
# Stable udid tie-break keeps repeated runs identical (intrada #854).
SIM_DEVICE="${SIM_DEVICE:-iPhone 16}"
UDID=$(xcrun simctl list devices available --json | python3 -c "
import json, re, sys
want = sys.argv[1]
runtimes = json.load(sys.stdin)['devices']
def ver(key):
    m = re.search(r'iOS-(\d+)-(\d+)', key)
    return (int(m.group(1)), int(m.group(2))) if m else (-1, -1)
cands = [
    (dev['name'] == want, ver(key), dev['udid'])
    for key, devices in runtimes.items() if 'iOS' in key
    for dev in devices if 'iPhone' in dev['name']
]
print(max(cands)[2] if cands else '')
" "$SIM_DEVICE")
if [ -z "$UDID" ]; then
    echo "✗ No iPhone simulator available (Xcode → Settings → Platforms → iOS)" >&2
    exit 1
fi

xcrun simctl boot "$UDID" 2>/dev/null || true

# REUSE_BUILD=1 reuses an existing build/dd .app (e.g. CI, right after a
# build step) — avoids a second xcodebuild.
if [ -z "${REUSE_BUILD:-}" ]; then
    echo "building…"
    xcodebuild -project Changes.xcodeproj -scheme Changes -sdk iphonesimulator \
        -destination "id=$UDID" -derivedDataPath "$DD" -configuration Debug \
        build CODE_SIGNING_ALLOWED=NO >/tmp/changes-ios-build.log 2>&1 || {
        echo "✗ build failed:" >&2
        grep -E "error:" /tmp/changes-ios-build.log | tail -20 >&2
        exit 1
    }
fi

APP=$(find "$DD/Build/Products" -name "Changes.app" -type d | head -1)
[ -n "$APP" ] || { echo "✗ no Changes.app in $DD (build first)" >&2; exit 1; }
xcrun simctl install "$UDID" "$APP"
xcrun simctl launch "$UDID" "$APP_ID" >/dev/null
sleep 3
xcrun simctl io "$UDID" screenshot "$SHOT" >/dev/null
echo "✓ launched on $UDID — screenshot: $SHOT"
