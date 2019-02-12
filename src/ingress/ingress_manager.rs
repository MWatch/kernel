extern crate cortex_m;
extern crate heapless;
extern crate rtfm;

use crate::ingress::buffer::{Buffer, Type};
use heapless::consts::*;
use heapless::spsc::Queue;
use simple_hex::hex_byte_to_byte;
use crate::system::system::System;
use crate::system::syscall::Syscall;
use core::str::FromStr;

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Wait, /* Waiting for data */
    Init,
    Payload,
    ApplicationChecksum,
    ApplicationStore,
}

const STX: u8 = 2;
const ETX: u8 = 3;
const PAYLOAD: u8 = 31; // Unit Separator

pub struct IngressManager {
    buffer: Buffer,
    rb: &'static mut Queue<u8, U512>,
    state: State,
    hex_chars: [u8; 2],
    hex_idx: usize,
}

impl IngressManager {
    pub fn new(ring: &'static mut Queue<u8, U512>) -> Self {
        IngressManager {
            buffer: Buffer::default(),
            rb: ring,
            state: State::Init,
            hex_chars: [0u8; 2],
            hex_idx: 0,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        for byte in data {
            match self.rb.enqueue(*byte) {
                Ok(_) => {},
                Err(e) => panic!("Ring buffer overflow by {:?} bytes", e)
            }
        }
    }

    pub fn process(
        &mut self,
        system: &mut System
    ) {
        if !self.rb.is_empty() {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => {
                        if self.state != State::Wait {
                            warn!("Partial buffer detected: {:?}", self.buffer);
                        }
                        /* Start of packet */
                        self.hex_idx = 0;
                        self.buffer.clear();
                        self.state = State::Init; // activate processing
                    }
                    ETX => {
                        /* End of packet */
                        /* Finalize messge then reset state machine ready for next msg*/
                        self.state = State::Wait;
                        match self.buffer.btype {
                            Type::Unknown => self.state = State::Wait, // if the type cannot be determined abort, and wait until next STX
                            Type::Application => {
                                match system.am().verify() {
                                    Ok(_) =>
                                    {
                                        
                                    }
                                    Err(e) => panic!("{:?} || AMNG: {:?}", e, system.am().status()),
                                }
                            }
                            Type::Notification => {
                                info!("Adding notification from: {:?}", self.buffer);
                                system.nm().add(&self.buffer).unwrap();
                            },
                            Type::Syscall => {
                                info!("Parsing syscall from: {:?}", self.buffer);
                                let syscall = Syscall::from_str(self.buffer.clone().as_str()).unwrap();
                                // let syscall = Syscall::from_str("T21:21:11").unwrap();
                                syscall.execute().unwrap();
                            }
                            _ => panic!("Unhandled buffer in {:?}", self.state),
                        }
                    }
                    PAYLOAD => {
                        match self.buffer.btype {
                            Type::Unknown => {
                                warn!("Dropping buffer of unknown type {:?}", self.buffer.btype);
                                self.state = State::Wait
                            }
                            Type::Application => {
                                if self.state == State::ApplicationChecksum {
                                    // We've parsed the checksum, now we write the data into ram
                                    self.state = State::ApplicationStore
                                } else {
                                    // reset before we load the new application
                                    system.am().stop().unwrap();
                                    // parse the checksum
                                    self.state = State::ApplicationChecksum;
                                }
                            }
                            _ => self.state = State::Payload,
                        }
                    }
                    _ => {
                        /* Run through byte state machine */
                        match self.state {
                            State::Init => {
                                self.buffer.btype = self.determine_type(byte);
                                info!("New buffer of type {:?}", self.buffer.btype);
                                match self.buffer.btype {
                                    Type::Unknown => self.state = State::Wait,
                                    _ => {} // carry on
                                }
                            }
                            State::Payload => {
                                self.buffer.write(byte);
                            }
                            State::ApplicationChecksum => {
                                self.hex_chars[self.hex_idx] = byte;
                                self.hex_idx += 1;
                                if self.hex_idx > 1 {
                                    system.am().write_checksum_byte(
                                        hex_byte_to_byte(self.hex_chars[0], self.hex_chars[1]).unwrap(),
                                    )
                                    .unwrap();
                                    self.hex_idx = 0;
                                }
                            }
                            State::ApplicationStore => {
                                self.hex_chars[self.hex_idx] = byte;
                                self.hex_idx += 1;
                                if self.hex_idx > 1 {
                                    system.am().write_ram_byte(
                                        hex_byte_to_byte(self.hex_chars[0], self.hex_chars[1]).unwrap(),
                                    )
                                    .unwrap();
                                    self.hex_idx = 0;
                                }
                            }
                            State::Wait => {
                                // do nothing, useless bytes
                            }
                        }
                    }
                }
            }
        }
    }

    fn determine_type(&mut self, type_byte: u8) -> Type {
        self.buffer.btype = match type_byte {
            b'N' => Type::Notification, /* NOTIFICATION i.e FB Msg */
            b'S' => Type::Syscall,
            b'A' => Type::Application,  /* Load Application */
            _ => Type::Unknown,
        };
        self.buffer.btype
    }

    pub fn print_rb(&mut self, itm: &mut cortex_m::peripheral::itm::Stim) {
        if self.rb.is_empty() {
            // iprintln!(itm, "RB is Empty!");
        } else {
            iprintln!(itm, "RB Contents: ");
            while let Some(byte) = self.rb.dequeue() {
                iprint!(itm, "{}", byte as char);
            }
            iprintln!(itm, "");
        }
    }
}
