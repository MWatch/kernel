//! Kernel types
//!
//! Exposes all the types the sdk may want to use, which the kernel provides

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[macro_use]
extern crate log;

pub mod application;
pub mod ingress;
pub mod system;

/// The core types of the hardware and api
pub use system::types as types;