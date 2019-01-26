//! Application Manager
//! 
//! Handles loading and running of custom applications
//! 
//! - Load information from the binary
//! - Start executing
//! - Setup input callbacks from the kernel which then are passed to the application
//! 

use mwatch_kernel_api::{Table, draw_pixel};
use crc::crc32::checksum_ieee;

pub struct ApplicationManager {
    ram: &'static mut [u8],
    ram_idx: usize,
    target_cs: [u8; 4],
    target_cs_idx: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Executing,
    ChecksumFailed,
}

impl ApplicationManager {

    pub fn new(ram: &'static mut [u8]) -> Self {
        Self {
            ram: ram,
            ram_idx: 0,
            target_cs: [0u8; 4],
            target_cs_idx: 0,
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

    pub fn verify(&self) -> Result<(), Error> {
        // reversed order becaused the bytes arrive in the reversed order
        let digest = ((self.target_cs[0] as u32) << 24)
                | ((self.target_cs[1] as u32) << 16)
                | ((self.target_cs[2] as u32) << 8)
                | ((self.target_cs[3] as u32) << 0);
        let self_cs = checksum_ieee(&self.ram[..self.ram_idx]);
        if digest == self_cs {
            Ok(())
        } else {
            Err(Error::ChecksumFailed)
        }
    }

    pub fn execute(&mut self) -> Result<(), Error> {
        // convert 4 bytes into a ffi function pointer
        let setup_addr = ((self.ram[3] as u32) << 24)
                | ((self.ram[2] as u32) << 16)
                | ((self.ram[1] as u32) << 8)
                | ((self.ram[0] as u32) << 0);
        let setup_ptr = setup_addr as *const ();
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
        Ok(())
    }

    pub fn stop() {

    }

    pub fn prepare_load(&mut self) -> Result<(), Error>{
        self.ram_idx = 0;
        self.target_cs_idx = 0;
        Ok(())
    }

    //TODO Expose an interface like below to allow the kernel to set input events
    // pub fn update_input(someEnum: InputVariant)

    //TODO call the relevant input handlers when the kernel notifies us of a change
}