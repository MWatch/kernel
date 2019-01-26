//! Application Manager
//!
//! Handles loading and running of custom applications
//!
//! - Load information from the binary
//! - Start executing
//! - Setup input callbacks from the kernel which then are passed to the application
//!

use crc::crc32::checksum_ieee;
use mwatch_kernel_api::{draw_pixel, Table};

pub struct ApplicationManager {
    ram: &'static mut [u8],
    ram_idx: usize,
    target_cs: [u8; 4],
    target_cs_idx: usize,
    status: Status,
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Executing,
    ChecksumFailed,
}

#[derive(Debug, Copy, Clone)]
pub struct Status {
    pub is_loaded: bool,
    pub is_running: bool,
}

impl Default for Status {
    fn default() -> Status {
        Status {
            is_loaded: false,
            is_running: false,
        }
    }
}

impl ApplicationManager {
    pub fn new(ram: &'static mut [u8]) -> Self {
        Self {
            ram: ram,
            ram_idx: 0,
            target_cs: [0u8; 4],
            target_cs_idx: 0,
            status: Status::default(),
        }
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.ram[self.ram_idx] = byte;
        self.ram_idx += 1;
        Ok(())
    }

    pub fn write_checksum_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.target_cs[self.target_cs_idx] = byte;
        self.target_cs_idx += 1;
        Ok(())
    }

    pub fn verify(&mut self) -> Result<(), Error> {
        // reversed order becaused the bytes arrive in the reversed order
        let digest = ((self.target_cs[0] as u32) << 24)
            | ((self.target_cs[1] as u32) << 16)
            | ((self.target_cs[2] as u32) << 8)
            | ((self.target_cs[3] as u32) << 0);
        let self_cs = checksum_ieee(&self.ram[..self.ram_idx]);
        if digest == self_cs {
            self.status.is_loaded = true;
            Ok(())
        } else {
            Err(Error::ChecksumFailed)
        }
    }

    pub fn execute(&mut self) -> Result<(), Error> {
        let setup_ptr = Self::fn_ptr_from_slice(&mut self.ram[..4]);
        let _update_ptr = Self::fn_ptr_from_slice(&mut self.ram[4..8]);
        let _result = unsafe {
            let t = Table {
                context: core::mem::uninitialized(),
                draw_pixel: draw_pixel,
                register_input: core::mem::uninitialized(),
            };
            let setup: extern "C" fn(*const Table) -> u32 = ::core::mem::transmute(setup_ptr);
            let result = setup(&t);
            result
        };
        self.status.is_running = true;
        Ok(())
    }

    pub fn pause(&mut self) {
        self.status.is_running = false;
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        self.ram_idx = 0;
        self.target_cs_idx = 0;
        self.status.is_loaded = false;
        self.status.is_running = false;
        Ok(())
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// convert 4 byte slice into a const ptr
    fn fn_ptr_from_slice(bytes: &mut [u8]) -> *const () {
        assert!(bytes.len() == 4);
        let addr = ((bytes[3] as u32) << 24)
            | ((bytes[2] as u32) << 16)
            | ((bytes[1] as u32) << 8)
            | ((bytes[0] as u32) << 0);
        addr as *const ()
    }

    //TODO Expose an interface like below to allow the kernel to set input events
    // pub fn update_input(someEnum: InputVariant)

    //TODO call the relevant input handlers when the kernel notifies us of a change
}
