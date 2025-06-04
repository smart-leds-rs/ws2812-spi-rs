//! This prerenders the data, so that no calculations have to be performed while sending the data.
//!
//! This approach minimizes timing issues, at the cost of much higher ram usage.
//! It also increases the needed time.

use embedded_hal as hal;

use hal::spi::{Mode, Phase, Polarity, SpiBus};

use core::marker::PhantomData;

use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};

const RESET_DATA_LEN: usize = 140;

use crate::pixel_order;
use crate::OrderedColors;

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

pub struct Ws2812<'a, SPI, DEVICE = devices::Ws2812, PIXELORDER = pixel_order::GRB> {
    spi: SPI,
    data: &'a mut [u8],
    index: usize,
    _device: PhantomData<DEVICE>,
    _pixel_order: PhantomData<PIXELORDER>,
}

impl<'a, SPI, E, PO> Ws2812<'a, SPI, devices::Ws2812, PO>
where
    SPI: SpiBus<u8, Error = E>,
    PO: OrderedColors,
{
    /// Use WS2812 devices via SPI
    ///
    /// The SPI bus should run within 2 MHz to 3.8 MHz
    ///
    /// You may need to look at the datasheet and your own HAL to verify this.
    ///
    /// You need to provide a buffer `data`, whose length is at least 12 * the
    /// length of the led strip
    /// - + 140 bytes if using the `reset_single_transaction` feature
    /// - + 140 bytes if using the `mosi_idle_high` feature
    /// - + 280 bytes if using the `mosi_idle_high` and `reset_single_transaction` features
    ///
    /// Please ensure that the MCU is pretty fast, otherwise weird timing
    /// issues will occur
    pub fn new(spi: SPI, data: &'a mut [u8]) -> Self {
        Self {
            spi,
            data,
            index: 0,
            _device: PhantomData {},
            _pixel_order: PhantomData {},
        }
    }
}

impl<'a, SPI, E, PO> Ws2812<'a, SPI, devices::Sk6812w, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Use SK6812W devices via SPI
    ///
    /// The SPI bus should run within 2.3 MHz to 3.8 MHz at least.
    ///
    /// You may need to look at the datasheet and your own HAL to verify this.
    ///
    /// You need to provide a buffer `data`, whose length is at least 16 * the
    /// length of the led strip
    /// - + 140 bytes if using the `reset_single_transaction` feature
    /// - + 140 bytes if using the `mosi_idle_high` feature
    /// - + 280 bytes if using the `mosi_idle_high` and `reset_single_transaction` features
    ///
    /// Please ensure that the MCU is pretty fast, otherwise weird timing
    /// issues will occur
    // The SPI frequencies are just the limits, the available timing data isn't
    // complete
    pub fn new_sk6812w(spi: SPI, data: &'a mut [u8]) -> Self {
        Self {
            spi,
            data,
            index: 0,
            _device: PhantomData {},
            _pixel_order: PhantomData {},
        }
    }
}

impl<SPI, D, E, PO> Ws2812<'_, SPI, D, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Write a single byte for WS2812-like devices
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
    // Is always used for `mosi_idle_high`, as otherwise the time required to fill the buffer can lead to idle cycles on the SPI bus
    fn write_reset(&mut self) -> Result<(), Error<E>> {
        if self.index + RESET_DATA_LEN > self.data.len() {
            return Err(Error::OutOfBounds);
        }
        for _ in 0..RESET_DATA_LEN {
            self.data[self.index] = 0;
            self.index += 1;
        }
        Ok(())
    }

    /// Send a reset sequence (140 zeroes) on the bus
    fn send_reset(&mut self) -> Result<(), Error<E>> {
        for _ in 0..RESET_DATA_LEN {
            self.spi.write(&[0]).map_err(Error::Spi)?;
        }

        Ok(())
    }

    fn send_data(&mut self) -> Result<(), E> {
        self.spi.write(&self.data[..self.index])
    }
}

impl<SPI, E, PO> SmartLedsWrite for Ws2812<'_, SPI, devices::Ws2812, PO>
where
    SPI: SpiBus<u8, Error = E>,
    PO: OrderedColors,
{
    type Error = Error<E>;
    type Color = RGB8;
    /// Write all the items of an iterator to a WS2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Error<E>>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.index = 0;

        if cfg!(feature = "mosi_idle_high") {
            self.write_reset()?;
        }

        for item in iterator {
            let color: RGB8 = item.into();
            let ordered_color = PO::order(color);
            self.write_byte(ordered_color[0])?;
            self.write_byte(ordered_color[1])?;
            self.write_byte(ordered_color[2])?;
        }

        if cfg!(feature = "reset_single_transaction") {
            self.write_reset()?;
        }

        self.send_data().map_err(Error::Spi)?;

        if !cfg!(feature = "reset_single_transaction") {
            self.send_reset()?;
        }
        Ok(())
    }
}

impl<SPI, E, PO> SmartLedsWrite for Ws2812<'_, SPI, devices::Sk6812w, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    type Error = Error<E>;
    type Color = RGBW<u8, u8>;
    /// Write all the items of an iterator to a SK6812W strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Error<E>>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.index = 0;

        if cfg!(feature = "mosi_idle_high") {
            self.write_reset()?;
        }

        for item in iterator {
            let item = item.into();
            // SK6812W always expects GRBW order
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
            self.write_byte(item.a.0)?;
        }

        if cfg!(feature = "reset_single_transaction") {
            self.write_reset()?;
        }

        self.send_data().map_err(Error::Spi)?;

        if !cfg!(feature = "reset_single_transaction") {
            self.send_reset()?;
        }
        Ok(())
    }
}
