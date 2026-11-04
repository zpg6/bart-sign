# ST7796 3.5" SPI TFT on ESP32: research notes

Target: 3.5" 320x480 ST7796 TFT, run in landscape 480x320, driven from an original ESP32 with Rust no_std (esp-hal 1.x, embedded-hal 1.0, Embassy) through `mipidsi` and `embedded-graphics`.

Fixed wiring: SCK=GPIO18, MOSI=GPIO23, CS=GPIO5, DC=GPIO27, RST=GPIO33, BL=GPIO32 (active high), MISO not connected.

Short answer: `mipidsi` 0.10.0 is a correct choice. It supports the ST7796, the API fits, and it will light the panel. Its ST7796 init is the generic ST7789 one, though. It never touches the Sitronix vendor registers (power, VCOM, gamma, source timing), and its default orientation is mirrored on every ST7796 module I could find. Both are fixable without forking the crate. Section 7 has the exact config, and section 8 has the init to send.

Citations: `[DS pN]` is the Sitronix ST7796S datasheet V1.0, 2014/11, 279 pages. From https://www.displayfuture.com/Display/datasheet/controller/ST7796s.pdf. The copies on lcdwiki (https://www.lcdwiki.com/res/MSP3525_MSP3526/ST7796S-Sitronix.pdf) and Espressif (https://dl.espressif.com/dl/schematics/ST7796S_SPEC_V1.0.pdf) are the same V1.0 document.

---

## 1. The actual module

The coordinator relayed two facts.

The Amazon listing is XIITIA "2pcs 3.5in SPI TFT LCD Display Module 320x480, ST7796 driver", https://www.amazon.com/dp/B0DFWL9M9B. It lists 320x480, SPI or 8/16-bit parallel, a 3V to 5V module supply, a 48.96 x 73.44 mm active area, and a 0.153 mm pixel pitch. Pins are GND, VCC, SCL, SDA, RST, DC, CS, BL, SDA-0.

From a photo, the board is a blue PCB with "3.5"TFT" silkscreen. The controller label is partly hidden and reads "ST779?". There is a 9-pin SPI header, numbered 9 to 1 as SDA-O (MISO), BL, CS, DC, RST, SDA (MOSI), SCL (SCK), VCC, GND. There is also an 8080-16P FPC (D15..D0, BL, RST, RD, WR, DC, CS, VCC, GND), an 8080-8P FPC, and a 40-pin FPC to the glass. U8 is a SOT-23 regulator and U7 is a backlight transistor. The interface solder-jumper table reads:

| Jumper label | IM0 | IM1 | IM2 |
|---|---|---|---|
| P8  | 1 | 1 | 0 |
| P16 | 0 | 1 | 0 |
| SPI | 1 | 1 | 1 |
| SPI3 | 1 | 0 | 1 |

### 1.1 IM table vs the ST7796S datasheet

The ST7796S has three mode pins, IM2..IM0 [DS p42, also p59]:

| IM2 IM1 IM0 | ST7796S mode [DS p42] | Board label | Match |
|---|---|---|---|
| 0 1 1 | 8080 8-bit, DB[7:0] | P8 | yes |
| 0 1 0 | 8080 16-bit, DB[15:0] | P16 | yes |
| 1 1 1 | 4-line SPI, SDA/SDO | SPI | yes |
| 1 0 1 | 3-line SPI, SDA/SDO | SPI3 | yes |

All four rows match the datasheet bit for bit. [DS p89] also says 4-line SPI is selected with IM[2:0] = "111", and [DS p85] says 3-line is "101". For this project the jumper must sit on SPI (111). With 3-line SPI selected the DC pin is ignored and every byte carries a 9th D/C bit, which `mipidsi` cannot produce.

The table does not fit an ST7789. The ST7789V has four mode pins, IM3..IM0, and its 4-line SPI code is a 4-bit value, so a 3-bit "SPI = 111" table cannot describe it. The ST7789 family also tops out at 240x320 and cannot drive a 320x480 glass at all. The 3-bit IM scheme is shared with the ILI9486/ILI9488 lineage that the ST7796S was modeled on. I did not verify the ILI948x tables against a datasheet here, so the IM table alone does not rule those out. The "ST779?" label, the 320x480 resolution, and the listing do. With MISO wired, `RDID4` (0xD3) would settle it: it returns a dummy byte, 0x00, then 0x77 0x96 [DS p233]. See 9.4.

### 1.2 Supply voltage

VCC on the header accepts 3.3V to 5V. The listing says 3V to 5V. U8 is a SOT-23 LDO, almost certainly a 3.3V part such as an XC6206/662K or ME6211/ME6217 (input max about 6V). It feeds the controller's VDD/VDDI. Every lcdwiki ST7796 SPI module documents the same design. The MSP4020/4021 page lists "VCC power voltage 3.3V~5V" and pin 1 as "5V/3.3V power input" (https://www.lcdwiki.com/4.0inch_SPI_Module_ST7796).

- Logic is 3.3V. The controller runs from the LDO output, so the ESP32's 3.3V GPIO drive is correct. No level shifting is needed, and none should be added.
- Feed VCC from 5V if you can. The LDO then has headroom, and the backlight LEDs, which on these boards are usually fed from VCC through U7 and a resistor, run at full brightness. At 3.3V in, the LDO output sags to about 3.0 to 3.1V, which is still within spec, and the backlight is dimmer.
- Backlight: U7 is a transistor driven from BL, so BL high means on, matching the lcdwiki boards ("LED: backlight control, high level lighting"). GPIO32 can drive it with LEDC PWM for dimming. Expect roughly 60 to 100 mA of backlight current from VCC. lcdwiki lists 95 to 104 mA for its 3.5" boards.

