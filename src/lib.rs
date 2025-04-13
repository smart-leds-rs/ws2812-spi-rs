//! # Use ws2812 leds via spi
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! Needs a type implementing the `spi::SpiBus` trait.
//!
//! The spi peripheral should run at 2MHz to 3.8 MHz

// Timings for ws2812 from https://cpldcpu.files.wordpress.com/2014/01/ws2812_timing_table.png
// Timings for sk6812 from https://cpldcpu.wordpress.com/2016/03/09/the-sk6812-another-intelligent-rgb-led/

#![cfg_attr(not(feature = "std"), no_std)]

use embedded_hal as hal;

#[cfg(feature = "std")]
pub mod hosted;
pub mod prerendered;

use hal::spi::{Mode, Phase, Polarity, SpiBus};

use core::marker::PhantomData;
use core::slice::from_ref;

use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};

/// SPI mode that can be used for this crate
///
/// Provided for convenience
/// Doesn't really matter
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

pub mod devices {
    pub struct Ws2812;
    pub struct Sk6812w;
}

/// The color order the WS2812-like device expects. In most cases, this is GRB
/// and doesn't need to be specified separately.
///
/// For SK6812W devices (with a separate white channel) this is unused
pub mod pixel_order {
    pub struct RGB;
    pub struct RBG;
    pub struct GRB;
    pub struct GBR;
    pub struct BRG;
    pub struct BGR;
}

/// Used to define the pixel order, refer to the pixel_order module
pub trait OrderedColors {
    fn order(color: RGB8) -> [u8; 3];
}

macro_rules! impl_ordered_colors {
    ($struct_name:ident, $r_field:ident, $g_field:ident, $b_field:ident) => {
        impl OrderedColors for pixel_order::$struct_name {
            fn order(color: RGB8) -> [u8; 3] {
                [color.$r_field, color.$g_field, color.$b_field]
            }
        }
    };
}

impl_ordered_colors!(RGB, r, g, b);
impl_ordered_colors!(RBG, r, b, g);
impl_ordered_colors!(GRB, g, r, b);
impl_ordered_colors!(GBR, g, b, r);
impl_ordered_colors!(BRG, b, r, g);
impl_ordered_colors!(BGR, b, g, r);

pub struct Ws2812<SPI, DEVICE = devices::Ws2812, PIXELORDER = pixel_order::GRB> {
    spi: SPI,
    _device: PhantomData<DEVICE>,
    _pixel_order: PhantomData<PIXELORDER>,
}

impl<SPI, E, PO> Ws2812<SPI, devices::Ws2812, PO>
where
    SPI: SpiBus<u8, Error = E>,
    PO: OrderedColors,
{
    /// Use ws2812 devices via spi
    ///
    /// The SPI bus should run within 2 MHz to 3.8 MHz
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            _device: PhantomData {},
            _pixel_order: PhantomData {},
        }
    }
}

impl<SPI, E, PO> Ws2812<SPI, devices::Sk6812w, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Use sk6812w devices via spi
    ///
    /// The SPI bus should run within 2.3 MHz to 3.8 MHz at least.
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    // The spi frequencies are just the limits, the available timing data isn't
    // complete
    pub fn new_sk6812w(spi: SPI) -> Self {
        Self {
            spi,
            _device: PhantomData {},
            _pixel_order: PhantomData {},
        }
    }
}

impl<SPI, D, E, PO> Ws2812<SPI, D, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Write a single byte for ws2812 devices
    fn write_byte(&mut self, mut data: u8) -> Result<(), E> {
        // Send two bits in one spi byte. High time first, then the low time
        // The maximum for T0H is 500ns, the minimum for one bit 1063 ns.
        // These result in the upper and lower spi frequency limits
        let patterns = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];
        for _ in 0..4 {
            let bits = (data & 0b1100_0000) >> 6;
            self.spi.write(from_ref(&patterns[bits as usize]))?;
            data <<= 2;
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<(), E> {
        // Should be > 300μs, so for an SPI Freq. of 3.8MHz, we have to send at least 1140 low bits or 140 low bytes
        for _ in 0..140 {
            self.spi.write(from_ref(&0))?;
        }
        Ok(())
    }
}

impl<SPI, E, PO: OrderedColors> SmartLedsWrite for Ws2812<SPI, devices::Ws2812, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    type Error = E;
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), E>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        if cfg!(feature = "mosi_idle_high") {
            self.reset()?;
        }

        for item in iterator {
            let color: RGB8 = item.into();
            let ordered_color = PO::order(color);
            self.write_byte(ordered_color[0])?;
            self.write_byte(ordered_color[1])?;
            self.write_byte(ordered_color[2])?;
        }
        self.reset()?;
        Ok(())
    }
}

impl<SPI, E, PO> SmartLedsWrite for Ws2812<SPI, devices::Sk6812w, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    type Error = E;
    type Color = RGBW<u8, u8>;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), E>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        if cfg!(feature = "mosi_idle_high") {
            self.reset()?;
        }

        for item in iterator {
            let item = item.into();
            // SK6812W always expects GRBW order
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
            self.write_byte(item.a.0)?;
        }
        self.reset()?;
        Ok(())
    }
}
