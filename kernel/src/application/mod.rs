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
static mut TABLE_POINTER: Option<&'static mut Table> = None;

pub struct Context {
    pub display: Option<*mut ()>,
}

/// WARNING only safe if we guarentee the safety ourselves, i.e context doesn't live longer than the &mut references that it contains
unsafe impl Send for Context {}

#[repr(C)]
/// The callbacks supplied by the OS.
pub struct Table {
    /// Draw a colour on the display - x, y, colour
    pub draw_pixel: unsafe extern "C" fn(*mut Context, u8, u8, u16) -> i32,
    /// Print a string using th info! macro
    pub print: unsafe extern "C" fn(*mut Context, ptr: *const u8, len: usize) -> i32,
}

impl Table {
    pub fn get() -> &'static mut Table {
        unsafe {
            if let Some(tbl) = &mut TABLE_POINTER {
                tbl
            } else {
                panic!("Callback table not initialized!");
            }
        }
    }

    pub unsafe fn install(t: &'static mut Self) {
        TABLE_POINTER = Some(t)
    }
}

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

