//! # Use ws2812 leds via spi
//!
//!

#![no_std]

extern crate embedded_hal as hal;

use hal::spi::{FullDuplex, Mode, Phase, Polarity};

use smart_leds_trait::{Color, SmartLedsWrite};

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
    timing: Timing,
}

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
    pub fn new(spi: SPI, timing: Timing) -> Ws2812<SPI> {
        Self { spi, timing }
    }

    /// Stash bits in `byte`, until it's full, then write it out
    fn stash_byte(&mut self, bit: bool, byte: &mut u8, count: &mut u8) -> Result<(), E> {
        *byte = (*byte << 1) | bit as u8;
        *count += 1;
        if *count == 8 {
            self.spi.read().ok();
            block!(self.spi.send(*byte))?;
            *count = 0;
        }
        Ok(())
    }

    /// Write a single byte for ws2812 devices
    fn write_color(&mut self, data: Color) -> Result<(), E> {
        let mut serial_bits = (data.g as u32) << 16 | (data.r as u32) << 8 | (data.b as u32) << 0;
        let mut serial_data = 0;
        let mut serial_count = 0;
        for _ in 0..24 {
            let one_bits = if (serial_bits & 0x00800000) != 0 {
                self.timing.one_high
            } else {
                self.timing.zero_high
            };
            let zero_bits = self.timing.total - one_bits;
            for _ in 0..one_bits {
                self.stash_byte(true, &mut serial_data, &mut serial_count)?;
            }
            for _ in 0..zero_bits {
                self.stash_byte(false, &mut serial_data, &mut serial_count)?;
            }
            serial_bits <<= 1;
        }
        // Now fill the last byte up
        while serial_count != 0 {
            self.stash_byte(false, &mut serial_data, &mut serial_count)?;
        }
        Ok(())
    }
}

impl<SPI, E> SmartLedsWrite for Ws2812<SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    type Error = E;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = Color>,
    {
        for item in iterator {
            self.write_color(item)?;
        }
        for _ in 0..(self.timing.flush) / 8 + 1 {
            block!(self.spi.send(0))?;
            self.spi.read().ok();
        }
        Ok(())
    }
}

pub struct Timing {
    one_high: usize,
    zero_high: usize,
    total: usize,
    total_max: usize,
    flush: usize,
}

impl Timing {
    pub fn new(mhz: u32) -> Option<Self> {
        if mhz < 2_000_000 {
            return None;
        }
        static ONE_HIGH: u32 = 1_510_000;
        static ZERO_HIGH: u32 = 5_000_000;
        static TOTAL: u32 = 1_100_000;
        static TOTAL_MAX: u32 = 200_000;
        static FLUSH: u32 = 3_000;

        let mut zero_high = (mhz / ZERO_HIGH) as usize;
        // Make sure we have at least something
        if zero_high == 0 {
            zero_high = 1;
        }

        // Round up
        let one_high = (mhz / ONE_HIGH + 1) as usize;
        let mut total = (mhz / TOTAL + 1) as usize;
        // Make sure total is at least one higher than one_high
        if total == one_high {
            total = one_high + 1;
        }
        let total_max = (mhz / TOTAL_MAX + 1) as usize;
        let flush = (mhz / FLUSH + 1) as usize;
        Some(Self {
            one_high,
            zero_high,
            total,
            total_max,
            flush,
        })
    }
}
