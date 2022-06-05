use crate::system::input::InputEvent;

pub mod application_manager;
pub mod display_manager;
pub mod states;
pub mod render_util;


pub type InputHandlerFn = extern "C" fn(*mut Context, bool) -> i32;

pub type SetupFn = extern "C" fn() -> i32;
pub type ServiceFn = extern "C" fn(*mut Context) -> i32;
pub type InputFn = extern "C" fn(*mut Context, InputEvent) -> i32;

pub static mut CONTEXT_POINTER: Option<&'static mut Context> = None;

pub struct Context {
    pub display: Option<*mut ()>, // *mut () acts as a void pointer
    pub log: extern "C" fn(&str) -> i32,
}

/// WARNING only safe if we guarentee the safety ourselves, i.e context doesn't live longer than the &mut references that it contains
unsafe impl Send for Context {}

#[repr(C)]
/// The callbacks supplied by the OS.
pub struct Table {
    /// Draw a colour on the display - x, y, colour
    pub draw_pixel: unsafe extern "C" fn(*mut Context, u8, u8, u16) -> i32,
    /// Print a string using th info! macro
    pub print: unsafe extern "C" fn(*mut Context, &str) -> i32,
}

pub static CALLBACK_TABLE: Table = Table {
    draw_pixel,
    print
};

impl Context {
    pub fn get() -> &'static mut Context {
        unsafe {
            if let Some(tbl) = &mut CONTEXT_POINTER {
                tbl
            } else {
                panic!("Bad context, context is only valid within update()!");
            }
        }
    }
}


/// Assumes control over the display, it is up to use to make sure the display is not borrowed by anything else
pub unsafe extern "C" fn draw_pixel(context: *mut Context, x: u8, y: u8, colour: u16) -> i32 {
    let ctx = &mut *context;
    if let Some(display) = &mut ctx.display {
        // let display = display as &mut 
        // display.set_pixel(u32::from(x), u32::from(y), colour);
        todo!("how to invoke display methods?") // TODO known interface?
    } else {
        panic!("Display invoked in an invalid state. Applications can only use the display within update.")
    }
    0
}

pub unsafe extern "C" fn print(context: *mut Context, string: &str) -> i32 {
    let ctx = &mut *context;
    (ctx.log)(string);
    0
}

