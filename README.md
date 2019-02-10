# `mwatch`

> An embedded smartwatch written with Rust, using the RTFM framework for multithreading.

![Logo](https://i.imgur.com/BYfEjaX.jpg)

## Features

- Capacitive touch sense inputs - with support for multitouch gestures
- Full 16bit colour ssd1351 OLED
- Runtime installable apps (see [SDK section](#developing-applications-for-the-mwatch) for more info)
- Real time clock
- Notification alerts via bluetooth
- Buildable on stable rust 2018

## Developing applications for the `MWatch`

The `mwatch` provides an SDK for developing applications that can be installed at runtime via the [mwatch-send-tool](https://github.com/MWatch/mwatch-protocol-spoofer). More info about the SDK can be found in the [SDK repo](https://github.com/MWatch/mwatch-sdk).

## System Architecture

### Overview

The `MWatch` tries to follow a modern computer operating system, complete with a kernel, a built in window/display manager along with a user space api for developing user space apps on the watch, as well as some builtin applications.



### Window/Display Manager

The window manager handles input and rendering of states/applications inside the watch, all states **must** implement the `State` trait to run but can optionally implement other helper traits which allows the window manager to enable more functionality for a state.

### Kernel API

The kernel among otherthings provides an API for the sdk to interact with, this is providided by `lib.rs` in the kernel crate. This allows the SDK to properly depend on the kernel, meaning if the kernel implements a new API all that is required for the sdk to use it is to bump the version of the kernel. Currently there is no checking done on the binary the sdk produces to make sure it is compatible with the current running kernel.

### Protocol

The `MWatch` has a builtin bluetooth module connected to `usart2`. Through this serial interface we can recieve `Notifications`, `Applications` and more. The basic procotol looks like this

```
STX -> TYPE -> (DELIM:DATA)* -> ETX
                ^^^^^^^^^^
                Can repeat many times based on the type 
```

In english, start byte followed by a type followed by any amount of delimiters followed by data finally ETX.
All data **must** be valid ascii, to send binary data you must convert to hex nibbles first. See the application_manager for more info.

### Input management

The TSC (touch sense controller) builtin to the `mwatch` provides three inputs. The kernel polls these inputs and multiplexes there results to produce a final output. For example touching the middle button produces a middle output, touching the left and right at the same time produces a dual-click output.

## [Documentation](https://docs.rs/mwatch_kernel/latest/mwatch_kernel/)

## Building

- Requires the `thumbv7em-none-eabi` target to be installed, use `rustup target add thumbv7em-none-eabi` to do so.
- Requires `cargo-binutils` for extra features, such as generating a stripped binary. Note: The `llvm-tools-preview` component must be installed with `rustup component add llvm-tools-preview` for it to work.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

**Copyright Scott Mabin 2019**
