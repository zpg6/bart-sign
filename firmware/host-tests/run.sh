#!/usr/bin/env bash
# Run the host unit tests. firmware/.cargo/config.toml targets the ESP32
# and adds firmware link flags, so pick stable Rust and the host target
# explicitly, and clear RUSTFLAGS.
set -euo pipefail
cd "$(dirname "$0")"
host=$(rustc +stable -vV | sed -n 's/^host: //p')
RUSTFLAGS="" exec cargo +stable test --target "$host" "$@"
