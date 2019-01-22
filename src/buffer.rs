//! Buffer
//! 

use buffer_manager::BUFF_SIZE;

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
    fn default() -> Buffer {
        Buffer {
            btype: Type::Unknown,
            payload: [0u8; BUFF_SIZE],
            payload_idx: 0
        }
    }
}

impl Buffer {
    pub fn new(rx_buffer: [u8; BUFF_SIZE]) -> Self {
        Buffer {
            btype: Type::Unknown,
            payload: rx_buffer,
            payload_idx: 0,
        }
    }

    pub fn get_type(self) -> Type {
        self.btype
    }
}