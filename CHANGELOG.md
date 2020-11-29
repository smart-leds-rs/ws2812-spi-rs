# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Modify `FullDuplex` FIFO handling to be more resilient

## [0.3.0] - 2020-02-09
### Added
- SK6812w support

### Changed
- Switched to a more efficient pattern generation, with 4 spi bits per ws2812
  bit instead of 3
