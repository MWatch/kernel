
use embedded_graphics::{pixelcolor::{Rgb565, raw::RawU16}, primitives::Rectangle, prelude::{Size, Point, Dimensions, RawData}, Pixel};

use crate::system::input::InputEvent;

pub mod application_manager;
pub mod display_manager;
pub mod states;

/// The FFI function signature for initialising an application.
pub type SetupFn = unsafe extern "C" fn(*mut Table) -> i32;
/// The FFI function signature for servicing an application.
pub type ServiceFn = unsafe extern "C" fn(*mut Context) -> i32;
/// The FFI function signature for pushing an input event to an application.
pub type InputFn = unsafe extern "C" fn(*mut Context, InputEvent) -> i32;

#[repr(C)]
/// The context passed to the application interface.
pub struct Context {
    pub framebuffer: *mut FrameBuffer,
}

#[repr(C)]
#[derive(Debug)] 
/// FrameBuffer
/// 
/// A generic interface to write to a frame buffer.
pub struct FrameBuffer {
    ptr: *mut u8,
    len: usize,
    width: u8,
    height: u8,
    // TODO rotation of the FB?
}

impl FrameBuffer {
    /// Create a frame buffer from a pointer
    /// 
    /// # Safety
    /// 
    /// The pointer passed **must** but a unique pointer and not shared or aliased.
    pub unsafe fn new(ptr: *mut u8, len: usize, width: u8, height: u8) -> Self {
        Self {
            ptr,
            len,
            width,
            height,
        }
    }
}

impl embedded_graphics::draw_target::DrawTarget for FrameBuffer {
    // TODO put this behind a feature gate when we support more pixel formats
    // we can't use generics here because the FrameBuffer is passed over ffi.
    type Color = Rgb565;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| {
                let x = pos.x;
                let y = pos.y;
                let color: u16 = RawU16::from(color).into_inner();
                let slice = unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) };
                slice[((x + (y * self.width as i32)) as usize * 2)] = (color >> 8) as u8;
                slice[(((x + (y * self.width as i32)) as usize) * 2) + 1] = color as u8;
            });

        Ok(())
    }
}

impl embedded_graphics::geometry::Dimensions for FrameBuffer {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(self.width as u32, self.height as u32))
    }
}

/// WARNING only safe if we guarentee the safety ourselves, i.e context doesn't live longer than the &mut references that it contains
unsafe impl Send for Context {}

#[repr(C)]
/// The callbacks supplied by the OS.
pub struct Table {
    /// Draw a colour on the display - x, y, colour
    pub draw_pixel: unsafe extern "C" fn(*mut Context, u8, u8, u16) -> i32,
    /// Print a string using the info! macro
    pub print: unsafe extern "C" fn(*mut Context, ptr: *const u8, len: usize) -> i32,
}
