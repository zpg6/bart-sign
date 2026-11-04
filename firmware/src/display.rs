//! ST7796S (320x480, used as 480x320 landscape) over SPI3 (VSPI) + DMA.
//!
//! This module exists so the bus types are checked by the compiler:
//! esp-hal `SpiDma<Blocking>` implements `embedded_hal::spi::SpiBus`,
//! `ExclusiveDevice` turns it into an `embedded_hal::spi::SpiDevice`, and
//! `mipidsi::interface::SpiInterface` accepts that.

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};
use esp_hal::peripherals::{DMA_SPI3, SPI3};
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config, Spi, SpiDma};
use esp_hal::time::Rate;
use esp_hal::{Blocking, dma_rx_buffer, dma_tx_buffer};
use mipidsi::interface::SpiInterface;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;
use mipidsi::dcs::{InterfaceExt, SetAddressMode};
use mipidsi::interface::Interface;
use mipidsi::models::{Model, ModelInitError};
use mipidsi::options::ModelOptions;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{Builder, Display};

pub type DisplaySpi = ExclusiveDevice<SpiDma<'static, Blocking>, Output<'static>, Delay>;
pub type DisplayInterface = SpiInterface<'static, DisplaySpi, Output<'static>>;
pub type Lcd = Display<DisplayInterface, St7796Vendor, Output<'static>>;

/// ST7796S with the panel vendor's power, VCOM and gamma settings (the
/// Sitronix reference used by TFT_eSPI and Espressif's esp_lcd_st7796).
/// mipidsi's stock ST7796 model never unlocks these registers, which leaves
/// the chip defaults: lifted blacks and a flat gamma on this TN glass.
pub struct St7796Vendor;

const VENDOR_INIT: &[(u8, &[u8])] = &[
    (0xF0, &[0xC3]), // CSCON: unlock command set 2, part 1
    (0xF0, &[0x96]), // part 2
    (0xB4, &[0x01]), // 1-dot inversion
    (0xB6, &[0x80, 0x02, 0x3B]),
    (0xB7, &[0xC6]),
    (0xE8, &[0x40, 0x8A, 0x00, 0x00, 0x29, 0x19, 0xA5, 0x33]),
    (0xC1, &[0x06]), // GVDD 3.85 V
    (0xC2, &[0xA7]),
    (0xC5, &[0x18]), // VCOM 0.9 V
    (0xE0, &[0xF0, 0x09, 0x0B, 0x06, 0x04, 0x15, 0x2F, 0x54, 0x42, 0x3C, 0x17, 0x14, 0x18, 0x1B]),
    (0xE1, &[0xE0, 0x09, 0x0B, 0x06, 0x04, 0x03, 0x2B, 0x43, 0x42, 0x3B, 0x16, 0x14, 0x17, 0x1B]),
    (0xF0, &[0x3C]), // lock
    (0xF0, &[0x69]),
];

impl Model for St7796Vendor {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (320, 480);
    const RESET_DURATION: u32 = 10_000;

    fn init<DELAY: DelayNs, DI: Interface>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, ModelInitError<DI::Error>> {
        let madctl = SetAddressMode::from(options);
        delay.delay_ms(120);
        di.write_raw(0x11, &[])?; // SLPOUT
        delay.delay_ms(120);
        di.write_command(madctl)?;
        di.write_raw(0x3A, &[0x55])?; // 16 bpp
        for &(cmd, args) in VENDOR_INIT {
            di.write_raw(cmd, args)?;
        }
        let inversion = match options.invert_colors {
            ColorInversion::Normal => 0x20,
            ColorInversion::Inverted => 0x21,
        };
        di.write_raw(inversion, &[])?;
        di.write_raw(0x13, &[])?; // NORON
        di.write_raw(0x29, &[])?; // DISPON
        delay.delay_ms(20);
        Ok(madctl)
    }
}

#[derive(Debug)]
pub enum DisplayError {
    Spi,
    Dma,
    Init,
}

/// Panel wiring. GPIO18 (SCK), GPIO23 (MOSI) and GPIO5 (CS) are VSPI's
/// IO_MUX pins on the ESP32.
pub struct DisplayPins {
    pub sck: AnyPin<'static>,
    pub mosi: AnyPin<'static>,
    pub cs: AnyPin<'static>,
    pub dc: AnyPin<'static>,
    pub rst: AnyPin<'static>,
}

