//! # Use ws2812 leds via spi
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! Needs a type implementing the `spi::FullDuplex` trait.
//!0
//! The spi peripheral should run at 4MHz. Each neopixel bit is 5 SPI bits. 
//! In this way the neopixel timing is exact 800 Khrz, and the timing for the low and high bits
//! is within the specification. 
//!            specification (ns)      this variant (ns)      lib.rs variant (ns)
//! low bit:   800/450                 725/500                 999/333
//! high bit   400/850                 500/750                 333/999
//! frequency  800 Khz                 800Khz                  750 Khz
//! 
//! The max deviation for the timing here is 100 ns, where the max deviation of the timing in lib.rs
//! is 199 us. 
//! 
//! Due to the use of half duplex, the tx fifo of the spi is optimal used , so 6 neopixel bits (== 30 spi bits) 
//! can be stored in the fifo.
//! 
//! During the calculation of the next pixel one can see that the bit pattern is stretched (at 16 mhz cpu clock), 
//! so calculation takes longer than the six neopixel bits in the fifo (longer than 6 * 1.25 us = 7.5 us).
//! 
//! Note that the 5 bit pattern always has a zero at the beginning and the end. In this way stretching  will always 
//! happen during a zero phase. The zero phase is less time sensitive than the one phase.
//!  
//! This variant can run on a cpu clock of 16 Mhz and multiples (32, 64)
//! 
//! Note that this halfduplex variant can only run with a feature branch of the hal.
//! where the fullduplex spi can be switched to half duplex output mode and 5 bits wide operation
//! An example how to use this variant can be found in the examples of this feature branch see
//! https://github.com/smeenka/rust-examples/blob/master/nucleo-G070/examples/neopixel5bits.rs
//! 
// 
// Timings for ws2812 from https://cpldcpu.files.wordpress.com/2014/01/ws2812_timing_table.png
// Timings for sk6812 from https://cpldcpu.wordpress.com/2016/03/09/the-sk6812-another-intelligent-rgb-led/

use embedded_hal as hal;

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
        // Send one bit in one spi byte. High time first, then the low time
        // clock is 4 hrz, 5 bits, each bit is 0.25 us.
        // a one bit is send as a pulse of 0.75 high -- 0.50 low
        // a zero bit is send as a pulse of 0.50 high -- 0.75 low
        // clock frequency for the neopixel is exact 800 khz

        for _ in 0..8 {
            let pattern = 
                match data & 0x80 {
                    0x80 => 0b1110_1110,
                    0x0  => 0b1110_1100,
                    _    => 0b1111_1111
                }; 
            block!(self.spi.send(pattern));
            data = data << 1;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), E> {
        // Should be > 300μs, so for an SPI Freq. of 3.8MHz, we have to send at least 1140 low bits or 140 low bytes
        for _ in 0..140 {
            block!(self.spi.send(0))?;
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
        // We introduce an offset in the fifo here, so there's always one byte in transit
        // Some MCUs (like the stm32f1) only a one byte fifo, which would result
        // in overrun error if two bytes need to be stored
        block!(self.spi.send(0))?;
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
        // We introduce an offset in the fifo here, so there's always one byte in transit
        // Some MCUs (like the stm32f1) only a one byte fifo, which would result
        // in overrun error if two bytes need to be stored
        block!(self.spi.send(0))?;
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
