//! Application Manager
//! 
//! Handles loading and running of custom applications
//! 
//! - Load information from the binary
//! - Start executing
//! - Setup input callbacks from the kernel which then are passed to the application

pub struct ApplicationManager {
    ram: &'static [u8],// USER buffer instead?
}

pub enum Error {
    Executing,
    ChecksumFailed,
}

impl ApplicationManager {

    pub fn new(ram: &'static [u8]) -> Self {
        Self {
            ram: ram
        }
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn execute() -> Result<(), Error> {
        unimplemented!()
    }

    pub fn stop() {

    }

    pub fn load() -> Result<(), Error>{
        unimplemented!()
    }

    //TODO Expose an interface like below to allow the kernel to set input events
    // pub fn update_input(someEnum: InputVariant)

    //TODO call the relevant input handlers when the kernel notifies us of a change
}