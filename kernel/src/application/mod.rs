
use embedded_graphics::{pixelcolor::{Rgb565, raw::RawU16}, primitives::Rectangle, prelude::{Size, Point, Dimensions, RawData}, Pixel};

use crate::system::input::InputEvent;

pub mod application_manager;
pub mod display_manager;
pub mod states;

pub type InputHandlerFn = extern "C" fn(*mut Context, bool) -> i32;

pub type SetupFn = extern "C" fn() -> i32;
pub type ServiceFn = extern "C" fn(*mut Context) -> i32;
pub type InputFn = extern "C" fn(*mut Context, InputEvent) -> i32;

pub static mut CONTEXT_POINTER: Option<&'static mut Context> = None;
static mut TABLE_POINTER: Option<&'static mut Table> = None;

#[repr(C)]
pub struct Context {
    pub framebuffer: Option<FrameBuffer>,
}

#[repr(C)]
#[derive(Debug)] // TODO rotation of the FB?
pub struct FrameBuffer {
    ptr: *mut u8,
    len: usize,
    width: u8,
    height: u8,
}

impl FrameBuffer {
    pub fn new(ptr: *mut u8, len: usize, width: u8, height: u8) -> Self {
        Self {
            ptr,
            len,
            width,
            height,
        }
    }
}

// impl embedded_graphics::Drawing<PixelColorU16> for FrameBuffer {
//     fn draw<T>(&mut self, item_pixels: T)
//     where
//         T: Iterator<Item = embedded_graphics::drawable::Pixel<PixelColorU16>>,
//     {
//         for embedded_graphics::drawable::Pixel(
//             embedded_graphics::unsignedcoord::UnsignedCoord(x, y),
//             color,
//         ) in item_pixels
//         {
//             if x <= self.width.into() && y <= self.height.into() {
//                 let color = color.into_inner();
//                 let slice = unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) };
//                 slice[((x + (y * self.width as u32)) as usize * 2)] = (color >> 8) as u8;
//                 slice[(((x + (y * self.width as u32)) as usize) * 2) + 1] = color as u8;
//             }
//         }
//     }
// }

impl embedded_graphics::draw_target::DrawTarget for FrameBuffer {
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
