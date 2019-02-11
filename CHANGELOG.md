# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.9.0]

- Added a proper window/display manager, this manager can handle input and render any structs that implement the `State` traits found in `wm.rs`
- Reduce sysclk down to 16mhz, and pclks down to 8mhz
- Fixed a bug where bytes were being lost while installing an application, this was due to partial fill of the tempory buffer, that would then go out of scope.
- Updated the readme for include more information about the overall structure of the project.

## [v0.8.1]

- Added an proper input manager, with multiplexing of inputs, with a API for applications
- Added a itm tracing using `cortex-m-log`

## [v0.8.0]

- Moved kernel objects into the kernel library crate, allowing the sdk to depend on it
- Now using software `tasks` to service the running application / states, this is an easy way to offload non time critical operations out of interrupt handlers.

## [v0.7.0]

- Added support for loading applications created by [the sdk](https://github.com/MWatch/sdk), which can be sent through the [protocol spoofer](https://github.com/MWatch/mwatch-protocol-spoofer)
- Updated ssd1351, which now includes buffered support. Using the 32k frame buffer for more efficient display operations

## [v0.6.0]

- Converted 'message manager' into ingress manager
- New `NotificationManager` which handles push notification over bluetooth

## [v0.5.0]

- Moved to RTFM v4
- Added CPU usage monitor
- More efficient TSC aquisition with a hardware timer

## [v0.4.0]

- Now on a PCB, see [the hardware repo](https://github.com/MWatch/hardware)
- Added support for reading SoC with the max17048 driver
- Added bluetooth serial support with hm11 driver

## [v0.3.0]

- Added touch sensor support into the application
- Basic state management based on TSC inputs
- Added Makefile for quick operation of hot functions
- Added new logo

## v0.2.0

- Switched to stm32l432kc, this is a more powerful, lower power device
- Added driver for the ssd1351 display
- Added real time clock support
- Switched to lld as the default linker


## v0.1.0

- Basic RTFM application running on a stm32f103 with DMA serial working
- Usart time out captures partial buffers from the DMA
- Simple 'message manager implemented'

- Initial release

[Unreleased]: https://github.com/mwatch/kernel/compare/v0.9.0...HEAD
[v0.1.0]: https://github.com/mwatch/kernel/tree/v0.1.0