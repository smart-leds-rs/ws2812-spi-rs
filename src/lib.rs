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
    pub fn new(spi: SPI) -> Ws2812<SPI> {
        Self { spi }
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
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
        }
        for _ in 0..20 {
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
    pub fn new(mhz: u32) -> Self {
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
        Self {
            one_high,
            zero_high,
            total,
            total_max,
            flush,
        }
    }
}
