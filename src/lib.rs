//! # Use ws2812 leds via spi
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! Needs a type implementing the `spi::FullDuplex` trait.
//!
//! The spi peripheral should run at 2MHz to 3.8 MHz

// Timings for ws2812 from https://cpldcpu.files.wordpress.com/2014/01/ws2812_timing_table.png
// Timings for sk6812 from https://cpldcpu.wordpress.com/2016/03/09/the-sk6812-another-intelligent-rgb-led/

#![no_std]

extern crate embedded_hal as hal;

pub mod prerendered;

use hal::spi::{FullDuplex, Mode, Phase, Polarity};

use core::marker::PhantomData;

use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};

use nb;
use nb::block;

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

pub struct Ws2812<SPI, DEVICE = devices::Ws2812> {
    spi: SPI,
    device: PhantomData<DEVICE>,
}

impl<SPI, E> Ws2812<SPI>
where
    SPI: FullDuplex<u8, Error = E>,
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
            device: PhantomData {},
        }
    }
}

impl<SPI, E> Ws2812<SPI, devices::Sk6812w>
where
    SPI: FullDuplex<u8, Error = E>,
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
            device: PhantomData {},
        }
    }
}

impl<SPI, D, E> Ws2812<SPI, D>
where
    SPI: FullDuplex<u8, Error = E>,
{
    /// Write a single byte for ws2812 devices
    fn write_byte(&mut self, mut data: u8) -> Result<(), E> {
        // Send two bits in one spi byte. High time first, then the low time
        // The maximum for T0H is 500ns, the minimum for one bit 1063 ns.
        // These result in the upper and lower spi frequency limits
        let patterns = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];
        for _ in 0..4 {
            let bits = (data & 0b1100_0000) >> 6;
            block!({
                // Some implementations (stm32f0xx-hal) want a matching read
                // We don't want to block so we just hope it's ok this way
                self.spi.send(patterns[bits as usize]);
                self.spi.read()
            })?;
            data <<= 2;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), E> {
        for _ in 0..20 {
            block!({
                self.spi.send(0);
                self.spi.read()
            })?;
        }
        Ok(())
    }
}

impl<SPI, E> SmartLedsWrite for Ws2812<SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    type Error = E;
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        if cfg!(feature = "mosi_idle_high") {
            self.flush()?;
        }

        for item in iterator {
            let item = item.into();
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
        }
        self.flush()?;
        Ok(())
    }
}

impl<SPI, E> SmartLedsWrite for Ws2812<SPI, devices::Sk6812w>
where
    SPI: FullDuplex<u8, Error = E>,
{
    type Error = E;
    type Color = RGBW<u8, u8>;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        if cfg!(feature = "mosi_idle_high") {
            self.flush()?;
        }

        for item in iterator {
            let item = item.into();
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
            self.write_byte(item.a.0)?;
        }
        self.flush()?;
        Ok(())
    }
}
