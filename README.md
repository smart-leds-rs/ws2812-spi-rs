# Ws2812 driver for embedded-hal spi traits

This is a driver for ws2812 leds using spi as Timing. Your spi peripheral has to
run at (mostly) 3MHz, otherwise this crate won't work. You also need to run at a
higher frequency, so there's enough time to prepare the data. 48 MHz should
work. 

Currently you pass an iterator that generates `Color` values to the write
function. You can create one by using `.iter().cloned()` on a `Color` slice.

## It doesn't work!!!
- Does your spi run at 3MHz? Lots if embeded devices don't support this, so you
  may need to look at your hal implementation and at your data sheet. If you use 
  the prerendered version, you should also verify that the spi frequency matches
- If the first one is always on, no matter what data you put in, your spi is
  probably not set to idle low. Some spi hals don't support this properly, so
  check with a multi meter that it's low if theres no write ongoing.
  It may also be a timing issue with the first bit being sent, this is the case
  on the stm32f030 with 2MHz (although using it with 2MHz is really not recommended)
- Is your device fast enough? Is your iterator fast enough? Taking too long may
  completly screw up the timings
- Is everything white? This may stem from an spi peripheral that's too slow or
  one that takes too much time in-between bytes

  When opening an issue about wrong/strange colors, it would help if you include
  your code (of course) and a capture of MOSI & SCK from an oscilloscop/ a logic 
  analyzer.
## TODO
- Support different spi frequencies, @jamesmunns had some suggestions for
  timings,
  https://cpldcpu.wordpress.com/2014/01/14/light_ws2812-library-v2-0-part-i-understanding-the-ws2812/
  seems like a good ressource

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
