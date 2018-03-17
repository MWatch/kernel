

extern crate heapless;
extern crate cortex_m;

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
    pub msg_pool : [Message; 8],
    rb: &'static mut RingBuffer<u8, [u8; 128]>,
    current_msg_idx: usize,
}

impl MessageManager 
{
    pub fn new(msgs: [Message; 8], ring_t: &'static mut RingBuffer<u8, [u8; 128]>) -> Self {
        MessageManager {
            msg_pool: msgs,
            rb: ring_t,
            current_msg_idx: 0,
        }
    }

    pub fn write(&mut self, data: &[u8]){
        for byte in data {
            // if we overrun it just means the data in the buffer is not usefull to us
            // or the consumer (MsgMngr in systick) is not keeping up!
            match self.rb.enqueue(*byte){
                Ok(_) => {}
                Err(_) => {
                    //TODO this needs some looking at
                    // wrap back around
                    for _ in 0..self.rb.capacity() {
                        self.rb.dequeue();
                    }
                    // requeue the byte that failed
                    self.rb.enqueue(*byte);
                }
            }
        }
    }

    pub fn process(&self){
        // for byte in self.rb {
        //     /* Run through state machine per byte */
        //     /* Start Bits */
        //     /* Stops bits */         
        // }
    }

    pub fn print_rb(&self, itm: &mut cortex_m::peripheral::itm::Stim){
        if self.rb.is_empty() {
            iprintln!(itm, "RB is Empty!");
        } else {
            iprintln!(itm, "RB Contents: ");
            for x in self.rb.iter() {
                iprint!(itm, "{}", *x as char);
            }
            iprintln!(itm, "");
        }
    }

    // Returns the index of the next free buffer,
    // Returns None if no free buffers are available
     fn get_next_free(self) -> Option<usize> {
        // Some(0)
        unimplemented!();
     }
    // pub fn get_msg(msg_index: usize) -> Message {

    // }

    
}