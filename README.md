# Ws2812 driver for embedded-hal spi traits

This is a driver for ws2812 leds using spi as Timing. Your spi peripheral has to
run at (mostly) 3MHz, otherwise this crate won't work. You also need to run at a
higher frequency, so there's enough time to prepare the data. 48 MHz should
work. 

Currently you pass an iterator that generates `Color` values to the write
function. You can create one by using `.iter().cloned()` on a `Color` slice.

## It doesn't work!!!
- Does your spi run at 3MHz? Lots if embeded devices don't support this, so you
  may need to look at your hal implementation and at your data sheet
- If the first one is always on, no matter what data you put in, your spi is
  probably not set to idle low. Some spi hals don't support this properly, so
  check with a multi meter that it's low if theres no write ongoing.
- Is your device fast enough? Is your iterator fast enough? Taking too long may
  completly screw up the timings

## TODO
- Find some way implement abstract effects, brightness and stuff somewhere else.
  Maybe "embedded-leds" or something?
- Support different spi frequencies, @jamesmunns had some suggestions for
  timings,
  https://cpldcpu.wordpress.com/2014/01/14/light_ws2812-library-v2-0-part-i-understanding-the-ws2812/
  seems like a good ressource
