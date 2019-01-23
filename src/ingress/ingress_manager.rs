

extern crate heapless;
extern crate cortex_m;
extern crate rtfm;

use heapless::spsc::Queue;
use heapless::consts::*;
use crate::ingress::buffer::{Buffer, Type};
use crate::ingress::notification::NotificationManager;

pub const BUFF_SIZE: usize = 256;
pub const BUFF_COUNT: usize = 8;

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Wait, /* Waiting for data */
    Init,
    Type,
    Payload,
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
    
    pub fn process(&mut self, notification_mgr: &mut NotificationManager){
        if !self.rb.is_empty() {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => { /* Start of packet */
                        self.buffer.clear(); 
                        self.state = State::Type; // activate processing
                    }
                    ETX => { /* End of packet */
                        /* Finalize messge then reset state machine ready for next msg*/
                        self.state = State::Wait;
                        match self.buffer.btype {
                            Type::Unknown => panic!("Invalid buffer type in {:?}", self.state),
                            Type::Application => {
                                //TODO signal installed - verify with checksum etc
                            },
                            Type::Notification => {
                                notification_mgr.add(&self.buffer).unwrap();
                            },
                            _ => panic!("Unhandled buffer in {:?}", self.state),
                        }
                    }
                    PAYLOAD => { // state change - how? based on type
                        match self.buffer.btype {
                            Type::Unknown => panic!("Invalid buffer type in {:?}", self.state),
                            Type::Application => {
                                /* Move to new payload processing state, as we will be writing into RAM/ROM */
                                self.state = State::ApplicationStore
                            },
                            _ => self.state = State::Payload,
                        }
                    }
                    _ => {
                        /* Run through byte state machine */
                        match self.state {
                            State::Init => {
                                self.state = State::Type
                            }
                            State::Type => {
                                self.buffer.btype = self.determine_type(byte);
                                match self.buffer.btype {
                                    Type::Unknown => panic!("Invalid buffer type in {:?}", self.state),
                                    Type::Application => {
                                        /* Move to new payload processing state, as we will be writing into RAM/ROM */
                                    },
                                    _ => {} // carry on
                                }
                            }
                            State::Payload => {
                                self.buffer.write(byte);
                            }
                            State::ApplicationStore => {
                                unimplemented!()
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
            b'A' => Type::Application, /* Spotify controls */
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