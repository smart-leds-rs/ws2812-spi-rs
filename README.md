# Ws2812 driver for embedded-hal spi traits

For usage with the [smart-leds](https://github.com/smart-leds-rs/smart-leds)
crate.

An embedded-hal driver for ws2812 leds using spi as the timing provider.

![rainbow on stm32f0](./stm32f0_ws2812_spi_rainbow.gif)

It provides two variants:
- The normal usage

  Your spi peripheral has to run betwee 2MHz and 3.8MHz & the SPI data is created on-the-fly.
  This means that your core has to be reasonably fast (48 MHz should suffice).
- Prerendered

  If your core is too slow (for example, the AVR family), you
  may want to use this. It creates all the data beforehand & then sends it. This
  means that you have to provide a data array that's large enough for all the
  spi data.

## It doesn't work!!!
- Do you use the normal variant? Does your spi run at the right frequency?

  Your CPU might be too slow, but this can also depend on the HAL implementation
  & your Iterator chain. Using the `prerendered` variant might help. For many
  SPI peripherals, the clock generations is way less sophisticated than e.g.
  the UART peripheral. You should verify it runs at an acceptable frequency, by
  either studying the datasheet & the hal code or using a logic analyzer. An
  fx2 based one, commonly available under $10 works great for this.

- If the first led is always on, no matter what data you put in, your spi is
  probably not setting the mosi line to low on idle (You can check with a multimeter).
  It may also be a timing issue with the first bit being sent, this is the case
  on the stm32f030 with 2MHz.

  You could try using the `mosi_idle_high` feature, it might help.

- Is your device fast enough? Is your iterator fast enough? Taking too long may
  completely screw up the timings for the normal version. Try the prerendered variant.

- Is everything white? This may stem from an spi peripheral that's too slow or
  one that takes too much time in-between bytes

- Is your first LED the wrong brightness/color while the rest of your LED's do what you expect?
This is due to low voltage of data line, or too much of a voltage difference between Vin and Din
voltages, making some "high" bits read as "low" bits to the chip. Due to the chips circuitry, these
voltages are regulated as they are passed on to the next LED in the line, which is why the other LED's
perform as expected. (For more on exactly what is going on here, see [Hackaday | Cheating at 5V WS2812 Control to Use 3.3V Data](https://hackaday.com/2017/01/20/cheating-at-5v-ws2812-control-to-use-a-3-3v-data-line/))

- Are you using the `--release` compiler flag?  

  The timing of each byte passed over SPI is very sensitive, and running code compiled
  without full optimizations can throw off your timing. Always use either `--release`
  flag with your `cargo <command>`, or alternatively set `[profile.dev] opt-level = "3"` 
  To ensure timing matches what your programmed. A dead giveaway of this is when all 
  pixels go full brightness for every color. 

When opening an issue about wrong/strange data, it would help if you include
your code (of course) and a capture of MOSI & SCK from an oscilloscope/a logic
analyzer.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
