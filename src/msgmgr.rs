
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
    pub msg_pool : [Message; 8]
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
     pub fn new(rx_buffers: [Message; 8]) -> Self {
        MessageManager {
            msg_pool: rx_buffers,
        }
     }
}