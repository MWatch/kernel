

extern crate heapless;
extern crate cortex_m;
extern crate rtfm;

use heapless::spsc::Queue;
use heapless::consts::*;
use crate::ingress::buffer::{Buffer, Type};
use crate::ingress::notification::NotificationManager;
use crate::kernel_api::application_manager::ApplicationManager;
use simple_hex::hex_byte_to_byte;

pub const BUFF_SIZE: usize = 256;
pub const BUFF_COUNT: usize = 8;

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
    rb: &'static mut Queue<u8, U256>,
    state: State,
}

impl IngressManager
{
    pub fn new(ring: &'static mut Queue<u8, U256>) -> Self {
        IngressManager {
            buffer: Buffer::default(),
            rb: ring,
            state: State::Init,
        }
    }

    
    pub fn write(&mut self, data: &[u8]){
        for byte in data {
            // this is safe because we are only storing bytes, which do not need destructors called on them
            unsafe { self.rb.enqueue_unchecked(*byte); } // although we wont know if we have overwritten previous data
        }
    }
    
    pub fn process(&mut self, notification_mgr: &mut NotificationManager, amng: &mut ApplicationManager){
        let mut hex_chars = [0u8; 2];
        let mut hex_idx = 0;
        if !self.rb.is_empty() && self.rb.len() > 2 {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => { /* Start of packet */
                        self.buffer.clear(); 
                        self.state = State::Init; // activate processing
                    }
                    ETX => { /* End of packet */
                        /* Finalize messge then reset state machine ready for next msg*/
                        self.state = State::Wait;
                        match self.buffer.btype {
                            Type::Unknown => self.state = State::Wait, // if the type cannot be determined abort, and wait until next STX
                            Type::Application => {
                                match amng.verify(){
                                    Ok(_) => amng.execute().unwrap(),
                                    Err(e) => panic!("Failed to execute application, {:?}", e)
                                }
                            },
                            Type::Notification => {
                                notification_mgr.add(&self.buffer).unwrap();
                            },
                            _ => panic!("Unhandled buffer in {:?}", self.state),
                        }
                    }
                    PAYLOAD => { // state change - how? based on type
                        match self.buffer.btype {
                            Type::Unknown => self.state = State::Wait,
                            Type::Application => {
                                if self.state == State::ApplicationChecksum {
                                    // We've parsed the checksum, now we write the data into ram
                                    self.state = State::ApplicationStore
                                } else {
                                    amng.prepare_load().unwrap();
                                    // parse the checksum
                                    self.state = State::ApplicationChecksum;
                                }
                            },
                            _ => self.state = State::Payload,
                        }
                    }
                    _ => {
                        /* Run through byte state machine */
                        match self.state {
                            State::Init => {
                                self.buffer.btype = self.determine_type(byte);
                                match self.buffer.btype {
                                    Type::Unknown => self.state = State::Wait,
                                    _ => {} // carry on
                                }
                            }
                            State::Payload => {
                                self.buffer.write(byte);
                            }
                            State::ApplicationChecksum => {
                                hex_chars[hex_idx] = byte;
                                hex_idx += 1;
                                if hex_idx > 1 {
                                    amng.write_checksum_byte(hex_byte_to_byte(hex_chars[0], hex_chars[1]).unwrap()).unwrap();
                                    hex_idx = 0;
                                }
                            }
                            State::ApplicationStore => {
                                hex_chars[hex_idx] = byte;
                                hex_idx += 1;
                                if hex_idx > 1 {
                                    amng.write_byte(hex_byte_to_byte(hex_chars[0], hex_chars[1]).unwrap()).unwrap();
                                    hex_idx = 0;
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
            b'W' => Type::Weather, /* Weather packet */
            b'D' => Type::Date,   /* Date packet */
            b'M' => Type::Music, /* Spotify controls */
            b'A' => Type::Application, /* Load Application */
            _ => Type::Unknown
        };
        self.buffer.btype
    }

    pub fn print_rb(&mut self, itm: &mut cortex_m::peripheral::itm::Stim){
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