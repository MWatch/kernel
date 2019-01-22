//! Push notification parsing
//! 

use buffer::Buffer;

pub struct Notification<'a>{
    app_name_idx: usize,
    title_idx: usize,
    text_idx: usize,
    inner: &'a Buffer,
}

impl<'a> From<&'a Buffer> for Notification<'a> {

    fn from(buffer: &'a Buffer) -> Notification<'a> {
        Notification { /* TODO parsing */
            app_name_idx: 0, 
            title_idx: 0,
            text_idx: 0,
            inner: buffer,
        }
    }
}