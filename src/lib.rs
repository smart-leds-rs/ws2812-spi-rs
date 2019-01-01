//! # Use ws2812 leds via spi
//!
//!

#![no_std]

extern crate embedded_hal as hal;

use hal::spi::{FullDuplex, Mode, Phase, Polarity};

use nb;
use nb::block;

/// SPI mode that is needed for this crate
///
/// Provided for convenience
///
/// If you have strange issues, like the first led always running, you should
/// verify that the spi is idle low
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

pub struct Ws2812<SPI> {
    spi: SPI,
}

/// Color type with RGB colors
pub type Color = (u8, u8, u8);

impl<SPI, E> Ws2812<SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    /// The SPI bus should run with 3 Mhz, otherwise this won't work.
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    pub fn new(spi: SPI) -> Ws2812<SPI> {
        Self { spi }
    }

    /// Write all the items of an iterator to a ws2812 strip
    pub fn write<'a, T>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = Color>,
    {
        for item in iterator {
            self.write_byte(item.1)?;
            self.write_byte(item.0)?;
            self.write_byte(item.2)?;
        }
        for _ in 0..20 {
            block!(self.spi.send(0))?;
            self.spi.read().ok();
        }
        Ok(())
    }

    /// Write a single byte for ws2812 devices
    fn write_byte(&mut self, mut data: u8) -> Result<(), E> {
        let mut serial_bits: u32 = 0;
        for _ in 0..3 {
            let bit = data & 0x80;
            let pattern = if bit == 0x80 { 0b110 } else { 0b100 };
            serial_bits = pattern | (serial_bits << 3);
            data <<= 1;
        }
        block!(self.spi.send((serial_bits >> 1) as u8))?;
        // Split this up to have a bit more lenient timing
        for _ in 3..8 {
            let bit = data & 0x80;
            let pattern = if bit == 0x80 { 0b110 } else { 0b100 };
            serial_bits = pattern | (serial_bits << 3);
            data <<= 1;
        }
        // Some implementations (stm32f0xx-hal) want a matching read
        // We don't want to block so we just hope it's ok this way
        self.spi.read().ok();
        block!(self.spi.send((serial_bits >> 8) as u8))?;
        self.spi.read().ok();
        block!(self.spi.send(serial_bits as u8))?;
        self.spi.read().ok();
        Ok(())
    }
}

/// An iterator that provides brightness reduction
pub struct Brightness<I> {
    iter: I,
    brightness: u8,
}

impl<'a, I> Iterator for Brightness<I>
where
    I: Iterator<Item = Color>,
{
    type Item = Color;

    fn next(&mut self) -> Option<Color> {
        self.iter.next().map(|a| {
            (
                (a.0 as u32 * self.brightness as u32 / 256) as u8,
                (a.1 as u32 * self.brightness as u32 / 256) as u8,
                (a.2 as u32 * self.brightness as u32 / 256) as u8,
            )
        })
    }
}

/// Pass your iterator into this function to get reduced brightness
///
/// This is linear scaling, so it won't appear linear to human eyes
pub fn brightness<I>(iter: I, brightness: u8) -> Brightness<I>
where
    I: Iterator<Item = Color>,
{
    Brightness { iter, brightness }
}
