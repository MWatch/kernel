

extern crate heapless;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use heapless::RingBuffer;
use heapless::consts::*;

/* 
    Message is a type
 */


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MessageType {
    Unknown, /* NULL */
    Notification,
    Weather,
    Date,
    Music,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum MessageState {
    Wait, /* Waiting for data */
    Init,
    Type,
    Payload,
    // End,
}

const STX: u8 = 2;
const ETX: u8 = 3;
const DELIM: u8 = 31; // Unit Separator

pub struct Message {
    pub msg_type: MessageType,
    pub payload: [u8; 256],
    pub payload_idx: usize,
}

impl Message {
    pub fn new(rx_buffers: [u8; 256]) -> Self {
        Message {
            msg_type: MessageType::Unknown,
            payload: rx_buffers,
            payload_idx: 0,
        }
    }

    pub fn get_type(self) -> MessageType {
        self.msg_type
    }
}

pub struct MessageManager {
    msg_pool : [Message; 8],
    rb: &'static mut RingBuffer<u8, U128>,
    msg_state: MessageState,
    msg_idx : usize,
}

impl MessageManager 
{
    pub fn new(msgs: [Message; 8], ring_t: &'static mut RingBuffer<u8, U128>) -> Self {
        MessageManager {
            msg_pool: msgs,
            rb: ring_t,
            msg_state: MessageState::Init,
            msg_idx: 0,
        }
    }

    /* 

     */
    pub fn write(&mut self, data: &[u8]){
        for byte in data {
            // this is safe because we are only storing bytes, which do not need destructors called on them
            self.rb.enqueue_unchecked(*byte); // although we wont know if we have overwritten previous data
        }
    }

    pub fn process(&mut self){
        if !self.rb.is_empty() {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => { /* Start of packet */
                        self.msg_state = MessageState::Init; // activate processing
                        let mut msg = &mut self.msg_pool[self.msg_idx];
                        msg.payload_idx = 0; // if we are reusing buffer - set the index back to zero 
                    }
                    ETX => { /* End of packet */
                        /* Finalize messge then reset state machine ready for next msg*/
                        self.msg_state = MessageState::Wait;
                        self.msg_idx += 1;
                        if self.msg_count() > self.msg_pool.len() {
                            /* buffer is full, wrap around */        
                            self.msg_idx = 0;
                        }
                    }
                    DELIM => { // state change - how? based on type
                        self.msg_state = MessageState::Payload;
                    }
                    _ => {
                        /* Run through Msg state machine */
                        match self.msg_state {
                            MessageState::Init => {
                                // if msg_idx + 1 > msgs.len(), cant go
                                self.msg_state = MessageState::Type;
                            }
                            MessageState::Type => {
                                self.determine_type(byte);
                            }
                            MessageState::Payload => {
                                let mut msg = &mut self.msg_pool[self.msg_idx];
                                msg.payload[msg.payload_idx] = byte;
                                msg.payload_idx += 1;
                            }
                            MessageState::Wait => {
                                // do nothing, useless bytes
                            }
                        }
                    }
                }
            }
        } 
    }

    fn determine_type(&mut self, type_byte: u8){
        self.msg_pool[self.msg_idx].msg_type = match type_byte {
            78 => MessageType::Notification, /* 'N' as a u8, NOTIFICATION i.e FB Msg */
            87 => MessageType::Weather, /* 'W' as a u8, Weather packet */
            68 => MessageType::Date,   /* 'D' as a u8, Date packet */
            77 => MessageType::Music, /* 'M' as a u8, Spotify controls */
            _ => MessageType::Unknown
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

    // takes a closure to execute on the buffer
    pub fn peek_message<F>(&mut self, index: usize, f: F)
    where F: FnOnce(&Message) {
        let msg = &self.msg_pool[index];
        f(&msg);
    }

    pub fn latest(&self) -> usize {
        self.msg_idx
    }

    pub fn msg_count(&self) -> usize {
        self.msg_idx + 1
    }
    
}