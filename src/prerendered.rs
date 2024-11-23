//! This prerenders the data, so that no calculations have to be performed while sending the data.
//!
//! This approach minimizes timing issues, at the cost of much higher ram usage.
//! It also increases the needed time.

use embedded_hal as hal;

use hal::spi::{Mode, Phase, Polarity, SpiBus};

use core::marker::PhantomData;

use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};

const FLUSH_DATA_LEN: usize = 140;

/// SPI mode that can be used for this crate
///
/// Provided for convenience
/// Doesn't really matter
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

#[derive(Debug)]
pub enum Error<E> {
    OutOfBounds,
    Spi(E),
}

pub mod devices {
    pub struct Ws2812;
    pub struct Sk6812w;
}

pub struct Ws2812<'a, SPI, DEVICE = devices::Ws2812> {
    spi: SPI,
    data: &'a mut [u8],
    index: usize,
    device: PhantomData<DEVICE>,
}

impl<'a, SPI, E> Ws2812<'a, SPI>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Use ws2812 devices via spi
    ///
    /// The SPI bus should run within 2 MHz to 3.8 MHz
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// You need to provide a buffer `data`, whoose length is at least 12 * the
    /// length of the led strip + 140 bytes (or 280, if using the `mosi_idle_high` feature)
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    pub fn new(spi: SPI, data: &'a mut [u8]) -> Self {
        Self {
            spi,
            data,
            index: 0,
            device: PhantomData {},
        }
    }
}

impl<'a, SPI, E> Ws2812<'a, SPI, devices::Sk6812w>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Use sk6812w devices via spi
    ///
    /// The SPI bus should run within 2.3 MHz to 3.8 MHz at least.
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// You need to provide a buffer `data`, whoose length is at least 16 * the
    /// length of the led strip + 140 bytes (or 280, if using the `mosi_idle_high` feature)
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    // The spi frequencies are just the limits, the available timing data isn't
    // complete
    pub fn new_sk6812w(spi: SPI, data: &'a mut [u8]) -> Self {
        Self {
            spi,
            data,
            index: 0,
            device: PhantomData {},
        }
    }
}

impl<'a, SPI, D, E> Ws2812<'a, SPI, D>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Write a single byte for ws2812 devices
    fn write_byte(&mut self, mut data: u8) -> Result<(), Error<E>> {
        // Send two bits in one spi byte. High time first, then the low time
        // The maximum for T0H is 500ns, the minimum for one bit 1063 ns.
        // These result in the upper and lower spi frequency limits
        let patterns = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];

        if self.index > self.data.len() - 4 {
            return Err(Error::OutOfBounds);
        }
        for _ in 0..4 {
            let bits = (data & 0b1100_0000) >> 6;
            self.data[self.index] = patterns[bits as usize];
            self.index += 1;
            data <<= 2;
        }
        Ok(())
    }

    /// Add a reset sequence (140 zeroes) to the data buffer
    #[cfg(feature = "reset_single_transaction")]
    fn flush(&mut self) -> Result<(), Error<E>> {
        let flush_data = [(); FLUSH_DATA_LEN].map(|_| 0);

        if self.index + FLUSH_DATA_LEN > self.data.len() {
            return Err(Error::OutOfBounds);
        }
        self.data[self.index..(self.index + FLUSH_DATA_LEN)].copy_from_slice(&flush_data);
        self.index += FLUSH_DATA_LEN;
        Ok(())
    }

    /// Send a reset sequence (140 zeroes) on the bus
    #[cfg(not(feature = "reset_single_transaction"))]
    fn flush(&mut self) -> Result<(), Error<E>> {
        for _ in 0..FLUSH_DATA_LEN {
            self.spi.write(&[0]).map_err(Error::Spi)?;
        }

        Ok(())
    }

    fn send_data(&mut self) -> Result<(), E> {
        self.spi.write(&self.data[..self.index])
    }
}

impl<'a, SPI, E> SmartLedsWrite for Ws2812<'a, SPI>
where
    SPI: SpiBus<u8, Error = E>,
{
    type Error = Error<E>;
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Error<E>>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.index = 0;

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
        self.send_data().map_err(Error::Spi)
    }
}

impl<'a, SPI, E> SmartLedsWrite for Ws2812<'a, SPI, devices::Sk6812w>
where
    SPI: SpiBus<u8, Error = E>,
{
    type Error = Error<E>;
    type Color = RGBW<u8, u8>;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Error<E>>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.index = 0;

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
        self.send_data().map_err(Error::Spi)
    }
}
