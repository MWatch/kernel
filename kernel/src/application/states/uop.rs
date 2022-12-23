//! Uop Logo state

use crate::{application::states::prelude::*, system::{System, input::InputEvent, Display}};

use embedded_graphics::{image::{Image, ImageRaw}, pixelcolor::{Rgb565, raw::LittleEndian}, prelude::{Point, OriginDimensions}, Drawable};

pub struct UopState {}

impl Default for UopState {
    fn default() -> Self {
        Self {
            
        }
    }
}

impl State for UopState {
    fn render(&mut self, _system: &mut impl System, display: &mut impl Display) -> Option<Signal> {
        let dsize = display.bounding_box().size;
        let image = ImageRaw::<Rgb565, LittleEndian>::new(include_bytes!("../../../data/uop.raw"), 48);
        let size = image.size();
        Image::new(
            &image,
            Point::new((dsize.width as i32 / 2) - size.width as i32 / 2, (dsize.height as i32 / 2) - size.height as i32 / 2),
        )
        .draw(display).ok();
        None
    }

    fn input(&mut self, _system: &mut impl System, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Left => Some(Signal::Previous),
            InputEvent::Right => Some(Signal::Next),
            _ => None
        }
    }
}

impl StaticState for UopState {}
