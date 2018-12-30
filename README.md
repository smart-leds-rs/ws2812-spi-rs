# Ws2812 driver for embedded-hal spi traits

This is a driver for ws2812 leds using spi as Timing. Your spi peripheral has to
run at (mostly) 3MHz, otherwise this crate won't work. You also need to run at a
higher frequency, so there's enough time to prepare the data. 48 MHz should
work. 

Currently you pass an iterator that generates `Color` values to the write
function. You can create one by using `.iter().cloned()` on a `Color` slice.

## TODO
- Find some way implement abstract effects, brightness and stuff somewhere else.
  Maybe "embedded-leds" or something?
- Support different spi frequencies, @jamesmunns had some suggestions for
  timings
