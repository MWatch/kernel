//! Push notification parsing
//! 

use buffer_manager::Buffer;

pub struct Notification<'a>{
    app_name_idx: usize,
    titleidx: usize,
    text_idx: usize,
    inner: &'a Buffer,
}

impl<'a> From<&'a Buffer> for Notification<'a> {

    fn from(buffer: &'a Buffer) -> Notification<'a> {
        Notification { /* TODO parsing */
            app_name_idx: 0, 
            titleidx: 0,
            text_idx: 0,
            inner: buffer,
        }
    }
}