/// Bring up the panel. `pixel_buf` is mipidsi's staging buffer for pixel
/// batches; 4 KB or more is reasonable.
pub fn init(
    spi: SPI3<'static>,
    dma: DMA_SPI3<'static>,
    pins: DisplayPins,
    pixel_buf: &'static mut [u8],
) -> Result<Lcd, DisplayError> {
    let DisplayPins { sck, mosi, cs, dc, rst } = pins;

    // DMA copy buffers, used when a slice is not DMA-capable (e.g. in flash
    // or not word aligned on the ESP32).
    let rx = dma_rx_buffer!(32).map_err(|_| DisplayError::Dma)?;
    let tx = dma_tx_buffer!(4092).map_err(|_| DisplayError::Dma)?;

    let bus = Spi::new(
        spi,
        Config::default().with_frequency(Rate::from_mhz(40)).with_mode(Mode::_0),
    )
    .map_err(|_| DisplayError::Spi)?
    .with_sck(sck)
    .with_mosi(mosi)
    .with_dma(dma)
    .with_buffers(rx, tx);

    let out = OutputConfig::default();
    let cs = Output::new(cs, Level::High, out);
    let dc = Output::new(dc, Level::Low, out);
    let rst = Output::new(rst, Level::High, out);

    let spi_dev = ExclusiveDevice::new(bus, cs, Delay::new()).map_err(|_| DisplayError::Spi)?;
    let di = SpiInterface::new(spi_dev, dc, pixel_buf);

    let mut delay = Delay::new();
    Builder::new(St7796Vendor, di)
        .reset_pin(rst)
        .display_size(320, 480)
        // Landscape with the pin header down the left edge, seen from the
        // front. These panels set the column-reverse bit in portrait, so
        // mipidsi's plain Deg90 comes out upside down; mirrored Deg90 is
        // MADCTL 0x28 (MV | BGR).
        .orientation(Orientation { rotation: Rotation::Deg90, mirrored: true })
        .color_order(ColorOrder::Bgr)
        // TN glass: no inversion (inverted shows black as white).
        .invert_colors(ColorInversion::Normal)
        .init(&mut delay)
        .map_err(|_| DisplayError::Init)
}

/// Bring-up pattern for checking color order, inversion and rotation:
/// - three bars labelled RED / GREEN / BLUE, which must show those colors
///   (swapped red and blue means BGR/RGB is wrong; the background showing
///   white means inversion is wrong);
/// - a white corner triangle and "TOP LEFT" label, which must sit in the top
///   left with readable (not mirrored) text.
pub fn test_pattern(lcd: &mut Lcd) -> Result<(), DisplayError> {
    use embedded_graphics::mono_font::MonoTextStyle;
    use embedded_graphics::mono_font::ascii::FONT_10X20;
    use embedded_graphics::pixelcolor::Rgb565;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, Triangle};
    use embedded_graphics::text::{Alignment, Text};

    let e = |_| DisplayError::Init;
    lcd.clear(Rgb565::BLACK).map_err(e)?;

    let bars = [(Rgb565::RED, "RED"), (Rgb565::GREEN, "GREEN"), (Rgb565::BLUE, "BLUE")];
    for (i, (color, label)) in bars.into_iter().enumerate() {
        let x = 30 + i as i32 * 150;
        Rectangle::new(Point::new(x, 110), Size::new(120, 120))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(lcd)
            .map_err(e)?;
        Text::with_alignment(label, Point::new(x + 60, 260), MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE), Alignment::Center)
            .draw(lcd)
            .map_err(e)?;
    }

    Triangle::new(Point::new(0, 0), Point::new(60, 0), Point::new(0, 60))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(lcd)
        .map_err(e)?;
    let white = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::new("TOP LEFT", Point::new(70, 30), white).draw(lcd).map_err(e)?;
    Text::with_alignment("BART SIGN DISPLAY TEST", Point::new(240, 80), white, Alignment::Center)
        .draw(lcd)
        .map_err(e)?;
    Rectangle::new(Point::zero(), Size::new(480, 320))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::YELLOW, 1))
        .draw(lcd)
        .map_err(e)?;
    Ok(())
}
