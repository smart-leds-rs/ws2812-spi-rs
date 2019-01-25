//! This prerenders the data, so that no calculations have to be performed while sending the data.
//!
//! This approach minimizes timing issues, at the cost of much higher ram usage.
//! It also increases the needed time.

extern crate embedded_hal as hal;

use hal::spi::FullDuplex;

use smart_leds_trait::{Color, SmartLedsWrite};

use nb;
use nb::block;

pub struct Ws2812<'a, SPI> {
    spi: SPI,
    timing: Timing,
    data: &'a mut [u8],
}

impl<'a, SPI, E> Ws2812<'a, SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    /// The SPI bus should run exactly with the provided frequency
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// You need to provide a buffer `data`, whoose length is at least 3 the
    /// length of the led strip.
    /// You can calculate the exact amount for frequencies >= 3 MHz with
    /// "spi frequency / 1_100_00" rounded up
    pub fn new(spi: SPI, timing: Timing, data: &'a mut [u8]) -> Ws2812<'a, SPI> {
        Self { spi, timing, data }
    }

    /// Write a single byte for ws2812 devices
    fn write_byte(
        &mut self,
        mut data: u8,
        serial_data: &mut u32,
        serial_count: &mut u8,
        index: &mut usize,
    ) -> Result<(), E> {
        for _ in 0..8 {
            let pattern = if (data & 0x80) != 0 {
                self.timing.one_pattern
            } else {
                self.timing.zero_pattern
            };
            *serial_count += self.timing.len;
            *serial_data |= pattern << (32 - *serial_count);
            while *serial_count > 7 {
                let data = (*serial_data >> 24) as u8;
                self.data[*index] = data;
                *index += 1;
                *serial_data <<= 8;
                *serial_count -= 8;
            }
            data <<= 1;
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), E> {
        for _ in 0..self.timing.flush_bytes {
            block!(self.spi.send(0))?;
            self.spi.read().ok();
        }
        Ok(())
    }
}

impl<SPI, E> SmartLedsWrite for Ws2812<'_, SPI>
where
    SPI: FullDuplex<u8, Error = E>,
{
    type Error = E;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T>(&mut self, iterator: T) -> Result<(), E>
    where
        T: Iterator<Item = Color>,
    {
        if cfg!(feature = "mosi_idle_high") {
            self.flush()?;
        }

        let mut serial_data: u32 = 0;
        let mut serial_count = 0;
        let mut index = 0;
        for item in iterator {
            self.write_byte(item.g, &mut serial_data, &mut serial_count, &mut index)?;
            self.write_byte(item.r, &mut serial_data, &mut serial_count, &mut index)?;
            self.write_byte(item.b, &mut serial_data, &mut serial_count, &mut index)?;
        }
        for d in self.data.iter().take(index + 1) {
            block!(self.spi.send(*d))?;
            self.spi.read().ok();
        }
        self.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Timing {
    one_pattern: u32,
    zero_pattern: u32,
    len: u8,
    flush_bytes: usize,
}

impl Timing {
    /// Create timing values for the provided frequency
    pub fn new(mhz: u32) -> Option<Self> {
        if mhz < 2_000_000 {
            return None;
        }
        static ONE_HIGH: u32 = 1_510_000;
        static ZERO_HIGH: u32 = 5_000_000;
        static TOTAL: u32 = 1_100_000;
        static FLUSH: u32 = 3_000;

        let mut zero_high = mhz / ZERO_HIGH;
        // Make sure we have at least something
        if zero_high == 0 {
            zero_high = 1;
        }

        // Round up
        let one_high = mhz / ONE_HIGH + 1;
        let mut total = mhz / TOTAL + 1;
        // Make sure total is at least one higher than one_high
        if total == one_high {
            total = one_high + 1;
        }
        if total > 28 {
            return None;
        }
        let flush = ((mhz / FLUSH + 1) / 8 + 1) as usize;
        // Create patterns
        let mut one_pattern = 0;
        let mut zero_pattern = 0;
        for _ in 0..one_high {
            one_pattern <<= 1;
            one_pattern |= 1;
        }
        for _ in 0..total - one_high {
            one_pattern <<= 1;
        }
        for _ in 0..zero_high {
            zero_pattern <<= 1;
            zero_pattern |= 1;
        }
        for _ in 0..total - zero_high {
            zero_pattern <<= 1;
        }
        Some(Self {
            one_pattern,
            zero_pattern,
            len: total as u8,
            flush_bytes: flush,
        })
    }
}
