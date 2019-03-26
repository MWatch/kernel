//! Application Manager
//!
//! Handles loading and running of custom applications
//!
//! - Load information from the binary
//! - Setup input callbacks from the kernel which then are passed to the application
//! - Start executing
//! 
//! Due to the abstract nature of the `ApplicationManager` it is possible to run more than one simultaneously
//! provided you have the available RAM

use crc::crc32::checksum_ieee;
use crate::types::{Context, ServiceFn, SetupFn, Ssd1351, InputFn, InputEvent};

/// Application manager
pub struct ApplicationManager {
    ram: Ram,
    target_cs: [u8; 4],
    target_cs_idx: usize,
    service_fn: Option<ServiceFn>,
    input_fn: Option<InputFn>,
    status: Status,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    /// The applicaton is running
    Executing,
    /// Failed checksum of ram
    ChecksumFailed,
    /// No application has been loaded
    NoApplication,
    /// The FFI function pointer for service is invalid
    InvalidServiceFn,
    /// The FFI function pointer for input is invalid
    InvalidInputFn,
    /// The application doesnt fit in memory
    NoMemory
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
    
    /// Create a new application manager from a chunk of ram
    pub fn new(ram: Ram) -> Self {
        Self {
            ram: ram,
            target_cs: [0u8; 4],
            target_cs_idx: 0,
            service_fn: None,
            input_fn: None,
            status: Status::default(),
        }
    }

    /// Write a byte into the managers internal ram
    pub fn write_ram_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.ram.write(byte)?;
        Ok(())
    }

    /// Write a checksum byte into the manager internal cs buffer
    pub fn write_checksum_byte(&mut self, byte: u8) -> Result<(), Error> {
        if self.target_cs_idx > self.target_cs.len() {
            Err(Error::NoMemory)
        } else {
            self.target_cs[self.target_cs_idx] = byte;
            self.target_cs_idx += 1;
            Ok(())
        }
    }

    /// Verify the contents of ram using a crc against the checksum
    pub fn verify(&mut self) -> Result<(), Error> {
       let ram_cs = self.ram.cs();
       let digest = ApplicationManager::digest_from_bytes(&self.target_cs); 
        info!("Current Ram Digest: {}, stored ram Digest: {}", ram_cs, digest);
        if digest == ram_cs {
            self.status.is_loaded = true;
            Ok(())
        } else {
            error!("Application checksum failed!");
            Err(Error::ChecksumFailed)
        }
    }

    /// Reconstruct a CRC32 from four bytes
    fn digest_from_bytes(bytes: &[u8]) -> u32 {
        assert_eq!(bytes.len(), 4);
        // bytes arrive in reversed order                
        let digest = ((u32::from(bytes[0])) << 24)
            | ((u32::from(bytes[1])) << 16)
            | ((u32::from(bytes[2])) << 8)
            | (u32::from(bytes[3]));
        digest
    }

    /// Run the application
    pub fn execute(&mut self) -> Result<(), Error> {
        if !self.status.is_loaded {
            return Err(Error::NoApplication);
        }
        let setup_ptr = Self::fn_ptr_from_slice(&self.ram.as_ref()[..4]);
        let service_ptr = Self::fn_ptr_from_slice(&self.ram.as_ref()[4..8]);
        let input_ptr = Self::fn_ptr_from_slice(&self.ram.as_ref()[8..12]);
        let _result = unsafe {
            let setup: SetupFn = ::core::mem::transmute(setup_ptr);
            let service: ServiceFn = ::core::mem::transmute(service_ptr);
            let input: InputFn = ::core::mem::transmute(input_ptr);
            self.service_fn = Some(service);
            self.input_fn = Some(input);
            setup()
        };
        self.status.is_running = true;
        Ok(())
    }


    /// Gives processing time to the application
    pub fn service(&mut self, display: &mut Ssd1351) -> Result<(), Error> {
       if let Some(service_fn) = self.service_fn {
        let mut ctx = Context {
            display: Some(display),
            log: application_logger,
        };
        self.status.service_result = service_fn(&mut ctx);
        Ok(())
       } else {
           Err(Error::InvalidServiceFn)
       }
    }

    /// Gives processing time to input handlers of the function
    pub fn service_input(&mut self, input: InputEvent) -> Result<(), Error> {
       if let Some(input_fn) = self.input_fn {
        let mut ctx = Context {
            // display is only passed in on update, not on input
            display: None,
            log: application_logger,
        };
        let _ = input_fn(&mut ctx, input);
        Ok(())
       } else {
           Err(Error::InvalidInputFn)
       }
    }

    /// Pause the application
    pub fn pause(&mut self) {
        self.status.is_running = false;
    }

    /// Kill the current application and unload from memory
    pub fn kill(&mut self) -> Result<(), Error> {
        self.ram.reset();
        self.target_cs_idx = 0;
        self.status.is_loaded = false;
        self.status.is_running = false;
        self.input_fn = None;
        self.service_fn = None;
        Ok(())
    }

    /// Return the status of the manager
    pub fn status(&self) -> Status {
        self.status
    }

    /// convert 4 byte slice into a const ptr
    fn fn_ptr_from_slice(bytes: &[u8]) -> *const () {
        assert!(bytes.len() == 4);
        let addr = ((u32::from(bytes[3])) << 24)
            | ((u32::from(bytes[2])) << 16)
            | ((u32::from(bytes[1])) << 8)
            | (u32::from(bytes[0]));
        addr as *const ()
    }
}

/// A structure for manipulating application memory
pub struct Ram {
    ram: &'static mut [u8],
    ram_idx: usize,
}

impl Ram {
    /// Create a new Ram instance with the size of the provided buffer
    pub fn new(ram: &'static mut [u8]) -> Self {
        // wipe the buffer initially
        for byte in ram.iter_mut() {
            *byte = 0u8;
        }
        Self {
            ram,
            ram_idx: 0,
        }
    }

    /// Write a byte into Ram
    pub fn write(&mut self, byte: u8) -> Result<(), Error> {
        if self.ram_idx > self.ram.len() {
            Err(Error::NoMemory)
        } else {
            self.ram[self.ram_idx] = byte;
            self.ram_idx += 1;
            Ok(())
        }
    }

    /// ieee crc32 of the ram buffer
    pub fn cs(&self) -> u32 {
        checksum_ieee(&self.ram[..self.ram_idx])
    }

    /// Reset ram
    pub fn reset(&mut self) {
        self.ram_idx = 0;
        self.wipe();
    }

    /// Zero the internal buffer
    fn wipe(&mut self) {
        for i in 0..self.ram_idx {
            self.ram[i] = 0u8;
        }
    }

    /// Get an immutable reference to the internal ram buffer
    pub fn as_ref(&self) -> &[u8] {
        &self.ram
    }
}

/// FFI for application debugging
extern "C" fn application_logger(string: &str) -> i32 {
    info!("{}", string);
    0
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn checksum_parsing_works() {
        assert_eq!(ApplicationManager::digest_from_bytes(&[35, 98, 167, 98]), 0x2362A762);
    }
}