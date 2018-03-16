
/* 
    Message is a type
 */

#[derive(Debug)]
struct Message {
    // type: 
}

pub struct MessageManager {
    buffers: &'static mut [[u8; 256]; 8],
}


impl MessageManager 
{
     pub fn new(rx_buffers: &'static mut [[u8; 256]; 8]) -> Self {
        MessageManager {
            buffers: rx_buffers,
        }
     }
}