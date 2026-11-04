#!/usr/bin/env bash
# Regenerate every image in docs/assets from the firmware's own layout and
# rendering code (sign-core), so the guide shows exactly what the sign draws.
# Needs Rust (stable) and Python 3 with Pillow. macOS: uses Helvetica Bold
# from /System/Library/Fonts for the bezel labels.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/tools/out"
assets="$root/docs/assets"
fixtures="$root/sign-core/tests/fixtures"
rm -rf "$out"
mkdir -p "$out/bands" "$out/boot" "$out/stations" "$assets"

cd "$root/sign-core"
run() { cargo +stable run -q --release --example "$@"; }

# 1. Every station from the live snapshot, plus Montgomery at 7:17 PM with
#    four northbound destinations, plus Lafayette through a whole night.
cp "$fixtures"/all/*.json "$out/stations/"
sed 's/"minutes":"Leaving"/"minutes":"2"/' "$fixtures/mont_live.json" > "$out/stations/MONT-RUSH.json"
cp "$fixtures/lafy_evening.json"      "$out/stations/LAFY-EVENING.json"
cp "$fixtures/lafy_leaving.json"      "$out/stations/LAFY-ARRIVING.json"
cp "$fixtures/lafy_late_96min.json"   "$out/stations/LAFY-LATE.json"
cp "$fixtures/lafy_south_done.json"   "$out/stations/LAFY-SOUTH-DONE.json"
cp "$fixtures/lafy_service_over.json" "$out/stations/LAFY-NIGHT.json"
run mockups "$out/stations" "$out/bands"
python3 -I "$root/tools/compose_mockups.py" "$out/bands" "$fixtures/stations.json" "$assets"

# 2. The boot sequence on both signs.
run boot_pair "$out/boot" > "$out/boot/index.txt"
python3 -I "$root/tools/compose_boot.py" "$out/boot" "$assets/boot.gif"
mv "$assets/boot_final.png" "$assets/boot-final.png"

# 3. Contact sheets of every station name in both fonts.
run all_stations "$assets"
# The full live contact sheet is a working file, not a guide image.
mv "$assets/live_all_stations.png" "$out/"
echo "docs/assets regenerated"
