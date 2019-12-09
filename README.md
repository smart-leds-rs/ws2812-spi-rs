# Ws2812 driver for embedded-hal spi traits

For usage with the [smart-leds](https://github.com/smart-leds-rs/smart-leds)
crate.

An embedded-hal driver for ws2812 leds using spi as the timing provider. 

It provides two variants: 
- The normal usage
  
  Your spi peripheral has to run at 3MHz & the SPI data is created on-the-fly. 
  This means that your core has to be reasonably fast (~48 MHz).
- Prerendered

  If your core is too slow or your SPI peripheral has a different frequency, you
  may want to use this. It creates all the data beforehand & then sends it. This
  means that you have to provide a data array that's large enough for all the
  spi data.

## It doesn't work!!!
- Do you use the normal variant? Does your spi run at 3MHz? Lots if embeded
  devices don't support this, so you may need to look at your hal implementation
  and at your data sheet. If you use the prerendered version, you should also
  verify that the spi frequency matches
- If the first led is always on, no matter what data you put in, your spi is
  probably not setting the mosi line to low on idle (You can check with a multimeter).
  It may also be a timing issue with the first bit being sent, this is the case
  on the stm32f030 with 2MHz (although using it with 2MHz is really not
  recommended).

  You could try using the `mosi_idle_high` feature, it might help.

- Is your device fast enough? Is your iterator fast enough? Taking too long may
  completely screw up the timings for the normal version. Try the prerendered variant.

- Is everything white? This may stem from an spi peripheral that's too slow or
  one that takes too much time in-between bytes

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
