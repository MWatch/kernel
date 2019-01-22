

extern crate heapless;
extern crate cortex_m;
extern crate rtfm;

use heapless::spsc::Queue;
use heapless::consts::*;
use buffer::{Buffer, Type};
// use notification::Notification;

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

pub struct BufferManager<'a> {
    pool: &'a mut [Buffer; BUFF_COUNT],
    rb: &'static mut Queue<u8, U256>,
    state: State,
    buffer_idx : usize,
}

impl<'a> BufferManager<'a>
{
    pub fn new(msgs: &'a mut [Buffer; BUFF_COUNT], ring: &'static mut Queue<u8, U256>) -> Self {
        BufferManager {
            pool: msgs,
            rb: ring,
            state: State::Init,
            buffer_idx: 0,
        }
    }

    /* 

     */
    pub fn write(&mut self, data: &[u8]){
        for byte in data {
            // this is safe because we are only storing bytes, which do not need destructors called on them
            unsafe { self.rb.enqueue_unchecked(*byte); } // although we wont know if we have overwritten previous data
        }
    }
    /* WHAT HAPPENS IF THE BUFFER MOVES FROM UNDER NEATH THE NOTIFICATION STRUCT>???? */
    pub fn process(&mut self){
        if !self.rb.is_empty() {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => { /* Start of packet */
                        self.state = State::Init; // activate processing
                        let mut msg = self.current_buffer_mut();
                        msg.payload_idx = 0; // if we are reusing buffer - set the index back to zero 
                    }
                    ETX => { /* End of packet */
                        /* Finalize messge then reset state machine ready for next msg*/
                        self.state = State::Wait;
                        self.buffer_idx += 1;
                        if self.used_count() + 1 > self.pool.len() {
                            /* buffer is full, wrap around */        
                            self.buffer_idx = 0;
                        }
                    }
                    PAYLOAD => { // state change - how? based on type
                        match self.determine_type(byte) {
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
                                // if buffer_idx + 1 > msgs.len(), cant go
                                self.state = State::Type;
                            }
                            State::Type => {
                                match self.determine_type(byte) {
                                    Type::Unknown => panic!("Invalid buffer type in {:?}", self.state),
                                    Type::Application => {
                                        /* Move to new payload processing state, as we will be writing into RAM/ROM */
                                    },
                                    _ => {} // carry on
                                }
                            }
                            State::Payload => {
                                let mut msg = self.current_buffer_mut();
                                msg.payload[msg.payload_idx] = byte;
                                msg.payload_idx += 1;
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

    fn current_buffer_mut (&mut self) -> &mut Buffer {
        &mut self.pool[self.buffer_idx]
    } 

    fn determine_type(&mut self, type_byte: u8) -> Type {
        self.pool[self.buffer_idx].btype = match type_byte {
            b'N' => Type::Notification, /* NOTIFICATION i.e FB Msg */
            b'W' => Type::Weather, /* Weather packet */
            b'D' => Type::Date,   /* Date packet */
            b'M' => Type::Music, /* Spotify controls */
            b'A' => Type::Application, /* Spotify controls */
            _ => Type::Unknown
        };
        self.pool[self.buffer_idx].btype
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

    /// takes a closure to execute on the buffer
    pub fn peek_buffer<F>(&mut self, index: usize, f: F)
    where F: FnOnce(&Buffer) {
        let buffer = &self.pool[index];
        f(&buffer);
    }

    pub fn used_count(&self) -> usize {
        self.buffer_idx
    }
    
}