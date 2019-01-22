

extern crate heapless;
extern crate cortex_m;
extern crate rtfm;

use heapless::spsc::Queue;
use heapless::consts::*;
use notification::Notification;

pub const BUFF_SIZE: usize = 256;
pub const MSG_COUNT: usize = 8;

/// Allows the 
pub trait BufferHandler {
    fn write(byte: u8);
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Type {
    Unknown, /* NULL */
    Notification,
    Weather,
    Date,
    Music,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Wait, /* Waiting for data */
    Init,
    Type,
    Payload,
}

const STX: u8 = 2;
const ETX: u8 = 3;
const DELIM: u8 = 31; // Unit Separator

pub struct Buffer {
    pub btype: Type,
    pub payload: [u8; BUFF_SIZE],
    pub payload_idx: usize,
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

pub struct BufferManager {
    msg_pool : [Buffer; MSG_COUNT],
    rb: &'static mut Queue<u8, U256>,
    msg_state: State,
    msg_idx : usize,
}

impl BufferManager 
{
    pub fn new(msgs: [Buffer; MSG_COUNT], ring_t: &'static mut Queue<u8, U256>) -> Self {
        BufferManager {
            msg_pool: msgs,
            rb: ring_t,
            msg_state: State::Init,
            msg_idx: 0,
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

    pub fn process(&mut self){
        if !self.rb.is_empty() {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => { /* Start of packet */
                        self.msg_state = State::Init; // activate processing
                        let mut msg = &mut self.msg_pool[self.msg_idx];
                        msg.payload_idx = 0; // if we are reusing buffer - set the index back to zero 
                    }
                    ETX => { /* End of packet */
                        /* Finalize messge then reset state machine ready for next msg*/
                        //TODO pop this address from the queue instead of directly working with the array!
                        // let msg = &self.msg_pool[self.msg_idx];
                        // let notification: Notification = msg.into();
                        self.msg_state = State::Wait;
                        self.msg_idx += 1;
                        if self.msg_count() + 1 > self.msg_pool.len() {
                            /* buffer is full, wrap around */        
                            self.msg_idx = 0;
                        }
                    }
                    DELIM => { // state change - how? based on type
                        self.msg_state = State::Payload;
                    }
                    _ => {
                        /* Run through Msg state machine */
                        match self.msg_state {
                            State::Init => {
                                // if msg_idx + 1 > msgs.len(), cant go
                                self.msg_state = State::Type;
                            }
                            State::Type => {
                                self.determine_type(byte);
                                
                            }
                            State::Payload => {
                                let mut msg = &mut self.msg_pool[self.msg_idx];
                                msg.payload[msg.payload_idx] = byte;
                                msg.payload_idx += 1;
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

    fn determine_type(&mut self, type_byte: u8){
        self.msg_pool[self.msg_idx].btype = match type_byte {
            b'N' => Type::Notification, /* NOTIFICATION i.e FB Msg */
            b'W' => Type::Weather, /* Weather packet */
            b'D' => Type::Date,   /* Date packet */
            b'M' => Type::Music, /* Spotify controls */
            _ => Type::Unknown
        }
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
    pub fn peek_message<F>(&mut self, index: usize, f: F)
    where F: FnOnce(&Buffer) {
        let msg = &self.msg_pool[index];
        f(&msg);
    }

    pub fn msg_count(&self) -> usize {
        self.msg_idx
    }
    
}