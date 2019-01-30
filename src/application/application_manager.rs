//! Application Manager
//!
//! Handles loading and running of custom applications
//!
//! - Load information from the binary
//! - Start executing
//! - Setup input callbacks from the kernel which then are passed to the application
//!

use crc::crc32::checksum_ieee;
use mwatch_kernel_api::{Context, ServiceFn, SetupFn, Ssd1351};

pub struct ApplicationManager {
    ram: Ram,
    target_cs: [u8; 4],
    target_cs_idx: usize,
    service_fn: Option<ServiceFn>,
    status: Status,
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Executing,
    ChecksumFailed,
    NoApplication,
    InvalidServiceFn,
}

#[derive(Debug, Copy, Clone)]
pub struct Status {
    pub is_loaded: bool,
    pub is_running: bool,
    pub ram_used: usize,
    pub service_result: i32,
}

impl Default for Status {
    fn default() -> Status {
        Status {
            is_loaded: false,
            is_running: false,
            service_result: -1,
            ram_used: 0,
        }
    }
}

impl ApplicationManager {
    pub fn new(ram: &'static mut [u8]) -> Self {
        Self {
            ram: Ram::new(ram),
            target_cs: [0u8; 4],
            target_cs_idx: 0,
            service_fn: None,
            status: Status::default(),
        }
    }

    pub fn write_ram_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.ram.write(byte)?;
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
        trace!("{:?}", self.ram);
        info!("Digest: {}", digest);
        let ram_cs = self.ram.cs();
        info!("Current Ram Digest: {}", ram_cs);
        if digest == ram_cs {
            self.status.is_loaded = true;
            Ok(())
        } else {
            error!("Application checksum failed!");
            Err(Error::ChecksumFailed)
        }
    }

    pub fn execute(&mut self) -> Result<(), Error> {
        if !self.status.is_loaded {
            return Err(Error::NoApplication);
        }
        let setup_ptr = Self::fn_ptr_from_slice(&self.ram.as_ref()[..4]);
        let service_ptr = Self::fn_ptr_from_slice(&self.ram.as_ref()[4..8]);
        let _result = unsafe {
            let setup: SetupFn = ::core::mem::transmute(setup_ptr);
            let service: ServiceFn = ::core::mem::transmute(service_ptr);
            self.service_fn = Some(service);
            setup()
        };
        self.status.is_running = true;
        Ok(())
    }


    /// Gives processing time to the application
    pub fn service(&mut self, display: &mut Ssd1351) -> Result<(), Error> {
       if let Some(service_fn) = self.service_fn {
        let mut ctx = Context {
            display: display,
        };
        self.status.service_result = service_fn(&mut ctx);
        Ok(())
       } else {
           Err(Error::InvalidServiceFn)
       }
    }

    pub fn pause(&mut self) {
        self.status.is_running = false;
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        self.ram.reset();
        self.target_cs_idx = 0;
        self.status.is_loaded = false;
        self.status.is_running = false;
        Ok(())
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// convert 4 byte slice into a const ptr
    fn fn_ptr_from_slice(bytes: &[u8]) -> *const () {
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

/// A structure for manipulating application memory
pub struct Ram {
    ram: &'static mut [u8],
    ram_idx: usize,
}

impl Ram {
    /// Create a new Ram instance with the size of the provided buffer
    pub fn new(ram: &'static mut [u8]) -> Self {
        Self {
            ram: ram,
            ram_idx: 0,
        }
    }

    /// Write a byte into Ram
    pub fn write(&mut self, byte: u8) -> Result<(), Error> {
        self.ram[self.ram_idx] = byte;
        self.ram_idx += 1;
        Ok(())
    }

    /// ieee crc32 of the ram buffer
    pub fn cs(&self) -> u32 {
        checksum_ieee(&self.ram[..self.ram_idx])
    }

    // Reset ram
    pub fn reset(&mut self) {
        self.ram_idx = 0;
    }

    pub fn as_ref(&self) -> &[u8] {
        &self.ram
    }
}

impl core::fmt::Debug for Ram {

    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Ram : [")?;
        for idx in 0..self.ram_idx {
            write!(f, " {},", self.ram[idx] as char)?;
        }
        write!(f, " ]")?;
        Ok(())
    }
}
