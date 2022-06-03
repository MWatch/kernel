//! Kernel types
//!
//! Exposes all the types the sdk may want to use, which the kernel provides

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate log;

// pub mod application;
pub mod ingress;
pub mod system;