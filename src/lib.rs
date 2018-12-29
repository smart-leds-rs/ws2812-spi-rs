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

pub struct Ws2812<'a, SPI> {
    spi: SPI,
    data: &'a mut [u8],
}

impl<'a, SPI, E> Ws2812<'a, SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    /// The SPI bus should run with 3 Mhz, otherwise this won't work
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occurr
    pub fn new(spi: SPI, data: &'a mut [u8]) -> Ws2812<'a, SPI> {
        if data.len() % 3 != 0 {
            panic!("Doesn't match")
        }
        Self { spi, data }
    }

    /// This writes the pixel data to the data slice
    ///
    /// Please make sure your slice is big enough
    pub fn write(&mut self, pos: usize, r: u8, g: u8, b: u8) {
        self.data[pos * 3] = g;
        self.data[(pos * 3) + 1] = r;
        self.data[(pos * 3) + 2] = b;
    }

    pub fn clear(&mut self) {
        for i in self.data.iter_mut() {
            *i = 0;
        }
    }

    pub fn flush(&mut self) -> Result<(), E> {
        for data in self.data.iter() {
            let mut serial_bits: u32 = 0;
            let mut data = *data;
            for _ in 0..8 {
                let bit = data & 0x80;
                let pattern = if bit == 0x80 { 0b110 } else { 0b100 };
                serial_bits = pattern | (serial_bits << 3);
                data <<= 1;
            }
            block!(self.spi.send((serial_bits >> 16) as u8))?;
            // Some implementations (stm32f0xx-hal) want a matching read
            // We don't want to block so we just hope to write
            self.spi.read().ok();
            // STM32f0xx-hal errors out if we never read
            block!(self.spi.send((serial_bits >> 8) as u8))?;
            self.spi.read().ok();
            block!(self.spi.send(serial_bits as u8))?;
            self.spi.read().ok();
        }
        // Now reset it
        // We need to wait 50 us with a low signa, that works out to just
        // under 19  0 bytes, we're going to do 20 just to be sure
        for _ in 0..20 {
            block!(self.spi.send(0))?;
            self.spi.read().ok();
        }
        Ok(())
    }
}
