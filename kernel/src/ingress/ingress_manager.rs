//! IngressManager
//! 
//! All communicated date is run through here, parsed, then executed. 

use crate::ingress::buffer::{Buffer, Type};
use crate::system::notification::BUFF_SIZE;
use heapless::consts::*;
use heapless::spsc::Queue;
use simple_hex::hex_byte_to_byte;
use crate::system::syscall::Syscall;
use core::str::FromStr;

use log::info;

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    /// Waiting for a STX byte, or just received an ETX, or entered an invalid state
    Wait,
    /// Init state, just after receiving and STX
    Init,
    /// Write into an internal buffer for parsing
    Payload,

    /// Parse the application checksum
    ApplicationChecksum,
    /// Store the application in ram
    ApplicationStore,
    ApplicationClearBuffer,

    /// Notification Source - what generated the push notification
    NotificationSource,
    /// Notification title
    NotificationTitle,
    /// Notification body
    NotificationBody,
}

const STX: u8 = 2;
const ETX: u8 = 3;
const PAYLOAD: u8 = 31; // Unit Separator

pub struct IngressManager {
    rb: Queue<u8, U512>,
    state: State,

    hex_chars: [u8; 2],
    hex_idx: usize,

    nsi: [usize; 3],
    nsi_idx: usize,
}

