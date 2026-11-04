# BART sign: power engineering

Scope: the replica BART arrival sign as it exists today (ESP-WROOM-32 dev board, 3.5" ST7796S TFT, HTTPS fetch every 60 s), a USB-C powered gift version, and what a battery version would take. Numbers in brackets like [1] point to the sources at the end. Where I estimated instead of citing, the text says so.

## Short answer

- On USB the sign draws about **196 mA at 5 V (1.0 W)** today. Two firmware changes take that to about **77 mA (0.39 W)**: Wi-Fi modem sleep, and the backlight at 30% PWM.
- On a battery with the display always on, no cell of the size you'd put in this gadget lasts a month. The best always-on case is about 3 days on an 18650. The ST7796S controller alone draws 12 mA whenever the panel shows anything [5]. That is 8.6 Ah a month before the backlight, the radio or the CPU draw anything.
- "A month or two" works in **glance mode**: deep sleep, a button press wakes it, it fetches, it shows trains for 30 to 60 s, it sleeps. A 2000 mAh cell gives about 2 months at 20 glances a day and about 4 months at 10 a day. That only holds on a board without the dev board's AMS1117, CP2102 and power LED. Those three hold the sleep floor at 4 to 12 mA, and on the stock board glance mode lasts 1 to 4 weeks.
- E-paper also reaches months, but it isn't this product. No glow, a 15 s flashing refresh on the red/black/white panels, and countdowns that are 3 to 5 minutes stale.
- Recommendation: ship **USB-C always-on as the default product**, with the two firmware fixes and a night mode. If you want a battery option, make it a small carrier board: BQ24074 charger with power path, TPS63802 buck-boost, a load switch for the display, a protected 2000 mAh LiPo. It runs glance mode on battery and switches to the live, always-on sign when it's plugged in.

### Runtime table

Custom battery board (no AMS1117, CP2102 or power LED), TPS63802 at 90%, 90% of rated capacity usable, 2.5%/month self-discharge. Section 2 has the derivations.

| Mode | 1000 mAh | 2000 mAh | 3000 mAh | 18650, 3500 mAh | Month or two? |
|---|---|---|---|---|---|
| (a) Always-on as today | 4.8 h | 9.6 h | 14 h | 17 h | No |
| (a0) Today's dev board on a power-bank chip | 3.0 h | 5.9 h | 8.9 h | 10 h | No |
| (b) Always-on, Wi-Fi modem sleep, backlight 30% | 13 h | 26 h | 39 h | 45 h | No. It's a day out, not a month |
| (b') As (b) plus automatic light sleep (experimental) | 23 h | 47 h | 70 h | 82 h | No |
| (c) Glance, 5 a day | 108 d | 197 d | 270 d | 303 d | Yes, with any cell |
| (c) Glance, 10 a day | 61 d | 115 d | 163 d | 186 d | Yes |
| (c) Glance, 20 a day | 32 d | 63 d | 91 d | 105 d | 1 month on 1000 mAh, 2 months on 2000 mAh |
| (c) Glance, 10 a day, stock dev board | 4-9 d | 9-17 d | 13-25 d | 15-29 d | No |
| (d) Scheduled, 4 h each weekday, (b) settings | 5 d | 9 d | 13 d | 16 d | No |
| (d) Scheduled, 2 h each weekday, light sleep, backlight 20% | 19 d | 37 d | 54 d | 62 d | Barely, with 3000 mAh or more |
| (e) Tri-color e-paper, every 5 min | 23 d | 45 d | 66 d | 76 d | Yes with 2000 mAh or more, but see 2.7 |
| (e) Black/white e-paper, every 5 min | 32 d | 62 d | 91 d | 105 d | Yes |

The "standard" glance is 45 s on screen at full backlight with a fast Wi-Fi reconnect: 1.31 mAh per glance. A lean glance (30 s, 50% backlight, light sleep while the screen is up) is 0.47 mAh, which about doubles the 10 and 20 a day figures.

### The three biggest power wins

1. **Wi-Fi modem sleep: about 80 mA saved, one line of code.** esp-radio's `WifiController` starts with `PowerSaveMode::None` (esp-radio 1.0.0-beta.1, `src/wifi/mod.rs`, set in `WifiController::new`), unlike ESP-IDF, whose default is modem sleep. With `None` the receiver listens nonstop at 95 to 100 mA [1]. With `PowerSaveMode::Minimum`, ESP-IDF measures an ESP32 average of 29 to 31 mA [2].
2. **Backlight PWM and night mode: 42 mA saved at 30% duty, 60 mA while it's dark.** The backlight is the largest load after the radio, and it's also what keeps a bedroom lit at 2 am.
3. **For a battery version, deep-sleep glance mode on a board without the dev-board parts.** The sleep floor drops from 4 to 12 mA to about 40 µA. That is the difference between 2 weeks and 4 months. Nothing else on the battery side comes close.

---

## 1. Current budget at 5 V USB

### 1.1 How the power flows today

USB-C 5 V comes in through the board's input diode into the AMS1117, which makes 3.3 V for the ESP32 and, through the 3V3 pin, for the display module. The module's own SOT-23 LDO feeds the ST7796S, and its backlight transistor switches the LEDs from module VCC (see `display-st7796.md` §1.2). A linear regulator passes its load current straight through, so

```
I_USB = I_3V3 + I_Q(AMS1117) + I_LED(power) + I_CP2102
P_USB = 5 V × I_USB
```

Everything at 3.3 V costs the same current at 5 V. The extra 1.7 V is burned as heat in the AMS1117: 0.33 W at 196 mA. That's warm but within an SOT-223's limits.

### 1.2 Component by component

| Component | Current at 3.3 V | Basis |
|---|---|---|
| ESP32, 240 MHz, Wi-Fi STA, `PowerSaveMode::None` | 115 mA (100-130) | RX/listen is 95-100 mA in the datasheet [1, Table 5-4]. With no power save the receiver never sleeps, so the chip sits at the RX figure plus CPU idle current. My estimate. |
| ESP32, 240 MHz, STA, `PowerSaveMode::Minimum` (DTIM modem sleep) | 35 mA (30-45) | ESP-IDF measures 31.1 mA at DTIM1, 28.8 at DTIM3, 29.7 at DTIM10, all at 160 MHz without DFS [2]. I added about 4 mA for 240 MHz, from the datasheet's modem-sleep floors of 27 mA at 160 and 30 at 240 [1, Table 4-2]. |
| ESP32 with automatic light sleep, Wi-Fi associated | 3 mA | ESP-IDF measures 2.2-3.3 mA [2]. In this stack it is experimental. See 4.6. |
| TLS fetch every 60 s, added average | +0.9 mA (no PS), +3.8 mA (PS) | Formula below |
| Rendering and SPI | about 1 mA | My estimate. The sign task wakes every 250 ms and pages every 6 s (`sign.rs`, `PAGE_TIME`). A full 480×320 RGB565 frame is 307 KB, 61 ms at 40 MHz. Most updates are small. |
| ST7796S controller, normal mode, all black | 12 mA typ, 16 max | ST7796S datasheet V1.0 §7.3 p.50 [5]. The test image is all black, which suits a sign that is mostly black. |
| ST7796S sleep-in (standby) | 0.025 mA typ, 0.05 max | Same table [5] |
| Backlight, full | 60 mA (30-90) | See below |
| Backlight, 30% PWM | 18 mA | LED current scales with duty |
| Module LDO quiescent | under 0.1 mA | SOT-23 part, likely XC6206 or ME6211 class (`display-st7796.md` §1.2) |
| Power LED on 3V3 | 2 mA (1-5) | Typical 1 kΩ resistor: (3.3 − 1.9 V) / 1 kΩ = 1.4 mA. Blue LEDs with small resistors run hotter. |
| AMS1117 quiescent | 5 mA typ, 11 max | AMS1117 datasheet [10] |
| CP2102, enumerated on a computer | 26 mA typ | CP2102 datasheet, older revision [9] |
| CP2102, USB suspend | 0.33 mA typ | Same [9]. On a wall charger the bridge should drop into suspend after 3 ms without host traffic. I couldn't confirm that every charger lets it. |

**Backlight.** I couldn't find a 3.3 V figure for this module, and it is the least certain number here. LCDwiki lists 95 mA backlight current at 5 V for its 3.5" ST7796 IPS module, 6 white LEDs, 0.5 W total [7]. It lists 104 mA at 5 V for its 3.5" ESP32 display [8]. Its 3.5" ILI9488 MSP3520 is listed at 0.62 W total [6]. At 3.3 V the white LEDs (forward voltage about 3.0 V) have only a few hundred millivolts across their resistor and transistor. The current is lower and changes a lot from module to module, so I used 60 mA with a 30 to 90 mA range. Measure it (section 1.5). Feeding module VCC from 5 V instead would make the sign brighter, put about 100 mA on the 5 V rail, and take that current out of the AMS1117.

**TLS burst.** Each fetch is 1 to 2 s of software P-256 plus SHA on the CPU, with the radio awake. I estimated 150 mA during the burst: the CPU fully busy (the datasheet's 240 MHz modem-sleep ceiling is 68 mA [1]) plus RX, plus short TX packets at 190 to 240 mA [1]. Its added average is

```
ΔI = (I_burst − I_base) × t_burst / T
no power save: (150 − 115) × 1.5 s / 60 s = 0.9 mA
modem sleep:   (150 −  35) × 2.0 s / 60 s = 3.8 mA   (the radio stays up a little longer around traffic)
```

With power save on, the fetch becomes a visible cost, but still small next to the backlight.

### 1.3 Totals at 5 V USB

The CP2102 is in suspend, as it should be on a wall charger. "Display off" means sleep-in with the backlight off.

| Wi-Fi | Backlight | 3.3 V rail | USB 5 V | Power | Energy per year |
|---|---|---|---|---|---|
| No power save (today) | full | 189 mA | **196 mA** | 0.98 W | 8.6 kWh |
| No power save | 30% | 147 mA | 154 mA | 0.77 W | 6.8 kWh |
| No power save | off | 116 mA | 123 mA | 0.62 W | 5.4 kWh |
| Modem sleep | full | 112 mA | 119 mA | 0.60 W | 5.2 kWh |
| Modem sleep | 30% | 70 mA | **77 mA** | 0.39 W | 3.4 kWh |
| Modem sleep | off | 39 mA | 46 mA | 0.23 W | 2.0 kWh |

Plugged into a computer, today's build draws about 219 mA, since the CP2102 enumerates and draws its 26 mA.

The full-range spread, all low or all high estimates, is roughly 160 to 280 mA today. The electricity doesn't matter much: 8.6 kWh a year is about $3 at California rates. What matters on USB is heat in the AMS1117, the light at night, and the fact that every milliamp saved here is a milliamp the battery version doesn't have to carry.

### 1.4 Wi-Fi power save in esp-radio

From the esp-radio 1.0.0-beta.1 source in the cargo registry:

- `esp_radio::wifi::PowerSaveMode { None, Minimum, Maximum }`, an `#[instability::unstable]` enum. The crate's `unstable` feature is already on in `Cargo.toml`.
- `WifiController::set_power_saving(&mut self, ps: PowerSaveMode) -> Result<(), WifiError>` maps to `esp_wifi_set_ps` with `WIFI_PS_NONE`, `WIFI_PS_MIN_MODEM` or `WIFI_PS_MAX_MODEM`.
- `WifiController::new` sets `PowerSaveMode::default()`, which is `None`, with the comment "The blob default is not the best for bandwidth." So every esp-radio station starts out with the receiver on full time.
- `Minimum` wakes for every DTIM beacon. `Maximum` wakes every `listen_interval` beacons, set by `StationConfig::with_listen_interval` (unstable builder; default 3, so 300 ms at the usual 100 ms beacon interval).

For a client that sends one request a minute, the only cost is latency on unsolicited inbound packets: up to one DTIM period, usually 100 to 300 ms. Nothing in this firmware waits for inbound traffic it didn't ask for. Start with `Minimum`, and try `Maximum` if a measurement shows a gain. Leave setup mode (the access point) at `None`.

### 1.5 Measure before trusting this

Ten minutes with two cheap meters will replace my estimates with your numbers:

1. Put a USB-C inline power meter between the charger and the board. Note the current at idle between fetches, during a fetch, and with the backlight forced low from a debug build.
2. Put a multimeter in series with the display module's VCC lead. Backlight on minus backlight off is the backlight current. Backlight off is the controller plus module LDO.
3. Put the measured values into section 1.2. Section 2 scales with them.

---

## 2. Battery runtime

### 2.1 Assumptions and formulas

- Cell: 3.7 V nominal LiPo or Li-ion. 90% of rated capacity is usable, allowing for cutoff margin and aging. Self-discharge is 2.5% a month, a typical room-temperature figure for LiPo. It turns out to matter at the glance-mode level.
- Converter: TPS63802 buck-boost at 90% for active loads [13]. It regulates across the whole 3.0 to 4.2 V range, so no capacity is stranded.
- Rail current to battery current: `I_bat = I_rail × 3.3 V / (η × 3.7 V) = 0.99 × I_rail`. In practice the two are the same.
- Runtime, continuous load: `t = 0.9 C / (I_bat + C × 0.025 / 720 h)`.
- Runtime, daily mix: `days = 0.9 C / (Q_day + 24 h × I_floor + C × 0.025 / 30)`.

### 2.2 Why the AMS1117 can't run from a LiPo

Its dropout is 1.1 V typical and 1.3 V maximum at 0.8 A, falling at lower load [10]. At this sign's current it's about 1 V. From a LiPo at 3.7 V the rail would sit near 2.7 V, below the ESP32's 3.0 V minimum. Only a nearly full cell (above about 4.1 V) gives a valid 3.3 V, and the 240 mA TX peaks [1] would cause brownout resets on top of that. Its 5 mA quiescent current also never stops. So a battery must not feed the dev board's 5 V or VIN pin directly.

Two ways around it:

- **Boost to 5 V, as a power bank does** (case a0). It's simple and keeps the dev board, but it goes 3.7 → 5 → 3.3 V through a linear stage: about 0.87 × 0.66 ≈ 57% end to end, plus every overhead in section 1. Also, IP5306-class chips and most power banks shut their output off when the load stays light. The IP5306 turns off after 32 s below 45 mA [15]. The optimized sign at 77 mA is near such limits, and a sleeping sign is far below them. So this route rules out deep sleep entirely.
- **Make 3.3 V directly** with a buck-boost or a low-dropout LDO, and get rid of the AMS1117 path. That's the custom-board route used below.

### 2.3 (a) and (b): always-on

| Case | 3.3 V rail | Battery | 1000 | 2000 | 3000 | 3500 |
|---|---|---|---|---|---|---|
| (a) today | 189 mA | 187 mA | 4.8 h | 9.6 h | 14.4 h | 16.8 h |
| (a0) stock dev board on IP5306 5 V boost (87%) | 196 mA at 5 V | 305 mA | 3.0 h | 5.9 h | 8.9 h | 10.3 h |
| (b0) stock board, modem sleep, BL 30%, on IP5306 | 77 mA at 5 V | 120 mA | 7.5 h | 15 h | 22.5 h | 26 h |
| (b) modem sleep, backlight 30% | 70 mA | 69 mA | 13 h | 26 h | 39 h | 45 h |
| (b') (b) plus automatic light sleep | 39 mA | 39 mA | 23 h | 47 h | 70 h | 82 h |

Even with the backlight off completely, an always-on TFT can't reach a month. The controller's 12 mA plus the radio's 3 to 35 mA is 15 to 47 mA, which works out to 11 to 34 Ah a month. "Take it on the go" is fine for a day. A 2000 mAh cell in mode (b) covers a day trip, and an 18650 covers two days. Two practical notes for on-the-go use: away from home the sign needs a phone hotspot, so the settings should hold a second SSID. And a USB power bank may switch off at the sign's 77 mA, as noted above.

### 2.4 (c) Glance mode

**Energy per glance.** These are rail figures, and the battery figures are the same within 1%:

| Phase | Time | Current | Charge | Notes |
|---|---|---|---|---|
| Wake, boot, display init | 0.5 s | 50 mA | 25 mAs | Bootloader, app start, ST7796 reset and 120 ms sleep-out |
| Wi-Fi join, fast path | 1.0 s | 130 mA | 130 mAs | Saved BSSID and channel, no scan, saved IP (4.4) |
| Wi-Fi join, slow path | 3.0 s | 120 mA | 360 mAs | Full scan plus DHCP |
| DNS, TLS, HTTP | 1.5 s | 150 mA | 225 mAs | Same as the 60 s fetch today |
| Show trains, standard | 45 s | 25 + 12 + 60 = 97 mA | 4365 mAs | Radio off, CPU idling at 80 to 160 MHz, controller, full backlight |
| Show trains, lean | 30 s | 2 + 12 + 30 = 44 mA | 1320 mAs | Light sleep between redraws, backlight 50% |

Standard glance: 4745 mAs = **1.31 mAh**. With the slow join: 1.37 mAh. Lean glance: **0.47 mAh**.

The screen time dominates. Wi-Fi and TLS together are about 8% of a standard glance. The fast reconnect saves only 5% of the energy. It's still worth doing because it cuts the wait from about 5 s to under 3 s, and the person pressing the button notices that. The backlight level and the on-screen time are the real energy controls.

**Sleep floor**, battery side, custom board:

| Item | Current | Source |
|---|---|---|
| ESP32 deep sleep, RTC timer and RTC memory | 10 µA, about 12 µA at the battery | [1, Table 4-2] |
| TPS63802 quiescent | 11 µA | [13] |
| TPS63802 feedback divider, 511 k + 91 k | 5.5 µA | [13], 3.3 V / 602 kΩ |
| BQ24074 battery leakage with no input | 6.5 µA max | [14], IBAT(PDWN) |
| Cell protection circuit | about 3 µA | Typical DW01-class; check the cell's datasheet |
| Battery voltage divider, 1 MΩ + 1 MΩ | 1.9 µA | 3.7 V / 2 MΩ |
| Display behind a load switch | about 0 | TPS22918 off-state leakage is negligible |
| Button pull-up | 0 | Draws only while pressed |
| **Total** | **about 40 µA** | **0.96 mAh/day** |

Self-discharge at 2.5% a month adds 0.83 mAh/day per 1000 mAh. On a 2000 mAh cell that's 1.7 mAh/day, more than the electronics. At this point the cell's own chemistry is as big a drain as the circuit.

Optional wake sources: an LIS2DH12 accelerometer for tap-to-wake (500 nA in power-down per LCSC data [18], a few µA in low-power sampling) adds about 0.05 to 0.15 mAh/day. ESP32 capacitive-touch wake keeps the touch state machine running in deep sleep. esp-hal 1.2.2 supports it (`src/touch.rs`, "wakeup from Deep Sleep"). I didn't find a reliable current figure, so measure it before relying on it. A plain button costs nothing.

**Why the stock dev board can't do this.** In deep sleep it keeps the AMS1117 quiescent current (5 mA typ, 11 max [10]), the power LED (1 to 5 mA), and a CP2102 that can draw milliamps when the USB side is unpowered. Lucidar measured 3.8 mA in deep sleep on an ESP32-DevKitC V4 with an AMS1117 and a CP2102N. It blames the LED and regulator, and puts the CP2102N at about 1.3 mA held in reset [11]. Forum reports for DevKit V1 boards range from about 4 mA, down to roughly 12 µA once the AMS1117 is cut out [12]. Those are reports, not verified measurements. With a 4 to 12 mA floor:

| Stock-board floor | 10 standard glances a day: 1000 | 2000 | 3000 | 3500 |
|---|---|---|---|---|
| 3.8 mA | 8.6 d | 17 d | 25 d | 29 d |
| 5 mA | 6.7 d | 13 d | 20 d | 23 d |
| 8 mA | 4.4 d | 8.7 d | 13 d | 15 d |
| 12 mA | 3.0 d | 5.9 d | 8.9 d | 10 d |

If you want to stay on the dev board for a prototype, desolder the AMS1117 (SOT-223, easy) and the power LED's resistor, and feed 3.3 V from your own regulator into the 3V3 pin. Whether the CP2102 still draws current then depends on whether that board runs it from VBUS or from the 3V3 net. Measure after the mod.

**Runtime**, custom board, 40 µA floor:

| Glances a day | Per-glance | 1000 mAh | 2000 mAh | 3000 mAh | 3500 mAh |
|---|---|---|---|---|---|
| 5 | standard | 108 d | 197 d | 270 d | 303 d |
| 10 | standard | 61 d | 115 d | 163 d | 186 d |
| 20 | standard | 32 d | 63 d | 91 d | 105 d |
| 5 | lean | 218 d | 362 d | 466 d | 507 d |
| 10 | lean | 139 d | 246 d | 332 d | 368 d |
| 20 | lean | 81 d | 150 d | 211 d | 238 d |
| 10 | standard, slow join | 58 d | 110 d | 157 d | 179 d |

Anything past about 6 months is academic. Self-discharge and the owner's charging habits will decide it. With a 1000 mAh cell, a month or two takes 10 to 20 glances a day. With 2000 mAh it's two months at 20 a day.

### 2.5 (d) Scheduled mode

Awake only in commute windows (7 to 9 am and 5 to 7 pm on weekdays, 4 h a day, 20 h a week), deep sleep otherwise, with the button still giving glances.

| Window | Settings | Rail | Per weekday | 1000 | 2000 | 3000 | 3500 |
|---|---|---|---|---|---|---|---|
| 4 h | (b): modem sleep, BL 30% | 70 mA | 277 mAh | 5 d | 9 d | 13 d | 16 d |
| 2 h | (b) | 70 mA | 138 mAh | 9 d | 18 d | 26 d | 31 d |
| 4 h | light sleep, BL 20% | 33 mA | 130 mAh | 9 d | 19 d | 28 d | 32 d |
| 2 h | light sleep, BL 20% | 33 mA | 65 mAh | 19 d | 37 d | 54 d | 62 d |

Four hours a day of a lit TFT costs more than 100 glances. A month needs one-hour windows, light sleep that actually works with Wi-Fi in this stack (4.6), and a 2000 mAh cell or bigger. Glance mode gives the same information at a tenth of the energy. Scheduled mode only wins if the sign must already be showing trains when you walk past without touching it.

### 2.6 (e) E-paper

A 4.2" 400×300 panel is the nearest size to the TFT. Waveshare's red/black/white 4.2" (B) module takes about 15 s per refresh at 38.4 mW and draws near zero between refreshes [16]. Waveshare says to refresh three-color panels no more often than every 180 s, and at least once every 24 h. It also says to put the panel to sleep between refreshes, because leaving it powered in the high-voltage state can damage it [16]. The black/white 4.2" takes about 5 s for a full refresh at 26.4 mW, and supports partial refresh, with an occasional full refresh to clear ghosting [17].

Energy per update (battery mAh). The ESP32 deep-sleeps between updates and reconnects every time:

```
wake + Wi-Fi (fast) + TLS          = 25 + 130 + 225  = 380 mAs
B/W partial refresh, 0.5 s         = 0.5 s × (25 + 10 mA)     =  18 mAs   → 0.109 mAh per update
tri-color full refresh, 15 s       = 15 s × (1 + 11.6 mA)     = 189 mAs   → 0.157 mAh per update
  (ESP32 light-sleeps while BUSY is high; 38.4 mW / 3.3 V = 11.6 mA)
  (if the ESP32 just idles and waits: 15 s × 36.6 mA → 0.256 mAh)
```

Updates pause from 1 to 5 am (20 active hours a day):

| Panel, interval | Updates a day | mAh a day | 1000 | 2000 | 3000 | 3500 |
|---|---|---|---|---|---|---|
| B/W, 1 min | 1200 | 131 | 7 d | 13 d | 20 d | 23 d |
| B/W, 2 min | 600 | 66 | 13 d | 26 d | 39 d | 45 d |
| B/W, 5 min | 240 | 26 | 32 d | 62 d | 91 d | 105 d |
| Tri-color, 3 min (fastest allowed) | 400 | 63 | 14 d | 28 d | 41 d | 47 d |
| Tri-color, 5 min | 240 | 38 | 23 d | 45 d | 66 d | 76 d |
| Tri-color, 10 min | 120 | 19 | 44 d | 84 d | 121 d | 139 d |
| Tri-color, 3 min in commute windows, 30 min otherwise | about 91 | 14 | 56 d | 106 d | 152 d | 173 d |

The panel itself is cheap. The cost is waking up and reconnecting to fetch.

My honest take on e-paper for this gift:

- It breaks the look. The real sign's character comes from orange-red LEDs glowing on black. Tri-color e-paper's red is a flat brick red on a reflective surface, and it can't be seen in the dark at all. Drawing red text on a black fill gets the layout right, but the light is gone.
- The 15 s three-color refresh flashes the whole panel black, white and red in turn. At the 3-minute minimum interval, it would be flashing about 8% of the time.
- Staleness defeats the purpose. A countdown that's up to 5 minutes old can't tell you whether to leave now. You could show clock times ("8:42") instead of minutes, which stay correct, but real BART signs don't do that.
- Black/white with partial refresh can update every minute without flashing, but it loses the red entirely, and at 1 minute it only lasts about 2 weeks on 2000 mAh.

E-paper is a good "hangs on the wall and you forget it's battery powered" device. It's a different product from a replica LED sign, though, and glance mode on the TFT gets similar battery life while keeping the look whenever someone is actually looking.

### 2.7 Verdict: is a month or two reachable?

| Mode | Verdict |
|---|---|
| (a) Always-on as today | No. 5 to 17 h. |
| (b) Dimmed with modem sleep | No. 1 to 2 days. Good for a day out. |
| (c) Glance, custom board | Yes. 1 month on 1000 mAh at 20 a day; 2 months on 2000 mAh at 20 a day; 4 months on 2000 mAh at 10 a day. |
| (c) Glance, stock dev board | No. 1 to 4 weeks, because of the AMS1117, LED and CP2102. |
| (d) Scheduled | Only with 2 h windows, working light sleep and 3000 mAh or more. Otherwise 1 to 4 weeks. |
| (e) E-paper | Yes at 5-minute updates on 2000 mAh or more, but it isn't the BART look. |

---

## 3. Charging and power-path architecture

### 3.1 Charger

| Part | LCSC | Price (qty 1) | Power path | Notes |
|---|---|---|---|---|
| TP4056 | C725790 | $0.09 | No | 1 A linear, has a TEMP pin for an NTC [18]. The load sits on the battery terminal, so the charger sees system current as charge current and may never terminate. Hobby modules pair it with DW01A + FS8205A protection. Fine for a prototype. |
| MCP73831T-2ACI/OT | C424093 | $0.79 | No | 500 mA, SOT-23-5, no NTC input [18]. Needs the usual load-sharing P-FET and Schottky from Microchip's application notes to get a crude power path. |
| IP5306 | C181692 | $0.28 | Power-bank style | Charger and 5 V boost in one. Auto-off below 45 mA after 32 s; 50 µA standby [15]. The I2C variant (C488349) can disable auto-off, but a 5 V boost still has to be regulated back down to 3.3 V. Only good for "stock dev board plus battery". |
| **BQ24074RGTR** | **C54313** | **$2.05** | **Yes (DPPM)** | 1.5 A linear; input rated to 28 V with OVP at 10.5 V; OUT regulated to 4.4 V while USB is present and connected to the cell when it isn't; 10 kΩ NTC input on TS; safety timers; open-drain CHG and PGOOD; programmable input limit [14]. LCSC showed only 677 in stock; that's plenty for gifts. |

Pick the **BQ24074**. Power path matters here because the sign runs while it charges. The charger then terminates correctly, a dead cell doesn't stop the sign from booting on USB, and the system rail never sees the cell's 3.0 V floor while USB is plugged in. Setup:

- `ISET`: RISET = KISET / ICHG = 890 AΩ / 0.5 A = 1.78 kΩ for 500 mA [14]. Keep charge current at or below 0.5C for a 1000 mAh cell.
- `ILIM` with EN2 = 1, EN1 = 0: RILIM = KILIM / IIN = 1610 AΩ / 1.0 A = 1.6 kΩ [14]. A USB-C sink with plain 5.1 kΩ Rd is only promised "Default USB" current unless it reads the source's CC advertisement. If you want strict compliance, use the 500 mA setting (EN1 = 1, EN2 = 0) or read the CC voltage on an ADC pin. The BQ24074's input DPM backs off if VBUS sags, so a weak source isn't dangerous.
- `TMR` floating gives the default timers (30 min precharge, 5 h fast charge) [14].
- `TS` goes to the cell's 10 kΩ NTC. If the cell has none, glue an NTC to it: HNTC0402-103F3950FB, C49247682. Never tie TS to a fixed resistor in a gift.

### 3.2 3.3 V regulator

| Part | LCSC | Price | Iq | Notes |
|---|---|---|---|---|
| AMS1117-3.3 (today) | n/a | n/a | 5 mA typ, 11 max | Dropout 1.1 V typ [10]. Unusable from a LiPo. |
| ME6211C33M5G-N | C82942 | $0.05 | 40-60 µA | 500 mA, 120 mV dropout at 100 mA [18]. Iq is too high for the glance floor. |
| AP2112K-3.3TRG1 | C51118 | $0.16 | 55 µA | 600 mA, 250 mV at 600 mA [18] |
| RT9080-33GJ5 | C841192 | $0.12 | 2 µA | 600 mA, 360 mV dropout at 600 mA [18]. The best LDO here. |
| **TPS63802DLAR** | **C2845237** | **$1.00** | **11 µA** | Buck-boost, 1.3-5.5 V in, 2 A at 3.3 V for VIN ≥ 2.3 V, 0.47 µH inductor, R1 511 k / R2 91 k for 3.3 V [13] |

An LDO from a LiPo works until the cell sags to 3.3 V plus dropout, about 3.55 V during TX peaks. That strands roughly the last 10% of the cell and gives about 87% average efficiency (3.3 / 3.8 V). The TPS63802 runs the cell down to 3.0 V at about 90%. The difference is around 10 to 15% runtime. Use the **TPS63802** on a real board. The **RT9080** is the simpler fallback if you'd rather not lay out a switcher: one SOT-23 and two capacitors, with a 2 µA Iq that also lowers the sleep floor to about 30 µA. Its input is BQ24074 OUT, which is at most 4.4 V, so both parts are within their input ratings.

### 3.3 Battery gauge

- **Voltage on an ADC1 pin, recommended.** 1 MΩ + 1 MΩ divider from the cell to GPIO34 (ADC1_CH6), with 100 nF at the pin so the high-impedance source can charge the ADC's sampling capacitor. It draws 1.9 µA all the time. Read it at wake, before Wi-Fi starts and before the backlight comes on, so you get something close to rest voltage. Calibrate each unit once against a multimeter and store the offset. For a handful of gifts that beats chasing the ESP32 ADC's nonlinearity. Rough thresholds: warn below 3.65 V (about 15% left), stop starting Wi-Fi below 3.45 V.
- **MAX17048G+T10** (C2682616, $2.18): ModelGauge state of charge over I2C, 3 µA in hibernate [18]. Use it if you want a real percentage on screen.
- **ADC1 conflict in the current firmware.** `main.rs` hands ADC1 to `TrngSource` for the program's whole lifetime. Either read the battery before creating `TrngSource` (natural in glance mode, since every wake is a fresh boot), or drop the `TrngSource` once Wi-Fi is running, since the ESP32's RNG is fed by RF noise while the radio is on. Check that against the esp-hal TRNG docs before you rely on it.

### 3.4 Display load switch

The ST7796S draws only 25 µA in sleep-in [5]. The module's LDO, its pull-ups, and the backlight transistor's bias network add more on top. Together they would roughly double the 40 µA floor. So cut the module's VCC:

- **TPS22918DBVR** (C131941, $0.25): 1 to 5.5 V, 2 A, 53 mΩ, active-high enable, controlled rise time, and quick output discharge [18]. The controlled slew matters because the module's input capacitors would otherwise dip the 3.3 V rail when it switches on.
- Budget option: **AO3401A** P-FET (C347476, $0.03) with a 100 kΩ gate pull-up to 3V3 and an RC on the gate for soft start.
- Put a 100 kΩ pull-down on the enable pin so the switch stays off while the ESP32's pads float in deep sleep.
- Before cutting power, drive SCK, MOSI, CS, DC, RST and BL low, or leave them floating. Otherwise the ESP32 back-powers the module through its input protection diodes.

### 3.5 One indicator light and a button window

**Light.** XL-1615SURUGC (C965839, $0.017) is a red/green LED with a common anode in a 1.6 × 1.5 mm package [18]. Put a light pipe over it in the housing window. Common anode suits open-drain drivers:

- Anode to 3V3. The red cathode, through 1 kΩ, is wired-OR between the BQ24074 `CHG` pin (open-drain, low while charging) and an ESP32 GPIO in open-drain mode. The green cathode, through 1 kΩ, goes to another ESP32 GPIO.
- **Red, steady**: charging. Hardware drives it, so it works even if the firmware hangs.
- **Green, steady**: on USB and charged. Firmware drives it when PGOOD is low and CHG has gone high. Plugging in USB wakes the ESP32 (3.6), so firmware is always awake while it's on USB.
- **Red, three blinks at wake**: low battery. An on-screen line says so as well.
- **Off**: on battery and fine. That costs 0 µA.

**Button.** B3U-1000P (C231329) or a 6 × 6 mm tact switch under a cap in the window. Wire it to **GPIO35** (RTC_GPIO5), with an external 100 kΩ pull-up and 100 nF to ground for debounce. GPIO34 to 39 are input-only and have no internal pulls, and esp-hal's deep-sleep wake docs ask for an external pull-up on a low-level wake pin anyway. Avoid GPIO36 and 39 for the button. Espressif's errata 3.11 describes short pull-downs on those two pins when ADC power changes [19], and users report a stream of false interrupts on GPIO36 once Wi-Fi power save is on [20].

GPIO0 (BOOT) is RTC_GPIO11 and can wake the chip from deep sleep, so it would work. It's also a strapping pin, though: hold it down through a power-on reset and the chip boots into the serial downloader. Keep BOOT as the hidden factory-reset button it is today, and give the housing its own button on GPIO35.

### 3.6 USB-C CC resistors

A USB-C source keeps VBUS off until it sees a 5.1 kΩ pull-down (Rd) on CC. USB-A to C cables don't care, because the legacy cable has Rp built in. A C-to-C cable from a USB-C charger will leave a board without Rd completely dead. Use **two separate 5.1 kΩ resistors, one on CC1 and one on CC2**. Tying both CC pins to a single resistor is a known mistake: the first Raspberry Pi 4 did it and wouldn't power from e-marked cables.

I couldn't find a schematic for the ELEGOO-style 30-pin USB-C board, and many cheap boards leave Rd off. Test it before choosing what to put in the gift box. Plug it into a USB-C wall charger (a phone or laptop brick) with a C-to-C cable. If the power LED stays dark, the resistors are missing. In that case either ship a USB-A charger and an A-to-C cable with it, or add Rd on a carrier board, since the connector's CC pins are too small to rework on the dev board. On the carrier board in section 5, Rd is two 0402 resistors (C25905).

### 3.7 Safety and shipping

- **Use protected cells only.** Pouch cells with a protection board, or "protected" 18650s, which are about 69 mm long because of the circuit in the cap. The BQ24074 controls charging. The protection circuit is the backstop for shorts, over-discharge and a failed charger. For an unprotected cell, add DW01A (C2927799) and FS8205A (C2830320) [18]. Buy 18650s from a reputable seller, since counterfeit capacity ratings are common.
- **NTC.** The BQ24074 pauses charging outside a safe temperature window when TS has a 10 kΩ NTC on the cell [14]. Don't bypass it.
- **Mechanical.** Pouch cells swell with age. Leave a few millimeters of space, keep screws and sharp edges away from the cell, and don't clamp it. Tell the recipient not to leave it in a car or a sunny window.
- **Connector polarity.** JST-PH 2-pin (S2B-PH-K-S, C173752) is the usual LiPo connector, but cell vendors don't agree on which pin is positive. Check every cell with a meter before plugging it in. Reversed polarity destroys the charger.
- **Shipping in the US (USPS Publication 52, Packaging Instruction 9D).** Lithium-ion cells installed in the equipment they power can go by surface or air: up to 20 Wh per cell and 100 Wh per battery. With no more than 4 cells or 2 batteries installed, no lithium battery mark is needed. Use rigid, crush-resistant packaging [21]. A 2000 mAh LiPo is 7.4 Wh and a 3500 mAh 18650 is 13 Wh, both well inside the limits. Used or defective devices go surface only. Pack it so the button can't be pressed in transit.
- **International (Packaging Instruction 9E).** Allowed only with the cell installed, only if the destination country accepts it, with no lithium markings permitted, at most 4 cells or 2 batteries, and in rigid cushioned packaging [21]. Couriers have their own rules.
- Ship cells partly charged. Air rules already cap state of charge at 30% for cells shipped on their own, and a partly charged installed cell is the safer habit. Handing gifts over in person avoids all of this.

---

## 4. Firmware changes per mode

All API names below come from the crate sources in the cargo registry (esp-hal 1.2.2, esp-radio 1.0.0-beta.1, esp-rtos 0.4.0, esp-hal-procmacros 0.23.0). I haven't compiled these snippets against the project.

### 4.1 Wi-Fi modem sleep (every mode that stays associated)

In `normal_mode`, right after `WifiController::new`:

```rust
use esp_radio::wifi::PowerSaveMode;
controller.set_power_saving(PowerSaveMode::Minimum).expect("set_power_saving");
```

Leave `setup_mode` alone. An access point must not doze. Optionally drop the CPU to 160 MHz (`CpuClock::_160MHz`): the datasheet's modem-sleep range goes from 30-68 mA at 240 MHz to 27-44 mA at 160 [1], while the TLS handshake gets about 1.5× slower. That trade is fine at a 60 s period.

### 4.2 Backlight PWM (LEDC)

Replace `Output::new(peripherals.GPIO32, Level::High, ...)` with an LEDC channel. The pattern follows esp-hal's `ledc` module docs:

```rust
use esp_hal::gpio::DriveMode;
use esp_hal::ledc::{channel::{self, ChannelIFace}, timer::{self, TimerIFace}, LSGlobalClkSource, Ledc, LowSpeed};

let mut ledc = Ledc::new(peripherals.LEDC);
ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
let t0 = mk_static!(timer::Timer<'static, LowSpeed>, ledc.timer::<LowSpeed>(timer::Number::Timer0));
t0.configure(timer::config::Config {
    duty: timer::config::Duty::Duty10Bit,
    clock_source: timer::LSClockSource::APBClk,
    frequency: Rate::from_khz(5),
})?;
let mut bl = ledc.channel(channel::Number::Channel0, peripherals.GPIO32);
bl.configure(channel::config::Config { timer: t0, duty_pct: 0, drive_mode: DriveMode::PushPull })?;
bl.start_duty_fade(0, 40, 600)?; // fade in after the first frame is drawn
```

Notes:

- 5 kHz is too fast to see and gentle on the module's backlight transistor. Above 20 kHz the transistor's switching edges start to eat into the low duty cycles.
- `duty_pct` steps in whole percent. For smooth night dimming at the low end, use `set_duty_hw` with raw 10-bit counts.
- Perceived brightness isn't linear. 30% duty looks closer to 60% bright. Offer three levels in the settings page (for example 15%, 40% and 100%).
- Keep the boot order the code uses today: clear the panel, then light the backlight.
- The timer borrows `ledc`, so if the channel moves into a task, `ledc` needs `'static` storage too (`mk_static!`).

### 4.3 Night mode

BART stops around 1 am and starts again about 5 am on weekdays, later on weekends. The firmware doesn't need a clock for this, because the feed has one. `Departures.time` holds the server time ("07:12:32 PM PST", `sign-core/src/etd.rs`). Two rules:

1. **Data-driven.** If the station has had no departures for 20 minutes, fade the backlight to off (or 2%) and fetch every 10 minutes. When departures come back, fade up again.
2. **Quiet hours.** Settable in the portal, default 01:00 to 05:00 from the feed's time: backlight off, one fetch every 10 to 15 minutes.

On USB this saves about 18 mA at 30% duty for 4 to 5 hours a night. That's small energy, but it removes a light from someone's bedroom. A cheap phototransistor on an ADC pin could add "dim when the room is dark" later.

### 4.4 Glance mode

On wake, for example when the button on GPIO35 is pressed:

1. Read `wakeup_cause()` (esp-hal `system`) and the battery voltage on ADC1, before `TrngSource` takes ADC1.
2. Raise the display enable, init the ST7796S, draw. If the RTC cache holds departures less than 3 minutes old, draw them right away, aged by the elapsed RTC time, with an "updating" marker, then fade in the backlight.
3. Join Wi-Fi with the cached BSSID, channel and IP, then fetch (fast path below).
4. Show trains for 30 to 60 s, counting the minutes down locally. Stop Wi-Fi as soon as the fetch is done (`controller.stop_async()` or drop the controller). Another button press restarts the timer, and a long press could extend it.
5. Go to sleep: fade the backlight out, DISPOFF (0x28), SLPIN (0x10), wait 120 ms, set the display pins low, drop the display enable, arm the wake pins, deep sleep.

Deep sleep with GPIO wake in esp-hal 1.2.2: a pin becomes a wake source when it `listen`s for a level and asks for the low-power path. The driver sorts the pins onto ext0 and ext1 itself (`src/gpio/wakeup/ext1_v1.rs`). On the ESP32, ext1 has one level for all its pins. Pins that wake on a different level take ext0, which wakes on one pad at its own level.

```rust
use esp_hal::gpio::{Event, Input, InputConfig, WakeupConfig};
use esp_hal::rtc_cntl::sleep::{LowPower, RtcSleepConfig};

let lp_path = WakeupConfig::default().with_low_power_path(true);

let mut button = Input::new(peripherals.GPIO35, InputConfig::default()); // external 100 k pull-up
button.listen(Event::LowLevel);
button.apply_wakeup_config(&lp_path)?;                                   // gets ext0, wakes on low

let mut vbus = Input::new(peripherals.GPIO36, InputConfig::default());   // 100 k / 100 k divider from VBUS
vbus.listen(Event::HighLevel);
vbus.apply_wakeup_config(&lp_path)?;                                     // gets ext1, wakes on high: USB plugged in

let mut lp = LowPower::new(peripherals.LPWR);
// Scheduled mode also sets: lp.set_wakeup_deadline(next_window_start);
lp.sleep_deep(RtcSleepConfig::deep());
```

If you start esp-rtos with `esp_rtos::sleep::configure(peripherals.LPWR)`, use its `deep_sleep` handle instead. It calls the same `sleep_deep(RtcSleepConfig::deep())`.

**Fast reconnect via RTC memory.** `connect_async()` returns `ConnectedInfo { bssid, channel, .. }`, and the DHCP lease gives the address, gateway and DNS. Keep them in RTC fast memory, which survives deep sleep:

```rust
#[derive(Clone, Copy)]
#[repr(C)]
struct WifiCache { magic: u32, bssid: [u8; 6], channel: u8, prefix: u8, ip: [u8; 4], gw: [u8; 4], dns: [u8; 4], crc: u32 }
unsafe impl esp_hal::Persistable for WifiCache {} // only plain integers and arrays inside

#[esp_hal::ram(unstable(rtc_fast, persistent))]
static mut WIFI_CACHE: WifiCache = /* zeroed */;
```

- After a cold power-on, persistent memory holds garbage. Check `magic` and a CRC, and treat a mismatch as "no cache".
- On wake, build the station config with the cached BSSID and channel. `StationConfig` has `bssid: Option<[u8; 6]>` and `channel: Option<u8>` with `BuilderLite` setters. Check the exact `with_bssid` and `with_channel` signatures. This skips the scan.
- Start `embassy_net` with `Config::ipv4_static(StaticConfigV4 { .. })` from the cached lease instead of DHCP. To keep this safe, only reuse a lease that is still inside its lease time (track it with the RTC timer). On any failure, drop the cache and redo a scan and DHCP.
- The same cache can hold the last `Departures` and the RTC time it was fetched, for the instant first frame in step 2.

### 4.5 Scheduled mode

Same deep-sleep path, plus `LowPower::set_wakeup_deadline(Instant)` for the next window start. The window boundaries come from the feed's time, saved in RTC memory with the RTC-timer value at which it was read. The RTC slow clock drifts a few percent over hours, so resync on each fetch. Inside a window, run the always-on loop with modem sleep and a dim backlight. At the end of the window, go through the glance-mode sleep path.

### 4.6 Automatic light sleep (experimental)

esp-rtos 0.4.0 has automatic light sleep: `esp_rtos::sleep::configure(LPWR)` returns a `light_sleep_hook` for `start_with_idle_hook`, `WakeLock` blocks it, and `ESP_RTOS_CONFIG_LIGHT_SLEEP_MIN_US` (default 1000) sets the minimum idle time. The crate marks it experimental. I found nothing showing it coordinates with the Wi-Fi blob's beacon timing the way ESP-IDF's power management does. ESP-IDF's 2 to 3 mA figure [2] is the target, not something this stack has shown. During a glance with Wi-Fi stopped, light sleep between redraws is lower risk, and it's what the lean glance assumes. Hold a `WakeLock` during SPI DMA transfers.

### 4.7 Per-mode summary

| Change | USB always-on | Battery, glance | Battery, scheduled | E-paper |
|---|---|---|---|---|
| `PowerSaveMode::Minimum` | Yes | Not needed: Wi-Fi is stopped after each fetch | Yes, inside windows | Not needed |
| Backlight via LEDC, user levels | Yes | Yes, fade in and out | Yes | n/a |
| Night mode (data-driven plus quiet hours) | Yes | n/a | Yes | Pause updates |
| Deep sleep, wake on GPIO35 low and VBUS high | No | Yes | Yes, plus a timer | Timer |
| RTC cache: BSSID, channel, lease, last departures | Optional | Yes | Yes | Yes |
| Display power off via load switch, SLPIN first | No | Yes | Outside windows | Panel deep sleep after every refresh |
| Battery ADC, low-battery LED and message | No | Yes | Yes | Yes |
| Switch to always-on while VBUS is present | n/a | Yes | Yes | Optional |
| Read the battery before `TrngSource` takes ADC1 | n/a | Yes | Yes | Yes |
| Second SSID (phone hotspot) | Optional | Useful | Useful | Optional |

---

## 5. Recommendation

**Default product: USB-C, always on.** Keep the dev board and the TFT. Make three firmware changes: modem sleep (4.1), backlight PWM at about 40% with user levels (4.2), and night mode (4.3). The result is about 80 to 100 mA at 5 V and under 0.5 W, with no hardware change and no battery-safety questions for the people receiving it. Before choosing the cable and charger for the box, check the CC resistors (3.6).

**Optional battery version: carrier board, glance on battery, live when docked.** The ESP32-WROOM-32E module goes directly on a small PCB that also carries the power parts, which replaces the dev board. On battery the sign sleeps at about 40 µA and wakes for 30 to 60 s on a button press. Plugged into USB, it wakes (VBUS on GPIO36), charges, and behaves like the default product. With a protected 2000 mAh LiPo that's about 2 months at 20 glances a day and about 4 months at 10.

I would skip e-paper and the commute-window schedule. E-paper is a different object. The schedule costs ten times the energy of glance mode for a small convenience.

### 5.1 Block diagram (battery carrier board)

```
                       5.1k Rd x2 on CC1, CC2
USB-C receptacle ──VBUS──┬────────────────────────► BQ24074 IN
  (D+/D- optional,       │                           │  OUT: 4.4 V on USB, VBAT on battery
   via USBLC6-2SC6)      │                           ▼
                         │                       TPS63802 buck-boost ──► 3V3 rail
                         │                           ▲                    ├──► ESP32-WROOM-32E
                         │                  BAT ◄────┘                    ├──► TPS22918 ──► display module VCC
                         │                   │                            │      (EN = GPIO25, 100k pull-down)
                         │          protected LiPo 1S + 10k NTC ──► TS    └──► LED anode (XL-1615 R/G)
                         │                   │
                         │                   └── 1M / 1M + 100nF ──► GPIO34 (ADC1_CH6, battery)
                         └── 100k / 100k ───────────────────────► GPIO36 (VBUS sense, ext1 wake)

BQ24074 CHG (open drain) ─┬── red cathode (1k)        GPIO13 (open drain) ── red cathode (wired-OR)
BQ24074 PGOOD ── GPIO39 (read only, never a wake source)
GPIO26 ── green cathode (1k)       GPIO35 ── button to GND, 100k pull-up, 100nF   (ext0 wake)
Display: SCK 18, MOSI 23, CS 5, DC 27, RST 33, BL 32 (LEDC) as today
Programming: 6-pin header (3V3, GND, TX, RX, EN, IO0) to an external USB-UART adapter
```

No USB-UART bridge sits on the board, so nothing draws power in sleep. If you'd rather flash over the USB-C port, add a CH340C (C84681, $0.59) powered from VBUS only, with 1 kΩ in series on TX and RX so the sleeping ESP32 doesn't back-power it.

### 5.2 BOM (battery carrier board)

Prices are LCSC qty-1 prices from the `lcsc` registry on 2026-11-02 [18].

| Function | Part | LCSC | Price | Notes |
|---|---|---|---|---|
| MCU | ESP32-WROOM-32E-N4 | C701341 | $3.82 | Same chip family as today's board |
| USB-C receptacle | KH-TYPE-C-16P | C709357 | $0.08 | 16-pin, power plus USB 2.0 |
| CC pull-downs | 5.1 kΩ 1% 0402, ×2 | C25905 | $0.001 | One per CC pin |
| USB ESD (if D+/D- are used) | USBLC6-2SC6 | C2827654 | $0.04 | |
| Charger, power path | BQ24074RGTR | C54313 | $2.05 | RISET 1.78 k (500 mA), RILIM 1.6 k (1 A), TMR open |
| NTC (if the cell has none) | HNTC0402-103F3950FB | C49247682 | $0.02 | 10 k, B3950 |
| 3.3 V buck-boost | TPS63802DLAR | C2845237 | $1.00 | R1 511 k, R2 91 k |
| Inductor | DFE252012F-R47M | C703140 | $0.18 | 0.47 µH as the datasheet asks |
| Alternative to the two parts above | RT9080-33GJ5 | C841192 | $0.12 | 2 µA LDO, 600 mA |
| Display load switch | TPS22918DBVR | C131941 | $0.25 | Or AO3401A, C347476, $0.03 |
| Indicator LED | XL-1615SURUGC | C965839 | $0.02 | Red/green, common anode, under a light pipe |
| Button | B3U-1000P | C231329 | $0.18 | Or a 6×6 tact switch under a cap |
| Battery connector | S2B-PH-K-S | C173752 | $0.04 | Check cell polarity |
| Optional fuel gauge | MAX17048G+T10 | C2682616 | $2.18 | Instead of the divider |
| Optional tap wake | LIS2DH12TR | C110926 | $0.84 | INT1 to an ext1 pin |
| Protection (unprotected cells only) | DW01A + FS8205A | C2927799, C2830320 | $0.09 | Prefer protected cells |
| Cell | Protected 1S LiPo, 2000 mAh, with NTC lead; or a protected 18650 with holder | not LCSC | | 7.4 Wh / 13 Wh |

Core silicon comes to about $7.50 a board before passives, the PCB and the cell. The display, the housing and the firmware stay as they are.

A shortcut for a first battery prototype: some off-the-shelf ESP32 boards are designed for battery use and publish a measured deep-sleep figure. Only trust one after measuring its floor yourself, since the whole glance-mode result depends on that number.

---

## Sources

1. Espressif, *ESP32 Series Datasheet* v5.3: Table 4-2, Power Consumption by Power Modes (p. 30); Table 5-4, Current Consumption Depending on RF Modes (p. 52). https://documentation.espressif.com/esp32_datasheet_en.pdf
2. ESP-IDF Programming Guide (ESP32), *Low Power Mode in Wi-Fi Scenarios*: modem sleep 31.12 / 28.81 / 29.66 mA at DTIM1/3/10, 160 MHz; auto light sleep 3.34 / 2.33 / 2.19 mA; deep sleep about 5 µA. https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/low-power-mode/low-power-mode-wifi.html
3. esp-radio 1.0.0-beta.1 source: `src/wifi/mod.rs` (`PowerSaveMode`, `set_power_saving`, default `None` in `WifiController::new`), `src/wifi/sta/mod.rs` (`StationConfig`, `ConnectedInfo`). Local cargo registry.
4. esp-hal 1.2.2 source: `src/rtc_cntl/sleep/mod.rs` (`LowPower`, `sleep_deep`, `set_wakeup_deadline`), `src/gpio/wakeup/` (`WakeupConfig::low_power_path`, ext0/ext1 allocation), `src/ledc/`, `src/touch.rs`. esp-rtos 0.4.0 `src/sleep.rs`. esp-hal-procmacros 0.23.0 `#[ram(unstable(rtc_fast, persistent))]`. Local cargo registry.
5. Sitronix, *ST7796S datasheet* V1.0 (2014-11), §7.3 Power Consumption, p. 50: normal mode IDD 12 mA typ / 16 max, standby 0.025 / 0.05 mA. https://www.lcdwiki.com/res/MSP3525_MSP3526/ST7796S-Sitronix.pdf
6. LCDwiki, 3.5" SPI module ILI9488, MSP3520: 0.62 W. http://www.lcdwiki.com/3.5inch_SPI_Module_ILI9488_SKU:MSP3520
7. LCDwiki, 3.5" IPS SPI module ST7796, MSP3525/3526: 5 V, 0.5 W, backlight 95 mA, 6 white LEDs. https://www.lcdwiki.com/3.5inch_IPS_SPI_Module_ST7796
8. LCDwiki, 3.5" ESP32-32E display: backlight 104 mA; 160 mA total with only the display working, at 5 V. https://www.lcdwiki.com/3.5inch_ESP32-32E_Display
9. Silicon Labs, *CP2102 datasheet* (older revision, SparkFun mirror): supply current 26 mA typ, suspend 330 µA typ. https://www.sparkfun.com/datasheets/IC/cp2102.pdf. The current Silicon Labs copy returned 403 and may list different values.
10. Advanced Monolithic Systems, *AMS1117 datasheet*: quiescent 5 mA typ / 11 max; dropout 1.1 V typ / 1.3 V max at 0.8 A. http://www.advanced-monolithic.com/pdf/ds1117.pdf
11. Lucidar, *ESP32-DevKitC V4 real current consumption in deep sleep mode*: 3.8 mA deep sleep, 43 mA active, CP2102N about 1.3 mA held in reset. https://lucidar.me/en/esp32/power-consumption-of-esp32-devkitc-v4/
12. esp32.com, "Power Consumption in Deep Sleep" thread: DevKit V1 about 4 mA, about 12 µA after removing the AMS1117. From a search summary; not verified. https://esp32.com/viewtopic.php?p=96892
13. Texas Instruments, *TPS63802 datasheet* SLVSEU9D: 1.3-5.5 V input, 11 µA Iq, 2 A at 3.3 V, 0.47 µH, R1 511 k / R2 91 k for 3.3 V. https://www.ti.com/lit/ds/symlink/tps63802.pdf
14. Texas Instruments, *BQ2407x datasheet* SLUS810N: BQ24074 OVP 10.5 V, OUT 4.4 V, IBAT(PDWN) 6.5 µA max, KISET 890 AΩ, KILIM 1610 AΩ, TS for 10 kΩ NTC, timers, open-drain CHG/PGOOD. https://www.ti.com/lit/ds/symlink/bq24074.pdf
15. Injoinic, *IP5306 datasheet* (distributor copies): load-removal shutdown after 32 s below 45 mA; standby about 50 µA. https://static.chipdip.ru/lib/688/DOC045688389.pdf and https://www.rcscomponents.kiev.ua/datasheets/IP5306.pdf
16. Waveshare, *4.2inch e-Paper Module (B) Manual*: 400×300 red/black/white, 15 s refresh, 38.4 mW, refresh interval of at least 180 s, refresh at least every 24 h, sleep the panel after refreshing. https://www.waveshare.com/wiki/4.2inch_e-Paper_Module_(B)_Manual
17. Waveshare, *4.2inch e-Paper Module Manual* (black/white): about 5 s full refresh, 26.4 mW, partial refresh needs an occasional full refresh. https://www.waveshare.com/wiki/4.2inch_e-Paper_Module_Manual
18. LCSC part data through the `lcsc` CLI registry, retrieved 2026-11-02. Part numbers, stock and qty-1 prices as quoted.
19. Espressif, *ESP32 Series SoC Errata*, 3.11, GPIO36 and GPIO39 inputs pulled down briefly when SAR ADC or AMP power changes. https://docs.espressif.com/projects/esp-chip-errata/en/latest/esp32/03-errata-description/esp32/gpio-inputs-pulled-down.html
20. espressif/esp-idf issue #4585: false interrupts on GPIO36 once Wi-Fi connects; reported fixed by `WIFI_PS_NONE`. https://github.com/espressif/esp-idf/issues/4585
21. USPS, *Publication 52*, Packaging Instruction 9D (domestic) https://pe.usps.com/text/pub52/pub52apxc_032.htm and 9E (international) https://pe.usps.com/text/pub52/pub52apxc_033.htm
