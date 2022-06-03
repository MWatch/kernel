//! Buffer
//! 
//! A thin abstraction over a static array, with some meta data

use crate::system::notification::BUFF_SIZE;
use core::write;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Type {
    Unknown, /* NULL */
    Notification,
    Syscall,
    Application,
}

#[derive(Copy, Clone)]
pub struct Buffer {
    pub btype: Type,
    pub payload: [u8; BUFF_SIZE],
    pub payload_idx: usize,
}

impl Default for Buffer {
    /// Creates a buffer with size `BUFF_SIZE`
    fn default() -> Buffer {
        Buffer {
            btype: Type::Unknown,
            payload: [0u8; BUFF_SIZE],
            payload_idx: 0,
        }
    }
}

impl Buffer {
    /// creates a buffer from a static array of bytes
    pub fn new(rx_buffer: [u8; BUFF_SIZE]) -> Self {
        Buffer {
            btype: Type::Unknown,
            payload: rx_buffer,
            payload_idx: 0,
        }
    }

    /// returns the type of the buffer, defaults to unknown
    pub fn get_type(&self) -> Type {
        self.btype
    }

    /// Writes a byte into the buffer
    pub fn write(&mut self, byte: u8) {
        self.payload[self.payload_idx] = byte;
        self.payload_idx += 1;
    }

    // Resets the index of the buffer, does not blank the memory
    pub fn clear(&mut self) {
        self.payload_idx = 0;
    }

    /// Buffer as &str
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.payload[0..self.payload_idx]) }
    }
}

impl core::fmt::Debug for Buffer {

    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Buffer<{:?}>[{}] : [", self.btype, self.payload_idx)?;
        for idx in 0..self.payload_idx{
            write!(f, " '{}',", self.payload[idx] as char)?;
        }
        write!(f, " ]")?;
        Ok(())
    }
}

impl core::fmt::Display for Buffer {

    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for idx in 0..self.payload_idx{
            write!(f, "{}", self.payload[idx] as char)?;
        }
        Ok(())
    }
}
