#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;
use ws2812_spi as ws2812;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::stm32;
use crate::ws2812::Ws2812;
use cortex_m::peripheral::Peripherals;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();

        // Constrain clocking registers
        let rcc = p.RCC.constrain();

        // Configure clock to 8 MHz (i.e. the default) and freeze it
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        // Configure pins for SPI
        let sck = gpioa.pa5.into_alternate_af0();
        let miso = gpioa.pa6.into_alternate_af0();
        let mosi = gpioa.pa7.into_alternate_af0();

        // Configure SPI with 3Mhz rate
        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            ws2812::MODE,
            3_000_000.hz(),
            clocks,
        );
        const max: usize = 8;
        const color1: (u8, u8, u8) = (0x00, 0xc3 / 5, 0x36 / 5);
        const color2: (u8, u8, u8) = (0x00, 0x24 / 5, 0xb0 / 5);
        let mut data = [0; max * 3];
        let mut main = 0;
        let mut ws = Ws2812::new(spi, &mut data);
        let mut up = true;
        loop {
            for i in 0..max {
                let distance = (main as i32 - i as i32).abs() as u8;
                let c1 = if i == main {
                    color1
                } else {
                    (
                        color1.0 / distance,
                        color1.1 / distance,
                        color1.2 / distance,
                    )
                };
                let c2 = (
                    color2.0 / (max as u8 - distance),
                    color2.1 / (max as u8 - distance),
                    color2.2 / (max as u8 - distance),
                );
                let ct = (c1.0 + c2.0, c1.1 + c2.1, c1.2 + c2.2);
                ws.write(i, ct);
            }
            if up {
                if main == max - 1 {
                    up = false;
                    main -= 2;
                }
                main += 1;
            } else {
                if main == 0 {
                    up = true;
                    main += 2;
                }
                main -= 1;
            }
            ws.flush().unwrap();
            delay.delay_ms(100 as u16);
        }
    }
    loop {
        continue;
    }
}
