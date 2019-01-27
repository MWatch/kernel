//! Buffer

use crate::ingress::ingress_manager::BUFF_SIZE;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Type {
    Unknown, /* NULL */
    Notification,
    Weather,
    Date,
    Music,
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
}
