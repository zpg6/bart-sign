#!/usr/bin/env bash
# Build, flash and open the serial monitor. Wi-Fi settings are not built in:
# a sign with no saved settings starts the BART-SIGN-XXXX setup network.
set -euo pipefail
cd "$(dirname "$0")"
# Xtensa linker/clang paths written by `espup install`.
source "${ESP_EXPORT:-$HOME/export-esp.sh}"
# Personal builds: turn on the private ba-mark feature when its file is here.
if [ -f ../sign-core/private/ba_mark.rs ]; then set -- --features ba-mark "$@"; fi
exec cargo run --release "$@"
