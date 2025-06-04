//! # Use WS2812 LEDs via SPI on Linux hosts
//!
//! This dynamically allocates an output buffer and writes out the data in a single call.
//! Much better suited for linux or similar environments, but may not always work
//!
//! Intended for use with rppal or linux-embedded-hal

use embedded_hal as hal;

use hal::spi::{Mode, Phase, Polarity, SpiBus};

use core::marker::PhantomData;

use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};

use std::vec;
use std::vec::Vec;

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

pub mod devices {
    pub struct Ws2812;
    pub struct Sk6812w;
}

pub struct Ws2812<SPI, DEVICE = devices::Ws2812, PIXELORDER = pixel_order::GRB> {
    spi: SPI,
    data: Vec<u8>,
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
    pub fn new(spi: SPI) -> Self {
        let data = vec![0; 140];

        Self {
            spi,
            data,
            _device: PhantomData {},
            _pixel_order: PhantomData {},
        }
    }
}

impl<SPI, E> Ws2812<SPI, devices::Sk6812w>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Use sk6812w devices via spi
    ///
    /// The SPI bus should run within 2.3 MHz to 3.8 MHz at least.
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    // The spi frequencies are just the limits, the available timing data isn't
    // complete
    pub fn new_sk6812w(spi: SPI) -> Self {
        let data = vec![0; 140];

        Self {
            spi,
            data,
            _device: PhantomData {},
            _pixel_order: PhantomData {},
        }
    }
}

impl<SPI, D, E> Ws2812<SPI, D, PO>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Write a single byte for ws2812 devices
    fn write_byte(&mut self, mut data: u8) {
        // Send two bits in one spi byte. High time first, then the low time
        // The maximum for T0H is 500ns, the minimum for one bit 1063 ns.
        // These result in the upper and lower spi frequency limits
        let patterns = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];
        for _ in 0..4 {
            let bits = (data & 0b1100_0000) >> 6;
            self.data.push(patterns[bits as usize]);
            data <<= 2;
        }
    }

    fn send_data(&mut self) -> Result<(), E> {
        self.data.extend_from_slice(&[0; 140]);
        self.spi.write(&self.data)?;
        self.data.truncate(140);
        Ok(())
    }
}

impl<SPI, E, PO> SmartLedsWrite for Ws2812<SPI, devices::Ws2812, PO>
where
    SPI: SpiBus<u8, Error = E>,
    PO: OrderedColors,
{
    type Error = E;
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), E>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        for item in iterator {
            let color: RGB8 = item.into();
            let ordered_color = PO::order(color);
            self.write_byte(ordered_color[0]);
            self.write_byte(ordered_color[1]);
            self.write_byte(ordered_color[2]);
        }
        self.send_data()
    }
}

impl<SPI, E> SmartLedsWrite for Ws2812<SPI, devices::Sk6812w>
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
        for item in iterator {
            let item = item.into();
            // SK6812W always expects GRBW order
            self.write_byte(item.g);
            self.write_byte(item.r);
            self.write_byte(item.b);
            self.write_byte(item.a.0);
        }
        self.send_data()
    }
}
