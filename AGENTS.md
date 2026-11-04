# AGENTS.md

Instructions for AI coding agents working on this repo. Humans should read the [build guide](docs/guide.md) instead.

Every command below was run on macOS (Apple silicon). Linux should work the same way. Windows is untested.

## Ground rules

- Paths are relative to the repo root unless a step says `cd`.
- `firmware/` builds only with the Xtensa toolchain named `esp-1.97` (pinned in `firmware/rust-toolchain.toml`). `sign-core/` and `firmware/host-tests/` build with ordinary stable Rust.
- Wi-Fi credentials are never compiled in. The sign gets them from its setup page and stores them in the `config` flash partition. Don't add credentials to code or commit `wifi.env`.
- Don't flash or reset a board unless the user asks you to. A sign may be running in someone's home.
- The images in `docs/assets/` are generated. Don't edit them by hand. Run `tools/render_docs.sh`.
- `docs/index.html` is the GitHub Pages site, generated from `docs/guide.md`. Edit the guide, then run `python3 tools/build_site.py`. CI fails if the page is stale.

## 1. Install the toolchain

Prerequisite: Rust via rustup (https://rustup.rs) with the stable toolchain.

```sh
rustup toolchain install stable
cargo install espup --locked
espup install --name esp-1.97 --toolchain-version 1.97.0.0 --targets esp32
cargo install espflash --locked
```

On Linux, espup's README asks for build packages first (Debian or Ubuntu: `sudo apt-get install -y gcc build-essential curl pkg-config`). espflash 4.7 doesn't need `libudev-dev`. Untested here.

`espup install` writes an export file with the Xtensa linker and clang paths. The default is `~/export-esp.sh`. If you passed `--export-file <path>`, use that path wherever this file says `~/export-esp.sh`.

Check:

```sh
rustc +esp-1.97 --version   # must end in (1.97.0.0)
espflash --version          # must be 4.7.0 or newer
```

### Toolchain pitfalls

- **Use the stable esp-rs release 1.97.0.0.** The 1.98 and 1.99 pre-releases miscompile ESP32 startup. The firmware builds and flashes fine, then the chip hangs before `main` and the RTC watchdog reboots it forever. The serial log never shows `bart-sign: boot`. If you see that, check `rustc +esp-1.97 --version` first.
- **The toolchain name must be `esp-1.97`.** `firmware/rust-toolchain.toml` pins `channel = "esp-1.97"`. A plain `espup install` names it `esp`, and cargo in `firmware/` then fails with "custom toolchain 'esp-1.97' ... is not installed".
- **espflash must be 4.7 or newer.** Older versions such as 4.1 flash an older second-stage bootloader. Upgrade with `cargo install espflash --locked --force`.
- **Source the export file in the same shell as the build.** Without it, `build.rs` stops with "Xtensa linker `xtensa-esp32-elf-gcc` was not found in PATH".

## 2. Build

```sh
cd firmware
source "${ESP_EXPORT:-$HOME/export-esp.sh}"
cargo build --release
```

Output: `firmware/target/xtensa-esp32-none-elf/release/bart-sign` (an ELF, about 10 MB with debug info). A clean build takes about a minute after the crates are downloaded.

Set `ESP_EXPORT` to point at a non-default export file. `firmware/flash.sh` reads the same variable.

The boot screen has an optional private `ba-mark` feature for personal builds. It needs `sign-core/private/ba_mark.rs`, which is git-ignored and not in the repo. If that file exists, add `--features ba-mark` to every firmware build. `flash.sh` adds it on its own. CI builds without it.

## 3. Find the serial port

The ELEGOO board uses a CP2102 USB-serial chip. With the board plugged in:

```sh
espflash list-ports
```

Typical names are `/dev/cu.usbserial-XXXX` on macOS and `/dev/ttyUSB0` on Linux. If nothing is listed, run `espflash list-ports --list-all-ports` and compare the output with the board unplugged and plugged in.

If no new port appears, the cable is most likely charge-only. Ask the user to swap it for a data cable. On Linux, the user may also need to be in the `dialout` group.

Pass the port to every espflash call below with `--port`, or export it once:

```sh
export ESPFLASH_PORT=/dev/cu.usbserial-XXXX
```

## 4. Flash

Interactive, as a human would (builds, flashes, then opens the serial monitor):

```sh
cd firmware
ESPFLASH_PORT=/dev/cu.usbserial-XXXX ./flash.sh
```

`flash.sh` sources the export file and runs `cargo run --release`, adding `--features ba-mark` when `sign-core/private/ba_mark.rs` exists. The cargo runner in `firmware/.cargo/config.toml` is:

```
espflash flash --monitor --chip esp32 --flash-size 4mb --partition-table partitions.csv
```

Non-interactive, for agents. Flash first, then monitor separately:

```sh
cd firmware
source "${ESP_EXPORT:-$HOME/export-esp.sh}"
cargo build --release   # plus --features ba-mark if sign-core/private/ba_mark.rs exists
espflash flash --non-interactive --chip esp32 --flash-size 4mb \
  --partition-table partitions.csv --port "$ESPFLASH_PORT" \
  target/xtensa-esp32-none-elf/release/bart-sign
```

Always pass `--partition-table partitions.csv`. The firmware stores its settings in the `config` partition that this table defines. Flashing the app leaves `config` alone, so a reflash keeps the user's Wi-Fi settings.

### If it hangs at "Connecting..."

The board didn't enter its bootloader on its own. Ask the user to:

1. Press and hold the **BOOT** button.
2. Press and release the **EN** button.
3. Release **BOOT** once espflash starts writing.

Then run the flash command again if it timed out.

## 5. Verify with the serial log

```sh
espflash monitor --non-interactive --chip esp32 --port "$ESPFLASH_PORT" \
  --elf target/xtensa-esp32-none-elf/release/bart-sign
```

The monitor runs until stopped. Run it in the background, or under a time limit (`timeout 90` on Linux, `gtimeout 90` from coreutils on macOS), and capture the output to a file. 90 seconds covers boot, Wi-Fi and the first fetch.

Log lines carry a level prefix such as `INFO -`. Match on the message text.

Every boot:

| Line | Meaning |
|---|---|
| `bart-sign: boot` | The app is running. If this never appears and the board keeps resetting, see the toolchain pitfalls above. |
| `display init done`, `display up` | The ST7796S answered and the backlight is on. |
| `config partition at 0x310000, 0x1000 bytes` | Settings storage found. |

No saved settings (setup mode):

| Line | Meaning |
|---|---|
| `no saved settings: setup mode` | First boot, or the settings were reset. |
| `Wi-Fi scan: N networks, M offered` | Networks the setup page will list. |
| `AP "BART-SIGN-XXXX" up, open, portal at http://192.168.4.1/` | The setup network is up. |
| `sign message: ["JOIN WI-FI", "BART-SIGN-XXXX", "OPEN 192.168.4.1"]` | What the sign shows. |
| `AP: client ... joined` | A phone joined. |
| `saved: ssid "...", station LAFY, line Yellow` then `settings saved, restarting in 2 s` | The user saved the form. The board reboots into normal mode. |

Saved settings (normal mode), in order:

| Line | Meaning |
|---|---|
| `settings: ssid "...", station LAFY, line Yellow` | Settings loaded. |
| `connecting to "..."` | Joining the saved network. |
| `Wi-Fi connected: ...` | Joined. |
| `got IP 192.168.x.y/24` | DHCP done. |
| `api.bart.gov -> <ip>` and `TLS up: ...` | DNS and the TLS handshake worked. |
| `ETD LAFY 10:20:31 PM PST: 2 south, 2 north` | **Success.** A fetch parsed: station, BART's clock, and destination groups per direction. |
| `boot screen done after N ms` | The intro finished and the sign switched to departures (or to a status message). |
| `next fetch in 61 s` | The fetch loop is running. About 60 s normally, about 300 s once service is over for the night, and up to 300 s after errors. |

Healthy signs: `bart-sign: boot`, then `ETD <station> ...`, then `boot screen done after`, then a new `ETD` line about once a minute.

Problems:

| Line | Meaning |
|---|---|
| One `Wi-Fi connect failed: ...` naming `AuthenticationFailed`, right after a reset | Normal on some routers, which still hold the board's old session. The firmware retries after 0.5 s and the retry succeeds. |
| `Wi-Fi connect failed: ...` repeating, and `"<ssid>" not in scan results` | Wrong network name, or a 5 GHz-only network. The sign only sees 2.4 GHz. |
| `sign message: ["CAN'T JOIN", ...]` | Three failed joins in a row. The firmware keeps retrying. The user can hold BOOT 5 s to reset. |
| `fetch_etd failed: ...` | Network or TLS error. The firmware backs off to 120 s, 240 s, then 300 s. |
| `ETD parse failed: ...` | BART's response didn't parse. Save the body for a test fixture if you can reproduce it. |
| `BOOT held 5 s: erasing settings` | Someone held BOOT. The board restarts in setup mode. |
| `====================== PANIC ======================` | The firmware panicked. The screen freezes on its last frame and the board does not restart on its own (esp-hal turns the watchdog off at init). Power-cycle it, and keep the log. |

To wipe the settings without the button (not tried on hardware here):

```sh
espflash erase-parts --chip esp32 --port "$ESPFLASH_PORT" --partition-table partitions.csv config
```

## 6. Host tests

No board needed. Both use stable Rust.

```sh
cd sign-core
cargo test
cargo clippy --all-targets -- -D warnings
```

Expect 40 tests across 4 groups: `src/lib.rs` 3, `tests/etd.rs` 6, `tests/logo.rs` 16, `tests/stations.rs` 15.

```sh
firmware/host-tests/run.sh
```

Expect 15 tests. `run.sh` compiles `firmware/src/dns.rs`, `settings.rs` and `web.rs` for the host with `cargo +stable`, so the stable toolchain must be installed. Don't run plain `cargo test` in `firmware/`: that targets the ESP32.

CI (`.github/workflows/ci.yml`) runs both, plus a firmware build with `esp-rs/xtensa-toolchain` at 1.97.0.0.

## 7. Regenerate the docs images

```sh
tools/render_docs.sh
```

Needs stable Rust, Python 3 with Pillow (`pip3 install pillow`), and macOS, because the bezel labels use `/System/Library/Fonts/Helvetica.ttc`. It renders with `sign-core`'s examples, writes everything in `docs/assets/`, and keeps working files in `tools/out/` (git-ignored). It ends with `docs/assets regenerated`.

The output is deterministic. With no code changes, `git status` stays clean after a run. Inputs are the fixtures in `sign-core/tests/fixtures/`:

- `all/*.json`: live departures for 49 stations, 10:20 PM, Nov 2 2026. Oakland Airport has no BART departures in the feed, so it has no snapshot.
- `lafy_*.json`, `mont_live.json`: the Lafayette and Montgomery screens in the guide.
- `night/*.json`: Lafayette, and the whole system in one response (`orig=ALL`), every 10 minutes from 10:27 PM to 1:57 AM.
- `stations.json`: the station list (50 stations).

## 8. Rebuild the website

The site at https://zpg6.github.io/bart-sign/ is `docs/index.html`, served by GitHub Pages from `docs/` on `main`. It is generated from `docs/guide.md`:

```sh
python3 tools/build_site.py           # rewrite docs/index.html
python3 tools/build_site.py --check   # exit 1 if it is stale (CI runs this)
```

Standard-library Python 3 only. The page style is `docs/assets/site.css`. `docs/.nojekyll` tells GitHub Pages to serve the files as they are. To preview, serve `docs/` and open http://localhost:8000:

```sh
python3 -m http.server -d docs 8000
```

## Where things live

| Want to change | File |
|---|---|
| Pins | `firmware/src/bin/main.rs` (display, backlight, BOOT button) |
| Fetch interval, backoff | `firmware/src/bin/main.rs` (`FETCH_PERIOD_SECS`, `NIGHT_FETCH_SECS`, `FETCH_BACKOFF_MAX_SECS`) |
| BART API key | `firmware/src/net.rs` (`PATH_END`). It uses BART's public key. |
| Station list on the setup page | `firmware/src/settings.rs` (`STATIONS`) |
| Setup page HTML | `firmware/src/web.rs` |
| Screen layout, name abbreviations | `sign-core/src/layout.rs` |
| Which platform goes on which sign | `sign-core/src/board.rs`, plus `TOP` in `firmware/src/sign.rs` |
| Fonts | `sign-core/src/glyphs.rs` |
| Boot animation | `sign-core/src/logo.rs` |
| Housing colors, backlight levels, rotation time | `firmware/src/sign.rs` |
| Website template and style | `tools/build_site.py`, `docs/assets/site.css` |
