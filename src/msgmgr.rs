

extern crate heapless;
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use heapless::RingBuffer;
use heapless::BufferFullError;
use cortex_m::asm;

/* 
    Message is a type
 */

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MessageType {
    Unknown = 0, /* NULL */
    Notification = 78,    /* 'N' as a u8, NOTIFICATION i.e FB Msg */
    Weather = 87,/* 'W' as a u8, Weather packet */
    Date = 68,   /* 'D' as a u8, Date packet */
    Music = 77,  /* 'M' as a u8, Spotify controls */
}

enum MessageState {
    Init,
    Type,
    Title,  /* Optional */
    Payload,
    End
}

const STX: u8 = 2;
const ETX: u8 = 3;
const DELIM: u8 = 31; // Unit Separator

// #[derive(Copy)]
pub struct Message {
    pub msg_type: MessageType,
    pub payload: [u8; 256],
}

impl Message {
    pub fn new(rx_buffers: [u8; 256]) -> Self {
        Message {
            msg_type: MessageType::Unknown,
            payload: rx_buffers,
        }
    }

    pub fn get_type(self) -> MessageType {
        self.msg_type
    }
}

pub struct MessageManager {
    msg_pool : [Message; 8],
    rb: &'static mut RingBuffer<u8, [u8; 128]>,
    msg_state: MessageState,
    current_msg_idx : usize,
}

impl MessageManager 
{
    pub fn new(msgs: [Message; 8], ring_t: &'static mut RingBuffer<u8, [u8; 128]>) -> Self {
        MessageManager {
            msg_pool: msgs,
            rb: ring_t,
            msg_state: MessageState::Init,
            current_msg_idx: 0,
        }
    }

    /* 

     */
    pub fn write(&mut self, data: &[u8]) -> Result<(), BufferFullError>{
        for byte in data {
            self.rb.enqueue(*byte)?;
        }
        Ok(())
    }

    pub fn process(&mut self){
        if self.rb.is_empty() {
            // Nothing todo!
        } else {
            while let Some(byte) = self.rb.dequeue() {
                match byte {
                    STX => { /* Start of packet */
                        self.msg_state = MessageState::Init;
                    }
                    ETX => { /* End of packet */
                        self.msg_state = MessageState::End;
                    }
                    DELIM => { // state change

                    }
                    _ => {
                        /* Run through Msg state machine */
                        match self.msg_state {
                            MessageState::Init => {
                                asm::bkpt();
                                // if current_msg_idx + 1 > msgs.len(), cant go
                                self.msg_state = MessageState::Type;
                            }
                            MessageState::Type => {
                                self.determine_type(byte);
                            }
                            MessageState::Title => {

                            }
                            MessageState::Payload => {
                                
                            }
                            MessageState::End => {
                                /* Finalize messge then reset state machine ready for next msg*/
                                self.msg_state = MessageState::Init;
                                self.current_msg_idx += 1;
                            }
                            _ => {
                                // do nothing, useless bytes
                            }
                        }
                    }
                }
            }
        }
    }

    fn determine_type(&mut self, type_byte: u8){
        self.msg_pool[self.current_msg_idx].msg_type = match type_byte {
            Notification => MessageType::Notification,
            Weather => MessageType::Weather,
            Date => MessageType::Date,   
            Music => MessageType::Music,
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
    pub fn peek_payload<F>(&mut self, index: usize, f: F)
    where F: FnOnce(&[u8]) {
        let msg = &self.msg_pool[index];
        f(&msg.payload);
    }

    
}