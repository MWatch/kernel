# `mwatch`

> An embedded smartwatch written with Rust, using the RTFM framework for multithreading.

## Features

- Capacitive touch sense input control
- Full 16bit colour OLED
- Installable apps (see [SDK section](#developing-applications-for-the-mwatch) for more info)
- Notification alerts via bluetooth

## Developing applications for the `mwatch`

The `mwatch` provides an SDK for developing applications that can be installed via the mobile app. More info about the SDK can be found in the [SDK repo](https://github.com/MWatch/mwatch-sdk).

<!-- # [Documentation](https://github/MabezDev) -->



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
