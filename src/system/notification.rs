//! Notification
//! 
//! Push notification parsing

use crate::ingress::buffer::Buffer;

pub const BUFF_SIZE: usize = 256;
pub const BUFF_COUNT: usize = 8;

#[derive(Copy, Clone)]
pub struct Notification {
    //TODO parsing
    // app_name_idx: usize,
    // title_idx: usize,
    // text_idx: usize,
    inner: Buffer,
}

impl Notification {
    pub const fn default() -> Notification {
        Notification {
            // app_name_idx: 0,
            // title_idx: 0,
            // text_idx: 0,
            inner: Buffer {
                btype: crate::ingress::buffer::Type::Unknown,
                payload: [0u8; BUFF_SIZE],
                payload_idx: 0,
            },
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.inner.payload[..self.inner.payload_idx]
    }

    pub fn parse_buffer(&mut self, buffer: &Buffer) -> Result<(), NotificationError> {
        //TODO actual parsing, using nom?
        self.inner = buffer.clone();
        Ok(())
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NotificationError {
    Parsing,
}

pub struct NotificationManager {
    pool: [Notification; BUFF_COUNT],
    idx: usize,
}

impl NotificationManager {
    pub fn new() -> NotificationManager {
        NotificationManager {
            pool: [Notification::default(); BUFF_COUNT],
            idx: 0,
        }
    }

    /// takes a closure to execute on the buffer
    pub fn peek_notification<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&Notification),
    {
        let notification = &self.pool[index];
        f(&notification);
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    // Parses a buffer for notification info, copying into the pool
    pub fn add(&mut self, buffer: &Buffer) -> Result<(), NotificationError> {
        self.pool[self.idx].parse_buffer(buffer)?;

        self.idx += 1;
        if self.idx + 1 > self.pool.len() {
            // TODO impl a cirucular buffer that track head and tail
            /* buffer is full, wrap around */

            self.idx = 0;
        }
        Ok(())
    }
}

// TODO testing of parsing
