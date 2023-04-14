# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Increased reset time from ~50μs to ~300μs, to deal with more/newer variants
- Add error checking (especially for the length) in the `prerendered` variant

## [0.4.0] - 2020-12-02
### Added
- SK812w support for the `prerendered` variant

### Changed
- Modify `FullDuplex` FIFO handling to be more resilient
- Switch `prerendered` to use the same bit patterns as the normal variant

  This removes the ability to use custom frequencies, but makes the whole code a
  *lot* simpler & more like the normal variant.

## [0.3.0] - 2020-02-09
### Added
- SK6812w support

### Changed
- Switched to a more efficient pattern generation, with 4 spi bits per ws2812
  bit instead of 3
