#![no_std]

extern crate embedded_hal as hal;

use hal::spi::{FullDuplex, Mode, Phase, Polarity};

use nb;
use nb::block;

/// SPI mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

pub struct Ws2812<SPI> {
    spi: SPI,
}

/// RGB
pub type Color = (u8, u8, u8);

impl<SPI, E> Ws2812<SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    /// The SPI bus should run with 3 Mhz, otherwise this won't work
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occurr
    pub fn new(spi: SPI) -> Ws2812<SPI> {
        Self { spi }
    }

    pub fn write<'a, T>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = &'a Color>,
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

    pub fn write_byte(&mut self, mut data: u8) -> Result<(), E> {
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
