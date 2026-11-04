# BART sign: parts sourcing for a 10 to 50 unit gift batch

All prices are USD and were seen on 2026-11-02 unless a row says otherwise. Every price has a link. "Not confirmed" means I could not read a real number, and I did not fill one in.

Scope: the display (3.5" 320x480 ST7796 over SPI) and the MCU. The harness or carrier PCB is covered elsewhere. Power is in `power.md` next to this file.

## Short answer

- The display is where the quality upgrade lives. The current XIITIA board is a generic Shenzhen design with TN glass. IPS modules with the same controller family (ST7796U), the same 48.96 x 73.44 mm active area and the same SPI command set cost $6 to $9 on AliExpress and $17 to $19 from branded resellers. Firmware changes by two lines. Turn inversion on, and recheck BGR and rotation on the first unit.
- The MCU is not where the money is. The ELEGOO board you have costs $6.66 per board on Amazon. AliExpress clones of the same board are $5.32, and Alibaba lists them at $2.50 to $3.30, but those carry real risks (fake CP2102, relabelled modules, USB-C that won't power from a C-to-C cable). Saving $1 to $4 a unit on a hidden part of a gift is not worth that. Keep the ESP32 and buy a known board.
- At 10 to 50 units, an IPS module plus a known ESP32 board lands at about the same cost as today's TN parts, roughly $15 to $19 a unit, for a clearly better picture. Real savings, under $11 in parts, only show up with a custom carrier PCB using a bare ESP32-WROOM-32E module and a bare IPS panel. That pays off at about 50 units and up.
- The all-in-one boards split into two camps. The cheap ones that keep your exact driver, Sunton ESP32-3248S035 and lcdwiki E32N35T, are TN, so they don't fix the main weakness. The IPS ones either cost $26 to $33 (Waveshare ESP32-S3-Touch-LCD-3.5, still ST7796 over SPI) or use a different controller over QSPI (Guition JC3248W535, AXS15231B), which means a driver rewrite and some batch-to-batch risk.

## Landed cost and tariffs

The US ended duty-free de minimis entry for China in May 2025 and for all countries in August 2025 ([California Insider](https://californiainsider.com/news/explainer-what-the-end-of-the-de-minimis-duty-free-parcel-rule-means-5908084)). The Supreme Court struck down the IEEPA tariffs on 2026-02-20, and the administration replaced them with a temporary Section 122 duty ([Thomson Reuters](https://www.thomsonreuters.com/en-us/help/onesource-global-trade/regulatory-insights/2026/february-20th/supreme-court-rejects-ieepa-as-a-tariff-statute.html), [Cozen O'Connor](https://www.cozen.com/news-resources/publications/2026/supreme-court-rules-president-lacks-ieepa-tariff-authority-president-imposes-section-122-tariffs)). Section 301 duties on Chinese goods still apply. I could not find a reliable source for the rate in force in November 2026.

So the landed figures below use an allowance of 30% of item value on anything shipped from China. That is an assumption, not a quote. AliExpress usually shows the import charge as its own line at checkout for US addresses, and that line is the real number. Amazon prices already include whatever duty the seller paid.

AliExpress prices seen on 2026-11-02 are 11.11 sale prices that end on 2026-11-18. Every AliExpress item page also shows a "new shopper" price of about $1.09, limited to one unit. I ignored those.

## A. The exact display module

### What it is

The XIITIA board is the 3.5" variant of a generic family sold as "2.4/2.8/3.5 Inch 240x320 320x480 SPI TFT Serial Port Module 5V/3.3V PCB Adapter ST7789V/ST7796S". Listing photos match every detail you gave. They show the 9-pin header with SDA-O, the "8080-16P" and "8080-8P" FPC sockets, the IM0/IM1/IM2 jumper table and the "3.5"TFT" silkscreen. Amazon's spec text is the Chinese stock copy, word for word. The Amazon page names no model number. Brand and manufacturer both read XIITIA, UPC 687117760173. Search for the family title above, not for "ST7796 8080", or you will miss most listings.

### Listings

AliExpress, US ship-to, read in a browser:

| Store | Listing | Real price for the 3.5" | Store signals | Match |
|---|---|---|---|---|
| SONGXIN LIGHT Store | [3256812768161833](https://www.aliexpress.us/item/3256812768161833.html) | 1 pc $8.07; 5 pcs $40.20, $8.04 each; 10 pcs $79.18, $7.92 each. Free shipping. | 97.9% positive, 12.1K followers, 6 sold | High, same photos |
| TENSTAR ROBOT Store | [3256805988905985](https://www.aliexpress.us/item/3256805988905985.html) | Not confirmed. Only the $1.09 new-shopper price and a $19.62 strikethrough showed. | 98.4% positive, 4.9 stars from 510 reviews, 5,000+ sold | High, main photo shows every feature |
| EGBO Store | [3256806904995020](https://www.aliexpress.us/item/3256806904995020.html) | Not confirmed. $2.05 welcome deal, $15.76 strikethrough, "3.5 inch not touch" variant. | 97.5% positive, 11.1K followers | High for "not touch". The "with touch" variant is a different board. |
| Estardyn Flagship Store | [3256809914893148](https://www.aliexpress.us/item/3256809914893148.html) | Not confirmed. $1.41 promo, $30.24 strikethrough. | 96.8% positive, 500+ sold | Medium-high |

Only SONGXIN showed a normal repeat-buyer price, about $8 a unit. No AliExpress listing shows tiers past 10.

Alibaba, search-result cards only. Product pages put up a slider CAPTCHA, which I did not attempt. The price ranges cover all of the 2.4/2.8/3.5 variants in each listing, so the 3.5" price, the 10/50/100/500 tiers, lead time and US shipping are all not confirmed.

| Supplier | Listing | Years, rating | Range across variants | MOQ |
|---|---|---|---|---|
| Shenzhen Chuangheying Electronic Technology | [1601988223641](https://www.alibaba.com/product-detail/_1601988223641.html) | 5 yrs, 4.8 from 345 | $0.99-3.07 | 1 |
| HELLOCHIP LIMITED, HK | [1601627808894](https://www.alibaba.com/product-detail/_1601627808894.html) | 2 yrs, 4.7 from 637 | $2-5 | 1 |
| HELLOCHIP LIMITED, HK | [1601616388849](https://www.alibaba.com/product-detail/_1601616388849.html) | 2 yrs, 4.7 from 637 | $2.80-5.80 | 1 |
| Shenzhen Shengxing Micro Technology | [1601995492227](https://www.alibaba.com/product-detail/_1601995492227.html) | 2 yrs, 4.8 from 5 | $2.80-5.80 | 1 |
| Shenzhen Zhiyuan Shengye Technology | [1601784460683](https://www.alibaba.com/product-detail/_1601784460683.html) | 1 yr, 4.8 from 70 | $0.99-3.07 | 1 |
| Shenzhen Riemann Technology | [1601890632178](https://www.alibaba.com/product-detail/_1601890632178.html) | 1 yr, 4.6 from 15 | $3-10 | 1 |

Two more suppliers, Beijing Mengxing (7 yrs) and Shenzhen Kwm (6 yrs), list $0.72-0.90 at MOQ 5. That range almost certainly covers only the 2.4" variant. The low end of every range above is probably the 2.4" too. To get real 3.5" tiers you have to log in and send an RFQ, which I did not do.

LCSC has no ST7796 module or panel in its catalog. A search for "ST7796" returns zero results. made-in-china has bare panels but not this board.

### The glass

I could not identify the panel part number. No listing names the 40-pin glass on this board. Read the code printed on the panel's FPC tail on one of your units, under the backlight near the connector.

The 3.5" 320x480 40-pin 0.5 mm plug-in glass is a de-facto standard shared by ST7796 and ILI9488 panels. A pin table from [this listing](https://www.aliexpress.us/item/3256805489615982.html) gives 5 GND, 6 VDDI, 7 VDD, 8 TE, 9 CSX, 10 DCX, 11 WRX/SCL, 12 RDX, 13 SDI, 14 SDO, 15 RESET, 17-32 DB0-DB15, 33 LED-A, 34-36 LED-K, 38-40 IM0-IM2. Its IM codes match your board's jumper table bit for bit, so your board very likely uses this pinout. That is an inference, not verified.

FPC codes seen on compatible panels, none of them indexed anywhere I could find: "CL35BC1017-40A" (HongXunScreen) and "HC35001V0" (Shenzhen Duoweisi, best reading of a photo). Vendor model numbers in the same class include Shenzhen Liyuan Huida TFT035F001, $7.40 at MOQ 2 ([Alibaba](https://www.alibaba.com/product-detail/3-5-inch-3-5-LCD_1600465741996.html)), and Genyu GY-TFT035F056, ST7796U, 40-pin ([frbiz](https://genyudisplay.company.frbiz.com/p119399272-3-5-inch-tft-lcd-module-320x480-st7796-40pin.html)).

### IPS glass with the same 40-pin pinout

This is a cheap experiment worth one sample order. If the glass on your board sits in a ZIF socket rather than being soldered, an IPS panel with the same pinout may drop straight in.

| Store | Listing | Prices | Notes |
|---|---|---|---|
| HongXunScreen Store | [3256807061236818](https://www.aliexpress.us/item/3256807061236818.html) | ST7796 TN $6.65, **ST7796 IPS $6.95**, IPS + resistive $7.99, IPS + capacitive $9.65. Extra 3% off at 50+. Free shipping. | Drawing shows the 48.96 x 73.44 mm active area and a 40-pin 0.5 mm plug-in. 100% positive, 229 sold. |
| Shenzhen Duoweisi | [3256804475873106](https://www.aliexpress.us/item/3256804475873106.html) | TFT $6.48, IPS $7.18, IPS + cap touch $10.96. Extra 3% off at 50+. | Store open since 2018, 96.4% positive. Pin count not confirmed. |

Avoid panels sold as "SPI+RGB 40pin", such as Leshida R035CM02-A ([made-in-china](https://lestarlcd.en.made-in-china.com/product/GpYRLljxVVku/)). Those use an RGB-interface pinout, not the MCU/8080 one.

## B. Better display options

### What changes in firmware

`firmware/src/display.rs` already uses `mipidsi` with a vendor init and `ColorInversion::Normal`, commented "TN glass". For an IPS ST7796U panel:

- Set `.invert_colors(ColorInversion::Inverted)`. lcdwiki's MSP3525 demos send 0x21 (INVON) on both STM32 and ESP32. See `display-st7796.md`.
- Recheck BGR and rotation on the first unit with `display::test_pattern`. Different glass can flip either.
- Gamma tables tuned for TN may look slightly off on IPS. lcdwiki's MSP3525 init is the reference to copy if it does.

Everything else, SPI mode, clock, DMA and command set, stays the same.

### Modules with a 2.54 mm header

| Module | Driver | Panel | Brightness, contrast, angle | Header and power | Size | Price | Source |
|---|---|---|---|---|---|---|---|
| Current XIITIA board | ST7796S | TN | Not published. Off-axis blacks go navy. | 9-pin, 3-5 V | 59.19 x 94.84 mm | $23.99 / 2 = $12.00 | [Amazon B0DFWL9M9B](https://www.amazon.com/dp/B0DFWL9M9B) |
| **lcdwiki MSP3525** (no touch) / MSP3526 (cap touch) | ST7796U, BOE glass, panel QD3525 | IPS, "all o'clock" | 300 cd/m². Contrast is only in an image table. | 14-pin: VCC, GND, CS, RST, RS, MOSI, SCK, LED, MISO, then touch and SD pins. 3.3-5 V with level shifting. Backlight 6 LEDs, 95 mA. | 55.5 x 98 mm, 12.98 mm thick (no touch) | No lcdwiki-branded store price found | [lcdwiki](https://www.lcdwiki.com/3.5inch_IPS_SPI_Module_ST7796), [panel spec](https://www.lcdwiki.com/res/MSP3525_MSP3526/QD3525-Specifications-A.pdf) |
| Hosyond B0CMD7Y55M, the MSP3526 on Amazon | ST7796U | IPS | 300 cd/m² | Same 14-pin, touch pins unused | Same | **$18.99** | [Amazon](https://www.amazon.com/dp/B0CMD7Y55M) |
| Elecrow DLS30035B, an MSP3526 rebadge | ST7796U | IPS | 300 cd/m² | Same 14-pin + FPC, FT6336U cap touch | 55.5 x 98 x 14.28 mm | **$16.90**, no qty tiers, "custom by MOQ" | [Elecrow](https://www.elecrow.com/3-5-ips-spi-lcd-capacitive-touch-module-st7796-driver-320-480-resolution.html), [wiki](https://elecrow.com/wiki/3.5-inch_IPS_SPI_LCD_Capacitive_Touch_Display_Module_With_ST7796_Driver-320x480_Resolution_Arduino_Compatible.html) |
| Waveshare 3.5inch Capacitive Touch LCD, SKU 29318 | ST7796S | IPS | Not published | No 2.54 header. GH1.25 15-pin locking connector + 18-pin FPC. 3.3/5 V, about 63 mA at 3.3 V. | 61.00 x 92.44 mm | $18.39 at 1-9, **$18.09 at 10-49, $17.97 at 50-99** | [Waveshare](https://www.waveshare.com/3.5inch-capacitive-touch-lcd.htm), [wiki](https://www.waveshare.com/wiki/3.5inch_Capacitive_Touch_LCD) |
| Adafruit 2050 | HX8357D, not ST7796 | Not stated | Not quantified | SPI or 8-bit, 3-5 V, constant-current backlight boost | 56 x 97 mm | $39.95 / $35.96 at 10-99 / $31.96 at 100+ | [Adafruit](https://www.adafruit.com/product/2050) |

Notes:

- The MSP3520/MSP3521 are ILI9488 modules, not ST7796 ([lcdwiki](https://www.lcdwiki.com/3.5inch_SPI_Module_ILI9488_SKU:MSP3520)). ILI9488 over SPI only takes 18-bit color, so every frame costs 50% more SPI traffic. Skip them.
- The Waveshare GH1.25 connector is worth a look for the harness work. It is a keyed, locking cable, which is what you want instead of jumpers.
- Adafruit is the best-documented option here, but a different controller at three times the price makes no sense for this build.

AliExpress generic IPS ST7796 modules. None showed quantity tiers or a contrast figure. Check the header layout on arrival, because these may be the 14-pin MSP3525 layout or something else.

| Store | Listing | IPS, no touch | IPS, cap touch | Notes |
|---|---|---|---|---|
| SZFYDOSH Technology | [3256805628710747](https://www.aliexpress.us/item/3256805628710747.html) | **$6.11** | $10.28 | Title says ST7796U. Shipping not confirmed. |
| iPistBit Official Store | [3256805809616969](https://www.aliexpress.us/item/3256805809616969.html) | **$8.85** | $11.29 | Says ST7796U, FT6336U, 178°. That text is flagged AI-generated on the page, so treat it as unverified. Standard shipping $7.21. |
| Funkeydo | [3256805925926605](https://www.aliexpress.us/item/3256805925926605.html) | $11.62 | $15.09 | Looks like an MSP3526 clone |
| YiXin | [3256812146061575](https://www.aliexpress.us/item/3256812146061575.html) | $16.33 | $21.33 | |

### Bare IPS panels for a custom carrier PCB

| Part | Driver | Panel | Brightness, contrast, angle | FPC | Backlight | Price | Source |
|---|---|---|---|---|---|---|---|
| HongXunScreen ST7796 IPS | ST7796 | IPS | Not published | 40-pin 0.5 mm, the de-facto pinout | Not stated | $6.95, 3% off at 50+ | [AliExpress](https://www.aliexpress.us/item/3256807061236818.html) |
| QD3525, the glass inside MSP3525 | ST7796U | IPS | 300 cd/m² | 48-pin. IM2-0 on pins 39-41. | 6 LEDs in parallel, 90 mA typ | No source to buy it bare | [spec](https://www.lcdwiki.com/res/MSP3525_MSP3526/QD3525-Specifications-A.pdf) |
| DWIN LI48320T035IB3098 | ST7796-G4 | IPS | 300 nit. A 700:1 contrast figure appears only on an Alibaba listing. | 40-pin 0.5 mm. One listing says RGB 24-bit, so SPI is not confirmed. | Not confirmed | $10.51 bare, Express shipping $11.63 | [AliExpress](https://www.aliexpress.us/item/3256804947776961.html) |
| Focus LCDs E35GH-I-RW300-N | ST7796U | IPS | 300 nits on the product page, 350 in a mirrored PDF | 40-pin 0.5 mm, mates with Hirose FH12A-40S-0.5SH. MCU or 3/4-wire SPI. Pin numbering differs from the de-facto pinout. | 6 LEDs, 120 mA | $40.35. DigiKey AT €32.50 at 50. | [Focus LCDs](https://focuslcds.com/product/e35gh-i-rw300-n/), [datasheet](https://static.dajiqun.com/product-medias//focus-lcds/E35GH-I-RW300-N/22158623-21351297.pdf) |
| DisplayModule DM-TFT35-431 | ST7796S | IPS | **550 min, 600 typ cd/m²; 800-1000:1; 70-80° each way** | 50-pin, MCU, SPI or RGB | 8 LEDs, 160 mA typ | $39.90; 5% off at 10-49, 10% off at 50-199 | [DisplayModule](https://displaymodule.com/products/3-5-inch-ips-display-320x480-touch-with-spi-mcu-rgb), [datasheet](https://cdn.shopify.com/s/files/1/0264/7629/files/DM-TFT35-431_Datasheet.pdf) |
| Orient AFK320480A0-3.5N12NTM | ST7796S | TN | 360 typ; CR 500; 70° H / 60° V | About 45-pin, SPI + RGB | 6 LEDs in series at about 18 V, needs a boost driver | $16.33; $15.54 at 50+ | [Orient](https://www.orientdisplay.com/store/3-5-tn-display-320x480-360-nits-spirgb-interface) |

The DM-TFT35-431 is the only ST7796 part I found with a published brightness well above 300 nits. At $36 in volume it costs four times a whole IPS module. For an indoor desk sign, 300 nits is plenty.

Dead ends: Buydisplay's ER-TFT035IPS-6 is ILI9488, not ST7796. Newhaven has no 320x480 ST7796 part. LCSC stocks no ST7796 3.5" panel. Its HS35B01A ([C19632787](https://www.lcsc.com/product-detail/C19632787.html)) is ILI9488.

## C. ESP32 boards

### What the firmware is today

`firmware/Cargo.toml` builds for the original ESP32 with esp-hal 1.2, esp-radio Wi-Fi and mbedtls-rs TLS. It renders in strips, so it needs no framebuffer and no PSRAM. Any ESP32 or ESP32-S3 board works. A C6 might, but TLS plus Wi-Fi in 512 KB is untested here.

### Dev boards

| Board | Price | Quality notes | Source |
|---|---|---|---|
| ELEGOO ESP-WROOM-32, USB-C, CP2102, 30-pin, 3-pack | $19.99 / 3 = **$6.66** | Works in the prototype. 4.7 stars from 555 ratings. Module revision not stated. Ships from the US, no duty. | [Amazon B0D8T53CQ5](https://www.amazon.com/dp/B0D8T53CQ5) |
| AliExpress clone, SAMIORE Store, "CP2102-30Pin Type-C" | $5.32, 1% off at 50+, shipping $4.01 | 5,000+ sold, 4.8 from 63 reviews | [AliExpress](https://www.aliexpress.us/item/3256807630806447.html) |
| Alibaba clones, 30-pin USB-C CP2102 | $2.48-3.29 depending on seller, MOQ 1 to 50. Tiers not confirmed. | Hapchuang, ZYSY, Yonglisheng (4.7 from 298), Sinapus | [Hapchuang](https://www.alibaba.com/product-detail/ESP32-ESP32S-NEW-Module-Development-Board_1601574201046.html), [ZYSY](https://www.alibaba.com/product-detail/ZYSY-Type-C-ESP32-CP2102-30Pin_1601822939077.html), [Yonglisheng](https://www.alibaba.com/product-detail/Stock-ESP32-Development-Board-WIFI-Module_1601404242294.html), [Sinapus](https://www.alibaba.com/product-detail/ESP32-Type-C-Development-Board-30_11000034446356.html) |
| Espressif ESP32-DevKitC-32E, genuine | $14.00 on Espressif's AliExpress store plus $5.93 shipping. Schukat $9.11 at 10/100. DigiKey $10.00 at 1 from a search snippet, not confirmed. | Real WROOM-32E (silicon v3). Wider than the 30-pin boards. | [Espressif AliExpress](https://www.aliexpress.us/item/3256804255226715.html), [Octopart](https://octopart.com/part/espressif-systems/ESP32-DEVKITC-32E) |
| Espressif ESP32-S3-DevKitC-1 N8R8 | $22.00 regular on Espressif's AliExpress store. DigiKey and Mouser $15.00 at 1 from snippets, not confirmed. LCSC $14.44 at 200, out of stock. | Two USB ports, native USB | [AliExpress](https://www.aliexpress.com/item/1005003979778978.html), [LCSC](https://www.lcsc.com/product-detail/C20541519.html) |
| **Seeed XIAO ESP32-S3**, 8 MB flash, 8 MB PSRAM | **$7.49**, same at 10+. Bulk needs a quote. | 21 x 17.5 mm, native USB-C, well-made, consistent revisions | [Seeed](https://www.seeedstudio.com/XIAO-ESP32S3-p-5627.html) |
| Seeed XIAO ESP32-C6 | $5.20, same at 10+ | RISC-V, stable Rust, no PSRAM, 512 KB SRAM | [Seeed](https://www.seeedstudio.com/Seeed-Studio-XIAO-ESP32C6-p-5884.html) |
| WeAct Studio ESP32-S3 N16R8 | $7.18, free shipping | WeAct official store, 5.0 from 43 reviews | [AliExpress](https://www.aliexpress.us/item/3256805406415437.html) |
| LOLIN S3, 16 MB / 8 MB | $9.33 at 1; $8.95 each for 10; $8.08 each for 20. Free shipping. | LOLIN official store, 4.8 from 29 reviews | [AliExpress](https://www.aliexpress.us/item/3256804457160611.html) |

What goes wrong with cheap 30-pin clones:

- USB-C with no 5.1k pull-downs on CC won't power from a C-to-C cable. A-to-C still works. Espressif staff confirm the mechanism on [esp32.com](https://esp32.com/viewtopic.php?p=129561). A gift recipient will try their phone charger and a C-to-C cable first, so test this on every board type before you order a batch, including the ELEGOO.
- Fake CP2102 chips drop characters on some hosts ([kernel patch discussion](https://patchwork.kernel.org/comment/24466857)). That only matters for flashing, which you do, not the recipient.
- "ESP32-S" and unbranded cans show up at the lowest prices. No clone listing I found claimed a WROOM-32E.
- The AMS1117 regulator on these boards drops about 1.1 V. Wi-Fi TX bursts plus a display backlight can brown it out. That's a general engineering concern, not a failure I found reported on these specific boards.

### Bare modules for a custom PCB

LCSC live prices. LCSC's tiers are 1/10/30/100/650, with no 50 break. All are JLCPCB Extended parts, so each carries a feeder loading fee, $1.53 per part type on Standard PCBA ([JLCPCB](https://jlcpcb.com/help/article/pcb-assembly-price)).

| Part | LCSC | @10 | @30 | @100 | Stock |
|---|---|---|---|---|---|
| ESP32-WROOM-32E-N4 | [C701341](https://lcsc.com/product-detail/WiFi-Modules_Espressif-Systems-Espressif-Systems-ESP32-WROOM-32E-N4_C701341.html) | $3.31 | $3.07 | **$2.73** | 28,574 |
| ESP32-WROOM-32E-N8 | [C701342](https://lcsc.com/product-detail/WiFi-Modules_Espressif-Systems-Espressif-Systems-ESP32-WROOM-32E-N8_C701342.html) | $4.08 | $3.81 | $3.44 | 10,405 |
| ESP32-S3-WROOM-1-N8R8 | [C2913201](https://lcsc.com/product-detail/WiFi-Modules_Espressif-Systems-Espressif-Systems-ESP32-S3-WROOM-1-N8R8_C2913201.html) | $4.46 | $4.14 | $3.70 | 9,173 |
| ESP32-S3-WROOM-1-N16R8 | [C2913202](https://www.lcsc.com/product-detail/C2913202.html) | $4.75 | $4.24 | $3.79 | 28,029 |
| ESP32-C6-MINI-1-N4 | [C5736265](https://lcsc.com/product-detail/WiFi-Modules_Espressif-Systems-Espressif-Systems-ESP32-C6-MINI-1-N4_C5736265.html) | $3.63 | $3.37 | $3.02 | 1,414 |

### ESP32 or ESP32-S3

Stay on the ESP32 for the first batch. The firmware works, and nothing in it needs more RAM.

Move to the S3 if you design a carrier PCB anyway. Its native USB-Serial-JTAG removes the CP2102 and its driver, its fakes and its power draw. The `power.md` work also wants the CP2102 and AMS1117 gone. The cost of moving:

- Change the chip feature in every esp-* crate from `esp32` to `esp32s3`.
- Re-pick pins. The S3 has no fixed VSPI pins, since everything routes through the GPIO matrix. GPIO26-32 are flash, and on R8 parts GPIO33-37 are octal PSRAM.
- SPI3 and its DMA channel exist on both chips, but the S3 uses GDMA, so the DMA setup lines change.
- mipidsi, sign-core and the renderer don't change.

WROOM-32 is NRND per Espressif's [datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32_datasheet_en.pdf). Use WROOM-32E on any custom board. It is pin-compatible.

## D. All-in-one boards

| Board | MCU | Panel | Display bus and pins | Backlight | Touch, USB | Price | Firmware change |
|---|---|---|---|---|---|---|---|
| Sunton ESP32-3248S035C / R | ESP32-WROOM-32, 4 MB, no PSRAM | **TN**, about 60° | ST7796 over SPI. SCLK 14, MOSI 13, MISO 12, CS 15, DC 2, no RST. | GPIO27, PWM | C: GT911 on I2C 33/32. R: XPT2046. CH340C. | AliExpress qty 1 about $19-29 from search snippets, $20.94 each at 3+. Alibaba DIYmalls €15.49-17.27 at MOQ 30, snippet only. Tinytronics €18.75. | Almost none. Use SPI2 (HSPI) and DMA_SPI2, remap pins, no reset pin. |
| lcdwiki / Hosyond E32N35T (no touch) / E32R35T (resistive) | ESP32-WROOM-32E, 4 MB | **TN** | ST7796U over SPI, same pins as Sunton. RST tied to EN. | GPIO27, active high | CH340C, USB-C, LiPo charger | Hosyond Amazon $18.99 from a price tracker, live price not confirmed | Almost none, same as Sunton |
| **Waveshare ESP32-S3-Touch-LCD-3.5** | ESP32-S3R8, 16 MB / 8 MB | **IPS**, 220 cd/m² | ST7796 over SPI. SCLK 5, MOSI 1, MISO 2, DC 3. CS and RST are on a TCA9554 I2C expander at 0x20 (SDA 8, SCL 7). Pins come from a user's working ESPHome config, not from Waveshare. | GPIO6, PWM | FT6336 touch, AXP2101 PMIC, native USB | $25.99-32.99 on waveshare.com, no tiers. The -C variant includes a case. | Small to moderate. Retarget to esp32s3, add a TCA9554 write for RST and CS, possibly bring up AXP2101 rails. ST7796 init unchanged. |
| Guition JC3248W535 | ESP32-S3-WROOM-1, 16 MB / 8 MB | **IPS** | **AXS15231B over QSPI**. CS 45, CLK 47, D0-D3 21/48/40/39. | GPIO1, PWM | Touch built into the AXS15231B, native USB | AliExpress qty 1 about $10-29 from snippets. Spotpear kit $21.49. No bulk tiers confirmed. | Large. New controller and QSPI framing. Windowed updates are broken on some batches, so full-frame only, which needs a 300 KB PSRAM framebuffer. No hardware rotation. |
| Wireless-Tag WT32-SC01 Plus | ESP32-S3, 16 MB / 2 MB | **IPS** | ST7796UI over 8080 8-bit parallel. D0-D7 9/46/3/8/18/17/16/15, WR 47, DC 0, RST 4. | GPIO45, PWM | FT6336U, native USB only, no BOOT button | Alibaba $25.90 sample, $25.50 at 100+, snippet only. RobotShop $33.75, backorder. | Moderate. Same ST7796 commands, but a new transport on esp-hal's `lcd_cam` I8080. |
| lcdwiki ES3C35P | ESP32-S3 | **IPS** | ST77922 over QSPI | GPIO41 | Native USB | Elecrow DLE06235B $20.90 | Large, different controller |
| Elecrow CrowPanel 3.5" ESP32 | ESP32-WROVER-B | TN | ILI9488 over SPI. MISO and touch CS swap between v2.0 and v2.2. | GPIO27 | Resistive | RobotShop $32.88 with case | Moderate, ILI9488 is RGB666 only |

Sources: Sunton pins from [homeding](https://homeding.github.io/boards/esp32/panel-3248S035.htm), [rzeldent](https://github.com/rzeldent/platformio-espressif32-sunton), [espboards](https://www.espboards.dev/esp32/cyd-esp32-3248s035/) and [openHASP](https://openhasp-docs.pages.dev/0.7.0/hardware/sunton/esp32-3248s035). Sunton prices from AliExpress [1](https://www.aliexpress.us/item/3256807290436851.html), [2](https://www.aliexpress.us/item/3256805963246872.html), the [DIYmalls Alibaba listing](https://www.alibaba.com/product-detail/DIYmalls-3-5-inch-ESP32-Display_1600992921720.html) and [Tinytronics](https://www.tinytronics.nl/en/development-boards/microcontroller-boards/with-wi-fi/jingcai-esp32-3248s035c-3.5-inch-tft-display-320*480-pixels-with-capacitive-touchscreen-esp32). lcdwiki board from [lcdwiki](https://www.lcdwiki.com/3.5inch_ESP32-32E_Display) and [Amazon B0D93MBWC2](https://www.amazon.com/dp/B0D93MBWC2). Waveshare from the [wiki](https://www.waveshare.com/wiki/ESP32-S3-Touch-LCD-3.5), [product page](https://www.waveshare.com/esp32-s3-touch-lcd-3.5.htm) and [HA thread](https://community.home-assistant.io/t/how-to-waveshare-3-5-lcd-with-st7796-setup/964308). Guition from the [spec PDF](https://cdn.static.spotpear.com/uploads/picture/learn/ESP32/esp32-s3-touch-3.5/JC3248W535-Specifications-EN.pdf), [atomic14](https://www.atomic14.com/esp32/boards/guition-jc3248w535/), [driver](https://github.com/me-processware/JC3248W535-Driver), AliExpress [1](https://www.aliexpress.us/item/3256812568791678.html), [2](https://www.aliexpress.us/item/3256807380017698.html) and [Spotpear](https://spotpear.com/shop/ESP32-S3-3.5-inch-LCD-Captive-TouchScreen-Display-480x320-Tablet-MP3-Video-Weather-Clock/JC3248W535-Kit.html). WT32-SC01 Plus from the [datasheet](https://www.tinytronics.nl/product_files/005341_ZX3D50CE02S-USRC-4832Datasheet.pdf), [Alibaba](https://wireless-tag.en.alibaba.com/product/1600607711352-813213807/ESP32_3_5_lcd_display_lcd_touch_screen_monitor_WT32_SC01_plus_16MB_based_WT32_S3_WROVER_HMI_Display_Touch_Screen_for_smart_home.html) and [RobotShop](https://Robotshop.com/products/wt32-sc01-plus-esp32-development-board-w-35inch-lcd-ips-display-touch-screen). ES3C35P from its [spec](https://www.lcdwiki.com/res/ES3C35P/3.5inch_IPS_ESP32-S3_Specification_V1.0.pdf) and [Elecrow](https://www.elecrow.com/3-5-esp32-s3-display-320x480-capacitive-ips-touchscreen-with-speaker-mic-bat-interface-supports-ai-voice-chat.html). CrowPanel from [Elecrow](https://www.elecrow.com/esp32-display-3-5-inch-hmi-display-spi-tft-lcd-touch-screen.html) and [RobotShop](https://www.robotshop.com/products/elecrow-crowpanel-35-hmi-esp32-display-480x320-spi-tft-lcd-touchscreen-w-case).

On the Sunton and lcdwiki boards, GPIO12 carries MISO and is also the ESP32's flash-voltage strapping pin. The boards ship that way and boot fine, but leave MISO unconnected in firmware.

My read on enclosures:

- The JC3248W535 ships in its own housing with mounting tabs and has four community cases ([MakerWorld desk](https://makerworld.com/en/models/1026987-guition-jc3248w535-3-5-esp32-desktop-enclosure), [MakerWorld wall](https://makerworld.com/en/models/1027009-guition-jc3248w535-3-5-esp32-wall-mount-enclosure), [Printables](https://www.printables.com/model/1442589-guition-jc3248w535-35-surface-mount-case), [Thingiverse](https://www.thingiverse.com/thing:7127557)).
- The WT32-SC01 Plus has a finished bezel.
- The Sunton and lcdwiki boards are bare PCBs, about 101.5 x 55 mm, with no official case. There is one community Sunton case on [Cults3D](https://cults3d.com/en/3d-model/gadget/display-enclosure-3248s035).
- Waveshare offers a case only with the -C variant.

The all-in-ones are tempting because they delete the harness. But each one makes you pick two of these three: IPS, your existing driver, a price under $20. None gets all three.

## E. Recommendation

### Recommendation table

Per-unit cost of display plus MCU only, with no harness, carrier, enclosure or cable. "Landed" adds shipping spread across the order and the 30% duty allowance on China-shipped items described above. Amazon items carry no extra duty. Where a store shows no tier, the qty 10/50/100 price is the qty 1 price.

| Option | Parts | Price @10 / @50 / @100 | Landed per unit @10 / @50 | Quality notes | Links |
|---|---|---|---|---|---|
| **1. Recommended: generic IPS module + ELEGOO** | iPistBit IPS ST7796U no-touch module + ELEGOO ESP32 | $8.85 + $6.66 = $15.51, same at all qty | about **$18.89 / $18.31**. Display $8.85 + $7.21 shipping per order + 30% allowance, MCU $6.66. | IPS, 178°. Spec text unverified. Same controller and command set. Two-line firmware change. No touch to pay for. | [iPistBit](https://www.aliexpress.us/item/3256805809616969.html), [ELEGOO](https://www.amazon.com/dp/B0D8T53CQ5) |
| 1b. Cheaper variant of 1 | SZFYDOSH IPS ST7796U no-touch + ELEGOO | $6.11 + $6.66 = $12.77 | about **$14.60** plus shipping, which was not confirmed | Same class as 1. Fewer signals about the seller. | [SZFYDOSH](https://www.aliexpress.us/item/3256805628710747.html) |
| **2. Backup: branded IPS module + ELEGOO** | Hosyond MSP3526 (Amazon) or Elecrow DLS30035B + ELEGOO | Hosyond $18.99 + $6.66 = $25.65. Elecrow $16.90 + $6.66 = $23.56. | Hosyond **$25.65**, domestic, no duty. Elecrow about $28.63 before its shipping, which was not confirmed. | Documented lcdwiki MSP3526: BOE IPS glass, 300 cd/m², spec PDF, known init code. Pays for touch you won't use. | [Hosyond](https://www.amazon.com/dp/B0CMD7Y55M), [Elecrow](https://www.elecrow.com/3-5-ips-spi-lcd-capacitive-touch-module-st7796-driver-320-480-resolution.html) |
| 3. IPS all-in-one | Waveshare ESP32-S3-Touch-LCD-3.5 | $25.99-32.99 | about $34-43 with the allowance | IPS, ST7796 over SPI, real brand, consistent revisions. No harness at all. Needs an S3 port plus an I/O-expander write. | [Waveshare](https://www.waveshare.com/esp32-s3-touch-lcd-3.5.htm) |
| 4. Custom carrier PCB, for 50+ units | HongXun IPS 40-pin glass + ESP32-WROOM-32E-N4 on your own PCB | $6.95 + $3.31 = $10.26 @10. $6.74 + $3.07 = $9.81 @50. $6.74 + $2.73 = $9.47 @100. | about $12.75 @50 in parts. PCB, assembly and the USB/regulator BOM come on top; the harness work will cost them. | IPS, no dev board, no CP2102, lowest power. Needs FPC pinout confirmation on a sample first. | [HongXun](https://www.aliexpress.us/item/3256807061236818.html), [LCSC C701341](https://lcsc.com/product-detail/WiFi-Modules_Espressif-Systems-Espressif-Systems-ESP32-WROOM-32E-N4_C701341.html) |
| Today, for reference | XIITIA TN + ELEGOO | $12.00 + $6.66 = $18.66 | $18.66, domestic | TN glass | [XIITIA](https://www.amazon.com/dp/B0DFWL9M9B) |
| Same TN board, direct | SONGXIN generic TN + ELEGOO | $7.92 + $6.66 = $14.58 @10 | about $16.96 | Same TN weakness. Saves about $1.70 a unit. Not worth it. | [SONGXIN](https://www.aliexpress.us/item/3256812768161833.html) |
| TN all-in-one | Sunton ESP32-3248S035 or Hosyond E32N35T | about $19-21, or $18.99 | about $25-27 for Sunton with the allowance; $18.99 for Hosyond on Amazon, not confirmed live | Deletes the harness, keeps your driver, keeps TN | See section D |

### Best quality per dollar for 10 to 50 gift units

Option 1. A no-touch IPS ST7796U module from AliExpress, driven by the ELEGOO board you already trust.

- Landed per unit is about $18 to $19 with a pessimistic duty allowance. That matches what you pay today, and you get IPS glass, which fixes the only real visual flaw in the prototype.
- If the import charge at checkout comes in under 30%, it gets cheaper.
- Pick between iPistBit at $8.85 and SZFYDOSH at $6.11 after the samples arrive. Look at off-axis black, backlight uniformity and the header layout.
- Ship each gift with an A-to-C cable, or confirm the ELEGOO powers from C-to-C. Same rule for any board you pick.

I would not chase the MCU price on the first batch. The ELEGOO at $6.66 has known-good silicon and a US return path, and you'd save at most $4 per unit on a part nobody sees.

### Backup

Option 2. The Hosyond MSP3526 on Amazon at $18.99 is the documented lcdwiki IPS module, with a published spec and known BOE glass. It arrives in days with no customs step. It costs about $7 more per unit than option 1, and you pay for touch you won't use. Use it if the AliExpress modules turn out inconsistent, or if the order deadline is close.

For 50 to 100+ units and a second batch, option 4 is where the cost really drops: under $10 in parts per unit at 100. It also fixes the power and USB-C issues in one step, since the carrier board picks its own regulator and CC resistors. Pair it with the ESP32-S3-WROOM-1 if you want native USB and no CP2102.

### What to order first

Order these, then pick after a side-by-side test against the current TN board.

1. **Two generic IPS no-touch modules**: 2x iPistBit ($8.85, [link](https://www.aliexpress.us/item/3256805809616969.html)) and 2x SZFYDOSH ($6.11, [link](https://www.aliexpress.us/item/3256805628710747.html)). These are the candidates for option 1. Check the header pinout, ST7796 ID with RDID4 (expect 0x77 0x96), inversion, off-axis black and whether two units from the same store match.
2. **One branded reference module**: 1x Hosyond MSP3526 ($18.99, [Amazon B0CMD7Y55M](https://www.amazon.com/dp/B0CMD7Y55M)). It's the yardstick for what the generic IPS modules should look like, and it's the backup if they disappoint.
3. **One IPS all-in-one**: 1x Waveshare ESP32-S3-Touch-LCD-3.5 ($25.99+, [Waveshare](https://www.waveshare.com/esp32-s3-touch-lcd-3.5.htm)). It tells you whether a no-harness IPS build is worth $10 to $15 more per unit. It also runs the same ST7796 driver, so it doubles as a test of the S3 port.

Add-on worth $7: 1x HongXunScreen ST7796 IPS 40-pin glass ([link](https://www.aliexpress.us/item/3256807061236818.html)). If the XIITIA board's glass sits in a socket, try swapping it in. Either way it confirms the bare-panel pinout for option 4.

Don't buy a CYD-style TN board as a sample. It deletes the harness but keeps the problem you're trying to fix.

## Gaps

- Alibaba tiers at 10/50/100/500, lead times and US shipping for the exact module: not confirmed. Every product page put up a CAPTCHA. Getting them needs a logged-in RFQ.
- The XIITIA glass part number: read it off your own panel's FPC.
- Current US duty on Chinese electronics: not confirmed. The 30% allowance is a placeholder until a checkout shows the real line.
- Contrast ratios for the IPS modules: lcdwiki publishes them only as an image in the QD3525 spec.
- AliExpress prices are 11.11 sale prices and may rise after 2026-11-18.