pub enum Event<'a> {
    ApplicationKill,
    ApplicationWrite { bytes: &'a [u8] },
    ApplicationWriteChecksum { checksum: [u8; 2] },
    ApplicationVerify { bytes: &'a [u8] },

    Notification { slice: &'a Buffer, indexes: [usize; 3] },

    Syscall(Syscall),
}

impl IngressManager {

    /// Constructs a new IngressManager
    pub fn new() -> Self {
        IngressManager {
            rb: Queue::new(),
            state: State::Init,
            hex_chars: [0u8; 2],
            hex_idx: 0,
            nsi: [0usize; 3], // notification section pointers
            nsi_idx: 0,
        }
    }

    /// Write data into the internal ring buffer
    /// raw bytes being the core type allows the ingress manager to 
    /// be abstracted over the communication medium,
    /// in theory if we setup usb serial, we could have two ingress managers
    /// working in harmony 
    pub fn write(&mut self, data: &[u8]) {
        for byte in data {
            match self.rb.enqueue(*byte) {
                Ok(_) => {},
                Err(e) => panic!("Ring buffer overflow by {:?} bytes", e)
            }
        }
    }

    /// Processs the internal ringbuffer's bytes and execute if the payload is complete
    pub fn process<'a>(&mut self, buffer: &'a mut Buffer) -> Option<Event<'a>> {
        if !self.rb.is_empty() {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => {
                        if self.state != State::Wait {
                            warn!("Partial buffer detected: {:?}", buffer);
                        }
                        /* Start of packet */
                        self.hex_idx = 0;
                        self.nsi_idx = 0;
                        buffer.clear();
                        self.state = State::Init; // activate processing
                    }
                    ETX => {
                        /* End of packet */
                        /* Finalize messge then reset state machine ready for next msg*/
                        self.state = State::Wait;
                        match buffer.btype {
                            Type::Unknown => {
                                self.state = State::Wait; // if the type cannot be determined abort, and wait until next STX
                            }
                            Type::Application => {
                                return Some(Event::ApplicationVerify { bytes : buffer.as_slice() });
                                
                            },
                            Type::Notification => {
                                info!("Adding notification from: {:?}, with section indexes {:?}", buffer, self.nsi);
                                self.nsi[2] = self.nsi_idx;
                                let nscopy = self.nsi;
                                return Some(Event::Notification { slice: buffer, indexes: nscopy })
                            },
                            Type::Syscall => {
                                info!("Parsing syscall from: {:?}", buffer);
                                match Syscall::from_str(buffer.as_str()) {
                                    Ok(syscall) => return Some(Event::Syscall(syscall)),
                                    Err(e) => error!("Failed to parse syscall {:?}", e),
                                }
                            }
                        }
                    }
                    PAYLOAD => {
                        match buffer.btype {
                            Type::Unknown => {
                                warn!("Dropping buffer of unknown type {:?}", buffer.btype);
                                self.state = State::Wait
                            }
                            Type::Application => {
                                if self.state == State::ApplicationChecksum {
                                    // We've parsed the checksum, now we write the data into ram
                                    self.state = State::ApplicationStore;
                                    return Some(Event::ApplicationWriteChecksum { checksum: [buffer.payload[0], buffer.payload[1]] });
                                } else {
                                    self.state = State::ApplicationChecksum;
                                    // reset before we load the new application
                                    return Some(Event::ApplicationKill);
                                }
                            }
                            Type::Notification => {
                                if self.state == State::NotificationSource { // we've parsed the app source
                                    self.nsi[0] = self.nsi_idx;
                                    self.state = State::NotificationTitle;
                                } else if self.state == State::NotificationTitle { // weve parsed the title
                                    self.nsi[1] = self.nsi_idx;
                                    self.state = State::NotificationBody;
                                } else {
                                    self.state = State::NotificationSource; // new parse
                                }
                            }
                            _ => self.state = State::Payload,
                        }
                    }
                    _ => {
                        /* Run through byte state machine */
                        match self.state {
                            State::Init => {
                                buffer.determine_type(byte);
                                info!("New buffer of type {:?}", buffer.btype);
                                if let Type::Unknown = buffer.btype {
                                    error!("Buffer type is unknown. Going back to wait state.");
                                    self.state = State::Wait 
                                }
                            }
                            State::Payload => {
                                buffer.write(byte);
                            }
                            State::ApplicationChecksum | State::ApplicationStore => {
                                self.hex_chars[self.hex_idx] = byte;
                                self.hex_idx += 1;
                                if self.hex_idx > 1 {
                                    match self.state {
                                        State::ApplicationChecksum => {
                                            match hex_byte_to_byte(self.hex_chars[0], self.hex_chars[1]) {
                                                Ok(byte) => {
                                                    buffer.write(byte);
                                                }
                                                Err(err) => {
                                                    error!("Failed to parse hex bytes to byte {:?}", err);
                                                    self.state = State::Wait; // abort
                                                }
                                            }
                                        }
                                        State::ApplicationStore => {
                                            match hex_byte_to_byte(self.hex_chars[0], self.hex_chars[1]) {
                                                Ok(byte) => {
                                                    if buffer.payload_idx < BUFF_SIZE - 1 {
                                                        buffer.write(byte);
                                                    } else {
                                                        self.state = State::ApplicationClearBuffer;
                                                        buffer.write(byte); // write last byte and send it out for writing to ram
                                                        return Some(Event::ApplicationWrite { bytes: buffer.as_slice() })
                                                    }
                                                }
                                                Err(err) => {
                                                    error!("Failed to parse hex bytes to byte {:?}", err);
                                                    self.state = State::Wait; // abort
                                                }
                                            }
                                        }
                                        _ => unreachable!()
                                    }
                                    self.hex_idx = 0;
                                }
                            }
                            State::ApplicationClearBuffer => {
                                buffer.clear();
                                self.state = State::ApplicationStore;
                            }
                            State::NotificationBody | State::NotificationTitle | State::NotificationSource => {
                                self.nsi_idx += 1;
                                buffer.write(byte);
                            }
                            State::Wait => {
                                // do nothing, useless bytes
                            }
                        }
                    }
                }
            }
        }
        None
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ingress_syscall() {
        let mut buffer = Buffer::default();
        let mut imgr = IngressManager::new();
        let mut data = vec![STX, b'S', PAYLOAD];
        for byte in "T00:00:00".bytes() {
            data.push(byte);
        }
        data.push(ETX);
        imgr.write(&data);
        imgr.process(&mut buffer);

        assert_eq!(imgr.state, State::Wait);
    }
}
