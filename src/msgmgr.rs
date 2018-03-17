

extern crate heapless;
use heapless::RingBuffer;

/* 
    Message is a type
 */

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MessageType {
    Unknown,
    Notification,    /* NOTIFICATION i.e FB Msg */
    Weather,/* Weather packet */
    Date,   /* Date packet */
    Music,  /* Spotify controls */
}

// #[derive(Copy)]
pub struct Message {
    pub msg_type: MessageType,
    pub payload: [u8; 256],
}

pub struct MessageManager {
    pub msg_pool : [Message; 8],
    rb: &'static mut RingBuffer<u8, [u8; 128]>,
}

impl Message {
    pub fn new(rx_buffers: [u8; 256]) -> Self {
        Message {
            msg_type: MessageType::Unknown,
            payload: rx_buffers,
        }
    }
}

impl MessageManager 
{
    pub fn new(rx_buffers: [Message; 8], ring_t: &'static mut RingBuffer<u8, [u8; 128]>) -> Self {
        MessageManager {
            msg_pool: rx_buffers,
            rb: ring_t,
        }
    }
    // Change return sig to static reference?
    // Returns the next free buffer, should remove or flush Message to disk if mem is full
    //  pub fn get_free() -> Message {

    //  }
    // Returns the borrow msg reference and resets it ready for reuse
    // pub fn set_free(msg: Message){

    // }
    // pub fn get_msg(msg_index: usize) -> Message {

    // }

    
}