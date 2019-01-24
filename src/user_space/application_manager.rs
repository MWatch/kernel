//! Application Manager
//! 
//! Handles loading and running of applications

pub struct ApplicationManager {
    ram: &'static [u8]
}

impl ApplicationManager {

    pub fn new(ram: &'static [u8]) -> Self {
        Self {
            ram: ram
        }
    }
}