### 1.3 ST7796S vs ST7796U

The listing only says "ST7796". The same 48.96 x 73.44 mm, 0.153 mm-pitch glass ships with both suffixes:

- lcdwiki MSP3525/MSP3526 is an IPS panel with an ST7796U (https://www.lcdwiki.com/3.5inch_IPS_SPI_Module_ST7796).
- lcdwiki E32R35T/E32N35T is a TN panel with an ST7796U, panel spec QD3579 ("LCD DRIVE IC: CTC3.5+ST7796U") (https://www.lcdwiki.com/3.5inch_ESP32-32E_Display).
- lcdwiki MSP4020/4021 is a 4.0" TN panel with an ST7796S.

I found no public ST7796U datasheet. In practice the two are command-compatible. Both lcdwiki ST7796U products link the ST7796S V1.0 datasheet as their controller document, and their demo code uses the same command set: CSCON 0xF0 0xC3/0x96 unlock, B4/B7/C0/C1/C2/C5/E8/E0/E1, and the same MADCTL values. What differs between modules is the glass. TN vs IPS decides inversion and the best gamma/VCOM. Treat the controller as "ST7796" and pick inversion and gamma per panel (section 8).

---

## 2. Datasheet facts

### 2.1 SPI write clock

4-line SPI timing [DS p54, section 7.4.3], specified at VDDI=1.8V, 25 °C:

| Symbol | Parameter | Min |
|---|---|---|
| TSCYCW | Serial clock cycle, write | 66 ns, so 15.15 MHz |
| TSHW / TSLW | SCL high / low width, write | 15 ns each |
| TSCYCR | Serial clock cycle, read | 150 ns, so 6.67 MHz |
| TCSS / TCSH | CS setup / hold, write | 15 ns |
| TCHW | CS high pulse width | 40 ns |
| TDCS / TDCH | D/CX setup / hold | 10 ns |
| TSDS / TSDH | Data setup / hold | 10 ns |

3-line SPI is identical [DS p53]. On paper, writes top out at 15 MHz. In practice the high and low pulse minimums of 15 ns each allow 33 MHz, the table is specified at 1.8V I/O while these boards run 3.3V, and the whole ecosystem runs the part at 40 to 80 MHz. lcdwiki's own ESP32 demo for the 3.5" ST7796 module ships `SPI_FREQUENCY 80000000` (MSP3525 demo zip, TFT_eSPI `User_Setup.h`). TFT_eSPI's ST7796 ESP32 setup says 80 MHz is OK on ESP32. Adafruit's ST7796S driver defaults to 40 MHz. See 6.1 for what to pick.

The serial format is MSB first, sampled on the rising SCL edge [DS p43, p90], so SPI mode 0 works. Mode 3 would too. RGB565 is two bytes per pixel, high byte first, with COLMOD 0x55 or 0x05 [DS p92].

### 2.2 Reset and sleep timing

- RESX is active low and "must be applied to properly initialize the chip" [DS p42].
- After power-on, HW reset, or SW reset, the chip is in Sleep In [DS p160-161, "Default" table].
- After SWRESET (0x01), wait 5 ms before the next command. If SWRESET is sent while in sleep-in, wait 120 ms before SLPOUT [DS p141-142].
- After SLPOUT (0x11), wait 5 ms before any command. Wait 120 ms before a later SLPIN [DS p161]. The usual rule of 120 ms after SLPOUT before DISPON is vendor practice, not a datasheet requirement. All vendor inits do it, and it costs nothing at boot.
- Power sequencing: VDD/VDDI can come up in any order. If RESX is not held stable during power-up, apply a HW reset afterwards [DS p271, note 4]. A RESX falling edge while in sleep-out needs 120 ms before power is removed [DS p271].
- This V1.0 datasheet has no tRW/tRT table for the reset pulse. Sitronix siblings such as the ST7789V spec tRW of at least 10 µs and a reset-complete time of 5 ms from sleep-in or 120 ms from sleep-out. Vendor code is far more generous: lcdwiki holds RST low 100 ms and waits 50 ms, Espressif's `esp_lcd_st7796` holds 10 ms and waits 120 ms. Use 10 ms low and 120 ms high-wait.

### 2.3 MADCTL (0x36) [DS p183-184]

| Bit | Name | Meaning |
|---|---|---|
| 7 | MY | Row address order |
| 6 | MX | Column address order |
| 5 | MV | Row/column exchange |
| 4 | ML | Vertical refresh order, 0 = top to bottom |
| 3 | RGB | 0 = RGB, 1 = BGR color filter |
| 2 | MH | Horizontal refresh order, 0 = left to right |

The datasheet's SS bit in DFC (0xB6, 2nd parameter, bit 5) also flips the source scan direction. Changing SS also swaps the R/B assignment to source pins [DS p219]. Leave SS = 0 and use MADCTL.

### 2.4 COLMOD (0x3A) [DS p190]

D6..D4 is the RGB-interface format and D2..D0 the MCU-interface format, where 101 is 16 bpp. 0x55 sets both to 16 bpp, and 0x05 sets just the MCU side, which is all SPI uses. Both work. `mipidsi` sends 0x55.

### 2.5 Inversion

INVON (0x21) [DS p166] and INVOFF (0x20) [DS p165] invert the color data sent to the panel. Whether you need INVON is a property of the glass, not the controller:

- The IPS 3.5" ST7796U module (lcdwiki MSP3525) sends `0x21` in both its STM32 and ESP32 demos.
- The TN 4.0" ST7796S module (lcdwiki MSP4020) never sends 0x21.
- The Amazon listing does not say IPS, so it is probably TN and wants INVOFF. Check it empirically (9.1).

DIC (0xB4) is a different thing. It selects the LC drive inversion scheme: 00 column, 01 1-dot (default), 10 2-dot [DS p216]. It does not invert colors.

### 2.6 Vendor ("Command 2") registers and defaults

Command 2 registers need the CSCON unlock first: `F0 C3` then `F0 96`. Lock again with `F0 3C` then `F0 69` [DS p239].

| Cmd | Name | Default [DS page] | Purpose |
|---|---|---|---|
| B0 | IFMODE | 00 [p212] | SPI_EN, RGB-interface polarity |
| B1 | FRMCTR1 | A0 10 [p213] | Frame rate, normal mode |
| B4 | DIC | 01 [p216] | Drive inversion: column / 1-dot / 2-dot |
| B5 | BPC | 02 02 00 04 [p217] | Blanking porch |
| B6 | DFC | 80 02 3B [p218] | GS/SS/SM scan, NL = 480 lines |
| B7 | EM | 06 [p222] | Entry mode, EPF 16-to-18 bit expansion |
| C0 | PWR1 | 80 25 [p223] | AVDD/AVCL, VGH/VGL |
| C1 | PWR2 | 13 [p224] | VRH, GVDD level. 0x13 = 4.5V, 0x06 = 3.85V |
| C2 | PWR3 | not given [p227] | Source / gamma driving current |
| C5 | VCMPCTL | 1C [p227-228] | VCOM. 0x18 = 0.9V, 0x1C = 1.0V, 0x21 = 1.125V, 0x24 = 1.2V |
| E0 / E1 | PGC / NGC | not given [p233-235] | Positive / negative gamma, 14 bytes each |
| E8 | DOCA | 40 8A 00 00 25 0A 38 33 [p237-238] | Source EQ, gate start/end timing |
| F0 | CSCON | 00 [p239] | Command 2 lock/unlock |

The datasheet gives no defaults for gamma or PWR3. Without the vendor init you get whatever the die powers up with, which the panel maker did not tune. That is the main image-quality gap in `mipidsi`'s init.

B7 EPF note [DS p222]: in RGB565 mode the controller stores 18-bit pixels. With the default EPF = 00 the missing red and blue LSB is 0, so 0xF800 red is stored as 62/63, not full scale. Vendor inits use 0xC6 (EPF = 11, copy G's LSB). For a red-on-black sign, `B7 = 0x86` (EPF = 10, "MSB written to LSB") gives true full-scale red. The difference is small.

---

## 3. Vendor init sequences, compared

Four real sequences. "TFT_eSPI/Espressif" is effectively the Sitronix reference used by most ST7796S 3.5"/4.0" boards.

### 3.1 TFT_eSPI `ST7796_Init.h` and Espressif `esp_lcd_st7796` default

Sources: https://github.com/Bodmer/TFT_eSPI/blob/master/TFT_Drivers/ST7796_Init.h and https://github.com/espressif/esp-bsp/blob/master/components/lcd/esp_lcd_st7796/esp_lcd_st7796_general.c (`vendor_specific_init_default`).

```
SWRESET, 120ms (TFT_eSPI only)        SLPOUT 11, 120ms
F0 C3 / F0 96                          unlock
36 48                                  MX|BGR, portrait
3A 55
B4 01                                  1-dot inversion
B6 80 02 3B                            (TFT_eSPI only; these are the defaults)
B7 C6                                  (Espressif; TFT_eSPI omits)
E8 40 8A 00 00 29 19 A5 33
C1 06                                  GVDD 3.85V
C2 A7
C5 18                                  VCOM 0.9V
E0 F0 09 0B 06 04 15 2F 54 42 3C 17 14 18 1B
E1 E0 09 0B 06 04 03 2B 43 42 3B 16 14 17 1B   (TFT_eSPI)
E1 F0 09 0B 06 04 03 2D 43 42 3B 16 14 17 1B   (Espressif)
F0 3C / F0 69                          lock
29 DISPON                              no INVON
```

### 3.2 lcdwiki MSP3525/3526, 3.5" IPS, ST7796U

STM32 `lcd.c` and the ESP32 TFT_eSPI `ST7796_Init.h` in `3.5inch_SPI_Module_ST7796_MSP3525_MSP3526_V1.0.zip`:

```
RST low 100ms, high, 50ms
11, 120ms
36 48 / 3A 55 / F0 C3 / F0 96
B4 02                                  2-dot inversion
B7 C6
C0 C0 00
C1 13                                  GVDD 4.5V (default)
C2 A7
C5 21                                  VCOM 1.125V
E8 40 8A 1B 1B 23 0A AC 33
E0 D2 05 08 06 05 02 2A 44 46 39 15 15 2D 32
E1 96 08 0C 09 09 25 2E 43 42 35 11 11 28 2E
F0 3C / F0 69, 120ms
21 INVON                               IPS panel
29 DISPON
```

### 3.3 lcdwiki MSP4020/4021, 4.0" TN, ST7796S

STM32 `lcd.c` in `4.0inch_SPI_Module_ST7796S_MSP4020&MSP4021_V1.0.zip`:

```
RST low 100ms, high, 50ms
F0 C3 / F0 96 / 36 68 / 3A 05 / B0 80 / B6 00 02 / B5 02 03 00 04 / B1 80 10
B4 00 (column inversion) / B7 C6 / C5 24 / E4 31
E8 40 8A 00 00 29 19 A5 33
C2, then A7 sent as a *command*  <- bug: A7 should be a data byte
E0 F0 09 13 12 12 2B 3C 44 4B 1B 18 17 1D 21
E1 F0 09 13 0C 0D 27 3B 44 4D 0B 17 17 1D 21
36 EC / F0 C3 / F0 69 <- lock typo: should be 3C then 69, so part I stays unlocked
13 / 11 / 29  <- DISPON right after SLPOUT, no 120 ms wait
then MADCTL from LCD_direction(), no INVON
```

Treat 3.3 as a data point. It has three bugs and should not be copied.

### 3.4 MADCTL table used by every vendor

lcdwiki `LCD_direction()` in both zips and TFT_eSPI `ST7796_Rotation.h` agree:

| Rotation | MADCTL | Bits |
|---|---|---|
| 0, portrait | 0x48 | MX, BGR |
| 1, landscape | 0x28 | MV, BGR |
| 2, portrait flipped | 0x88 | MY, BGR |
| 3, landscape flipped | 0xE8 | MY, MX, MV, BGR |

Two facts follow. These panels are BGR. And the unrotated, non-mirrored picture needs MX = 1, which means the glass's source S1 is on the far edge. A plain MADCTL 0x00 or 0x08 shows a mirror image.

---

## 4. mipidsi: what it does

Version checked: **mipidsi 0.10.0**, released 2026-02-17. It is the latest on crates.io, and `master`'s `st7796.rs` and `st7789.rs` are byte-identical to it. Downloaded from https://static.crates.io/crates/mipidsi/mipidsi-0.10.0.crate.

Dependencies: `embedded-hal = "1.0.0"`, `embedded-graphics-core = "0.4.1"`, which pairs with `embedded-graphics 0.8.x` (0.8.2 is current), and optional `heapless 0.8` for the default `batch` feature. MSRV is 1.75.

### 4.1 ST7796 model

`src/models/st7796.rs`:

- `ST7796`: `ColorFormat = Rgb565`, `FRAMEBUFFER_SIZE = (320, 480)`. It accepts `Serial4Line`, `Parallel8Bit`, and `Parallel16Bit` interfaces.
- `init` delegates entirely to `ST7789.init`.

### 4.2 Full init as sent over the wire

From `builder.rs` and `models/st7789.rs`:

```
RST low, 10 µs (Model::RESET_DURATION default), RST high     [or SWRESET 0x01 if no reset pin]
delay 150 ms
11 SLPOUT, delay 10 ms
36 MADCTL (from orientation + color_order + refresh_order)
21 or 20 (invert_colors: Inverted -> INVON, Normal -> INVOFF; default Normal)
3A 55, delay 10 ms
13 NORON, delay 10 ms
29 DISPON, delay 120 ms
```

### 4.3 Line-by-line against the datasheet and vendor init

| Item | mipidsi 0.10.0 | Datasheet / vendor | Matters? |
|---|---|---|---|
| Reset pulse | 10 µs low | Vendor holds 10 to 100 ms. No tRW in DS V1.0. | Low. 10 µs meets the Sitronix tRW. Override `RESET_DURATION` in a custom model to be safe. |
| Wait after reset | 150 ms | 120 ms vendor | OK |
| SLPOUT to next cmd | 10 ms | ≥5 ms [DS p161] | OK |
| SLPOUT to DISPON | about 30 ms | Vendors wait 120 ms | Low. Not a DS rule. |
| CSCON unlock | never | F0 C3, F0 96 | Needed for anything below |
| B4 DIC | default 01 | 01 ref, 02 IPS | Low |
| B6 DFC | default 80 02 3B | same | none |
| B7 EM | default 06 (EPF=00) | C6 | Small. Max red/blue 62/63. |
| C0 / C1 / C2 PWR | defaults (C1 = 4.5V GVDD, C2 unknown) | C1 06, C2 A7 (ref) | Yes. Contrast and drive strength. |
| C5 VCOM | default 1C (1.0V) | 18 (ref), 21 (IPS) | Yes. Wrong VCOM gives flicker, greyish blacks, and image retention. |
| E8 DOCA | default 40 8A 00 00 25 0A 38 33 | 40 8A 00 00 29 19 A5 33 | Moderate. Source EQ and gate timing. |
| E0 / E1 gamma | undefined, die default | tuned 14-byte curves | Yes. Biggest visible difference: tone curve, dark-level crush or washout. |
| Inversion | user option, default INVOFF | TN: off, IPS: on | Must be set per panel |
| Color order | user option, default RGB | BGR | Must set `ColorOrder::Bgr` |
| Orientation | Deg0 = MADCTL 0x00 | Vendor "rotation 0" = 0x48 | Default image is mirrored. Must set `mirrored: true`. |
| Offsets | (0, 0) | panel is full 320x480 | OK |
| Max size | 320x480 | 320x480 [DS p1] | OK |

Bottom line: `mipidsi`'s ST7796 init is enough to get a picture. The picture will not be what the panel maker tuned. For a mostly-black sign with saturated red dots, the gamma and VCOM differences show up as black level and how "lit" the dots look. Send the vendor registers (section 8).

### 4.4 Orientation mapping in mipidsi

`options/orientation.rs`, `MemoryMapping::from_orientation`, with `SetAddressMode` mapping reverse_rows to MY, reverse_columns to MX, and swap to MV:

| `Orientation` | MY MX MV | MADCTL with `Bgr` |
|---|---|---|
| Deg0, not mirrored | 0 0 0 | 0x08 (mirrored on these panels) |
| **Deg0, mirrored** | 0 1 0 | **0x48** = vendor rotation 0 |
| Deg90, not mirrored | 0 1 1 | 0x68 (mirrored on these panels) |
| **Deg90, mirrored** | 0 0 1 | **0x28** = vendor rotation 1, landscape |
| Deg270, not mirrored | 1 0 1 | 0xA8 (mirrored on these panels) |
| **Deg270, mirrored** | 1 1 1 | **0xE8** = vendor rotation 3, landscape flipped 180° |

So for typical ST7796 modules, neither `Rotation::Deg90` nor `Rotation::Deg270` alone is right. Both need `mirrored: true`. Which of the two depends on which way the board is mounted. Deg90+mirrored is lcdwiki's "rotate 90° clockwise" (USE_HORIZONTAL 1), and Deg270+mirrored is the same image turned 180°.

Build the orientation as a struct literal. The fields are public. The `flip_horizontal()` / `flip_vertical()` helpers swap meaning when the rotation is vertical, which is easy to get backwards. For example, `Orientation::new().rotate(Deg90).flip_horizontal()` yields Deg270+mirrored.

`display_size(320, 480)` takes the native portrait size. `mipidsi` swaps it for Deg90/Deg270, so the `DrawTarget` reports 480x320 (`options.rs`, `ModelOptions::display_size`).

### 4.5 SpiInterface and drawing path

`interface/spi.rs`: `SpiInterface::new(spi: impl SpiDevice, dc: impl OutputPin, buffer: &mut [u8])`.

- Commands are written as one `spi.write(&[cmd])` with DC low, then one `spi.write(args)` with DC high. Each `write` is its own `SpiDevice` transaction, so CS toggles between command and parameters. The ST7796 accepts that.
- Pixels: `send_pixels` copies `Rgb565` into `buffer` as big-endian bytes and issues one `spi.write` per full buffer. Bigger buffers mean fewer CS toggles and less per-transaction overhead. 4 to 8 KiB is a good size. With DMA, keep it at or under 32,736 bytes, the esp-hal max DMA transfer, and 4-byte aligned (section 6.3).
- `send_repeated_pixel` fills the buffer once and resends it. That makes `fill_solid` and `clear` cheap.
- `Display::set_pixels(sx, sy, ex, ey, iter)` sets CASET/RASET, sends RAMWR, then streams. `DrawTarget::fill_contiguous` calls it after clipping. With the default `batch` feature, `draw_iter` coalesces runs.
- `unsafe fn dcs(&mut self) -> &mut DI` exposes the interface for raw commands via `mipidsi::dcs::InterfaceExt::write_raw(cmd, &args)`.
- `Model` is a public, unsealed trait, so you can implement your own model with the vendor init and still use `Builder`, `Display`, and the `DrawTarget` impl.

---

## 5. ESP32 side

### 5.1 Which SPI peripheral

GPIO18 and GPIO23 are the ESP32's VSPI IO_MUX pins: GPIO18 = VSPICLK and GPIO23 = VSPID, function 1 (esp-metadata-generated 0.5.1, `_generated_esp32.rs`). In esp-hal, VSPI is `SPI3`. HSPI is `SPI2`.

esp-hal 1.2.2 picks the IO_MUX direct path automatically when the pin has a native function for the signal (`gpio/interconnect.rs`, `connect_peripheral_to_output`: it looks up `output_signals()` and only falls back to the GPIO matrix when none matches). So:

- `peripherals.SPI3` with GPIO18/GPIO23 bypasses the GPIO matrix. 80 MHz is electrically possible.
- `peripherals.SPI2` on the same pins goes through the GPIO matrix. Espressif limits that to about 40 MHz on the original ESP32. Use SPI3.

CS on GPIO5 can be a plain `Output` managed by `embedded-hal-bus`'s `ExclusiveDevice`. GPIO5 is also VSPICS0, but hardware CS is not needed. GPIO5 is a strapping pin that idles high, which is harmless for CS. GPIO27, 32, and 33 have no conflicts.

### 5.2 Clock options

esp-hal 1.2.2 `spi::master::Config::with_frequency` picks "the closest available frequency that does not exceed" the request. The SPI clock source is the 80 MHz APB clock, so the useful steps are 80, 40, 26.7, and 20 MHz. A request for 60 MHz gives 40 MHz.

### 5.3 Versions

| Crate | Latest stable | Notes |
|---|---|---|
| esp-hal | 1.2.2 | blocking `Spi` implements `embedded_hal::spi::SpiBus`, which is stable. DMA (`with_dma`, `SpiDma`) needs the `unstable` feature. |
| embedded-hal-bus | 0.3.0 | `ExclusiveDevice::new_no_delay(bus, cs) -> Result<_, CS::Error>` |
| mipidsi | 0.10.0 | |
| embedded-graphics | 0.8.2 | |

---

## 6. Performance

### 6.1 SPI frequency

| SCK | Bytes per full frame | Raw wire time | Max full-frame rate |
|---|---|---|---|
| 26.7 MHz | 307,200 | 92.2 ms | 10.8 fps |
| **40 MHz** | 307,200 | **61.4 ms** | 16.3 fps |
| 80 MHz | 307,200 | 30.7 ms | 32.6 fps |

The math is 480 x 320 x 2 bytes x 8 bits / f.

Recommendation: start at 40 MHz on SPI3. It is the community default, it is 2.6x the datasheet's conservative 15 MHz, and it is reliable with short wires. Then try 80 MHz, which the lcdwiki 3.5" ST7796 ESP32 demo uses out of the box, and keep it if a soak test with a full-screen gradient and moving pattern shows no corrupted pixels. Keep the SCK/MOSI leads under about 10 cm and share a solid ground. Never use 80 MHz for reads. TSCYCR is 150 ns, so reads need 6.67 MHz or less.

Real throughput with `mipidsi` and blocking non-DMA SPI is lower than wire time. The CPU converts each `Rgb565` into the buffer and then refills the 64-byte SPI FIFO, and none of that overlaps with transmission. Estimates, to be measured: about 70 to 85 ms per full frame at 40 MHz, and about 40 to 50 ms at 80 MHz. DMA at 80 MHz gets close to the 31 ms wire time.

### 6.2 Rendering approach for the LED dot matrix

A 307 KB framebuffer is out, and it is not needed. The ST7796's own GRAM holds the picture, so only changed regions need to be sent.

1. Keep the sign as a small logical LED grid, not pixels. At a 4 px pitch, a 3 px dot plus a 1 px gap, 480x320 is 120 x 80 LEDs. At 1 bit per LED that is 1.2 KB. Use 2 bits if you want lit, dim, and off. A 5 px pitch gives 96 x 64. Real BART signs use a coarse pitch, so 5 or 6 px probably reads better at arm's length.
2. Precompute one dot sprite, `pitch x pitch` RGB565 pixels with an anti-aliased round dot. Make two variants: lit, for example `Rgb565::new(31, 4, 0)` for an orange-red LED, and unlit, a very dark red such as `Rgb565::new(3, 0, 0)` so the off-LED grid stays faintly visible like the real sign. Store each sprite row as raw big-endian bytes.
3. Render one LED row at a time into a strip buffer of 480 x pitch pixels. At pitch 4 that is 3,840 bytes, at pitch 5 it is 4,800. Rendering is just copying `2*pitch` bytes per LED from the right sprite row, which costs a few µs per strip.
4. Push the strip to the window `(0, y) .. (479, y+pitch-1)`. With `mipidsi`, call `display.set_pixels(0, y, 479, y + p - 1, iter)` or `fill_contiguous`, where `iter` maps the strip bytes back to `Rgb565`. That keeps the generic API at the cost of one copy. A pitch-4 strip is 1.5 ms of wire time at 40 MHz.
5. Diff before you draw. Keep the previous LED grid and only push LED rows, or rectangles, that changed. A countdown that changes "5 min" to "4 min" touches a handful of rows, which takes about 10 ms at 40 MHz.
6. Marquee text: redraw only the text band. A 7-LED-tall font at pitch 4 is 28 px, or 26.9 KB, which is 5.4 ms per step at 40 MHz. Smooth 30+ fps scrolling of one line fits easily.
7. Hide the boot garbage. GRAM content is random at power-up. Keep BL low, run init, clear to black, then drive BL high.

The ST7796's hardware vertical scroll (VSCRDEF 0x33 / VSCSAD 0x37) runs along the native 480-line axis, which in landscape is horizontal. It scrolls the whole screen height as one band, so it cannot scroll a single text line on its own. Skip it.

The 9-pin header has no TE pin, so there is no tearing sync. Small dirty-region updates make tearing invisible in practice. A full-screen redraw will show a 60 ms wipe.

### 6.3 DMA

esp-hal 1.2.2 has no separate `SpiDmaBus` type anymore. `SpiDma` itself implements `embedded_hal::spi::SpiBus` and `embedded_hal_async::spi::SpiBus`, both `#[instability::unstable]` (`spi/master/dma.rs`). So `ExclusiveDevice<SpiDma<Blocking>, Output, NoDelay>` is a valid `SpiDevice` for `mipidsi`.

```rust
let (rx_buf, rx_desc, tx_buf, tx_desc) = esp_hal::dma_buffers!(8192); // RX unused but required
let spi = Spi::new(peripherals.SPI3, cfg)?
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23)
    .with_dma(peripherals.DMA_SPI3)
    .with_buffers(DmaRxBuf::new(rx_desc, rx_buf)?, DmaTxBuf::new(tx_desc, tx_buf)?);
```

Caveats from the esp-hal docs:

- The blocking `SpiBus::write` on `SpiDma` waits for completion. It removes the FIFO-refill gaps but does not let the CPU render while bytes go out.
- On ESP32, a source slice in internal SRAM that is not 4-byte aligned is copied whole into the DMA buffer. Align the `mipidsi` buffer with a `#[repr(align(4))]` wrapper.
- The max single DMA transfer is 32,736 bytes.

To overlap rendering and sending, skip `mipidsi` for the pixel stream. Use `SpiDma::write_buffer(len, DmaTxBuf)`, which returns a `SpiDmaTransfer` with `wait()`, and ping-pong two strip buffers. Or go async with `into_async()` and `write_async`. Do this only if measurements show the blocking path is too slow. For a mostly static sign it will not be.

### 6.4 Embassy

`mipidsi` is blocking. A full-screen redraw at 40 MHz blocks the executor thread for about 70 ms. esp-radio WiFi runs in its own esp-rtos threads and interrupts, so it keeps working, but other tasks on the same executor stall. Draw strip by strip with `embassy_futures::yield_now().await` between strips, or give the display its own executor or core.

---

## 7. Recommended mipidsi configuration

`Cargo.toml` for the firmware owner:

```toml
mipidsi = "0.10.0"
embedded-graphics = "0.8.2"
embedded-hal = "1.0"
embedded-hal-bus = "0.3"
esp-hal = { version = "1.2", features = ["esp32", "unstable"] } # "unstable" only for DMA
```

Builder, blocking SPI3 without DMA:

```rust
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    spi::{master::{Config, Spi}, Mode},
    time::Rate,
};
use mipidsi::{
    interface::SpiInterface,
    models::ST7796,
    options::{ColorInversion, ColorOrder, Orientation, Rotation},
    Builder,
};

let mut bl  = Output::new(peripherals.GPIO32, Level::Low,  OutputConfig::default()); // BL off until clear
let dc      = Output::new(peripherals.GPIO27, Level::Low,  OutputConfig::default());
let rst     = Output::new(peripherals.GPIO33, Level::High, OutputConfig::default());
let cs      = Output::new(peripherals.GPIO5,  Level::High, OutputConfig::default());

let spi_bus = Spi::new(
    peripherals.SPI3, // VSPI: GPIO18/23 use IO_MUX, not the GPIO matrix
    Config::default().with_frequency(Rate::from_mhz(40)).with_mode(Mode::_0),
)
.unwrap()
.with_sck(peripherals.GPIO18)
.with_mosi(peripherals.GPIO23);

let spi_dev = ExclusiveDevice::new_no_delay(spi_bus, cs).unwrap();

#[repr(align(4))]
struct Buf([u8; 8192]);
static mut DI_BUF: Buf = Buf([0; 8192]); // or a StaticCell
let di = SpiInterface::new(spi_dev, dc, unsafe { &mut (*core::ptr::addr_of_mut!(DI_BUF)).0 });

let mut delay = Delay::new();
let mut display = Builder::new(ST7796, di)       // or St7796Vendor, section 8.2
    .display_size(320, 480)                       // native portrait size; mipidsi swaps for 90/270
    .orientation(Orientation { rotation: Rotation::Deg90, mirrored: true }) // MADCTL 0x28 = landscape
    // .orientation(Orientation { rotation: Rotation::Deg270, mirrored: true }) // 0xE8: landscape turned 180°
    .color_order(ColorOrder::Bgr)
    .invert_colors(ColorInversion::Normal)        // TN glass. Use Inverted if white shows black (IPS)
    .reset_pin(rst)
    .init(&mut delay)
    .unwrap();

// display.clear(Rgb565::BLACK) via embedded_graphics::prelude::DrawTarget, then:
bl.set_high();
```

If the stock `ST7796` model is kept, patch in the vendor registers right after `init()` and before turning on the backlight:

```rust
use mipidsi::dcs::InterfaceExt;
let di = unsafe { display.dcs() };
for &(cmd, args, _) in VENDOR_REGS { di.write_raw(cmd, args).unwrap(); } // section 8.1 minus 11/36/3A/2x/29
```

This is safe. None of these registers is state `mipidsi` tracks. The cleaner route is the custom model in 8.2.

---

## 8. Recommended init

### 8.1 Sequence

This is the Sitronix reference set used by TFT_eSPI and Espressif's `esp_lcd_st7796`, with the ordering fixed and inversion left as a per-panel choice. Format is (cmd, data, delay after in ms).

```
HW reset: RST low 10 ms, RST high, wait 120 ms
(0x11, [],                                   120)  SLPOUT [DS p160-161]
(0xF0, [0xC3],                                 0)  CSCON unlock part I  [DS p239]
(0xF0, [0x96],                                 0)  CSCON unlock part II
(0x36, [0x28],                                 0)  MADCTL: MV|BGR, landscape 480x320. 0xE8 for 180°.
(0x3A, [0x55],                                 0)  COLMOD 16 bpp [DS p190]
(0xB4, [0x01],                                 0)  DIC 1-dot inversion [DS p216]
(0xB6, [0x80, 0x02, 0x3B],                     0)  DFC: SS=0, GS=0, 480 lines [DS p218]
(0xB7, [0xC6],                                 0)  Entry mode. 0x86 gives full-scale R/B [DS p222]
(0xE8, [0x40,0x8A,0x00,0x00,0x29,0x19,0xA5,0x33], 0)  DOCA [DS p237-238]
(0xC1, [0x06],                                 0)  PWR2 GVDD 3.85V [DS p224]
(0xC2, [0xA7],                                 0)  PWR3 drive current [DS p227]
(0xC5, [0x18],                                 0)  VCOM 0.9V [DS p227-228]
(0xE0, [0xF0,0x09,0x0B,0x06,0x04,0x15,0x2F,0x54,0x42,0x3C,0x17,0x14,0x18,0x1B], 0)  PGC [DS p233-234]
(0xE1, [0xE0,0x09,0x0B,0x06,0x04,0x03,0x2B,0x43,0x42,0x3B,0x16,0x14,0x17,0x1B], 0)  NGC [DS p235]
(0xF0, [0x3C],                                 0)  CSCON lock part I
(0xF0, [0x69],                                 0)  CSCON lock part II
(0x20, [],                                     0)  INVOFF for TN. Use 0x21 INVON for IPS [DS p165-166].
(0x13, [],                                     0)  NORON
-- clear GRAM to black with CASET/RASET/RAMWR --
(0x29, [],                                    20)  DISPON
then BL high
```

If the panel turns out to be IPS (white shows black with INVOFF, and viewing angles are wide), switch to the lcdwiki MSP3525 values from 3.2 for B4, C0, C1, C5, E8, E0, and E1, with INVON.

Do not send SWRESET right after a HW reset. It is redundant, and it adds the 120 ms sleep-in rule [DS p142].

### 8.2 As a mipidsi custom model

```rust
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;
use mipidsi::{
    dcs::{InterfaceExt, SetAddressMode},
    interface::Interface,
    models::{Model, ModelInitError},
    options::{ColorInversion, ModelOptions},
};

pub struct St7796Vendor;

const VENDOR: &[(u8, &[u8])] = &[
    (0xF0, &[0xC3]), (0xF0, &[0x96]),
    (0xB4, &[0x01]),
    (0xB6, &[0x80, 0x02, 0x3B]),
    (0xB7, &[0xC6]),
    (0xE8, &[0x40, 0x8A, 0x00, 0x00, 0x29, 0x19, 0xA5, 0x33]),
    (0xC1, &[0x06]), (0xC2, &[0xA7]), (0xC5, &[0x18]),
    (0xE0, &[0xF0,0x09,0x0B,0x06,0x04,0x15,0x2F,0x54,0x42,0x3C,0x17,0x14,0x18,0x1B]),
    (0xE1, &[0xE0,0x09,0x0B,0x06,0x04,0x03,0x2B,0x43,0x42,0x3B,0x16,0x14,0x17,0x1B]),
    (0xF0, &[0x3C]), (0xF0, &[0x69]),
];

impl Model for St7796Vendor {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 480);
    const RESET_DURATION: u32 = 10_000; // µs; Builder pulses RST low this long

    fn init<DELAY: DelayNs, DI: Interface>(
        &mut self, di: &mut DI, delay: &mut DELAY, options: &ModelOptions,
    ) -> Result<SetAddressMode, ModelInitError<DI::Error>> {
        let madctl = SetAddressMode::from(options);
        delay.delay_ms(120);                 // after RST high
        di.write_raw(0x11, &[])?;            // SLPOUT
        delay.delay_ms(120);
        di.write_command(madctl)?;           // 0x36
        di.write_raw(0x3A, &[0x55])?;
        for &(cmd, args) in VENDOR { di.write_raw(cmd, args)?; }
        di.write_raw(match options.invert_colors {
            ColorInversion::Normal => 0x20,
            ColorInversion::Inverted => 0x21,
        }, &[])?;
        di.write_raw(0x13, &[])?;            // NORON
        di.write_raw(0x29, &[])?;            // DISPON (keep BL off until first clear)
        delay.delay_ms(20);
        Ok(madctl)
    }
}
```

Then use `Builder::new(St7796Vendor, di)` with the same options as section 7. I have not compiled this snippet. The trait signature, `SetAddressMode: From<&ModelOptions>`, `InterfaceExt::write_raw`, `ModelInitError: From<DI::Error>`, and `ModelOptions.invert_colors` all match the 0.10.0 source.

---

## 9. Bring-up checklist

### 9.1 Getting the picture right

1. Jumper set to SPI (IM 111). VCC on 5V if available. MISO left open.
2. Fill solid red. If it shows blue, swap `ColorOrder`.
3. Fill white. If it shows black, toggle `invert_colors`, which also means the glass is IPS (see 8.1).
4. Draw a 1 px rectangle at (0,0)-(479,319) and the text "F 123" at top-left. Missing edges mean the size or orientation is wrong. Mirrored text means toggling `mirrored`. Upside-down text means switching Deg90 and Deg270.
5. Draw a 32-step red ramp, `Rgb565::new(i, 0, 0)` for i in 0..32. Compare the stock `mipidsi` init against the vendor init. Look for crushed low steps, a lifted black, or banding.
6. Soak at the target clock. Run a moving pattern for a few minutes at 80 MHz. Any speckle or shifted rows means drop to 40.

### 9.2 TN viewing angle

TN glass has a preferred viewing direction. The QD3579 TN panel is "12 o'clock". In landscape that axis becomes left/right. If the colors look off from the viewing position, try the other landscape orientation (Deg90 vs Deg270) and mount the board accordingly.

### 9.3 Backlight

Drive GPIO32 with LEDC PWM at a few kHz for night dimming. Keep it low through init and the first clear.

### 9.4 Optional ID read

To confirm ST7796 for certain, temporarily wire SDA-O to GPIO19, the VSPIQ IO_MUX pin. At 6 MHz or lower, send 0xD3 and read 4 bytes. Expect `xx 00 77 96` [DS p233]. Not needed for operation.

---

## 10. Files and sources

Datasheets:

- Sitronix ST7796S V1.0: https://www.displayfuture.com/Display/datasheet/controller/ST7796s.pdf, with the same document at https://www.lcdwiki.com/res/MSP3525_MSP3526/ST7796S-Sitronix.pdf and https://dl.espressif.com/dl/schematics/ST7796S_SPEC_V1.0.pdf
- QD3525, the 3.5" IPS panel used on MSP3525: https://www.lcdwiki.com/res/MSP3525_MSP3526/QD3525-Specifications-A.pdf

Web:

- lcdwiki 3.5" IPS ST7796 (MSP3525/3526): https://www.lcdwiki.com/3.5inch_IPS_SPI_Module_ST7796
- lcdwiki 4.0" ST7796S (MSP4020/4021): https://www.lcdwiki.com/4.0inch_SPI_Module_ST7796
- lcdwiki 3.5" ESP32-32E (ST7796U TN): https://www.lcdwiki.com/3.5inch_ESP32-32E_Display
- MSP3525 demo zip: https://www.lcdwiki.com/res/Program/CTP_SPI/3.5inch/SPI_ST7796_MSP3525_MSP3526_V1.0/3.5inch_SPI_Module_ST7796_MSP3525_MSP3526_V1.0.zip
- MSP4020 demo zip: https://www.lcdwiki.com/res/Program/Common_SPI/4.0inch/SPI_ST7796S_MSP4020_4021_V1.0/4.0inch_SPI_Module_ST7796S_MSP4020%26MSP4021_V1.0.zip
- TFT_eSPI ST7796 driver: https://github.com/Bodmer/TFT_eSPI/tree/master/TFT_Drivers
- Espressif esp_lcd_st7796: https://github.com/espressif/esp-bsp/tree/master/components/lcd/esp_lcd_st7796
- mipidsi: https://github.com/almindor/mipidsi and https://crates.io/crates/mipidsi
- esp-hal 1.2.2: https://crates.io/crates/esp-hal
- Amazon listing: https://www.amazon.com/dp/B0DFWL9M9B
