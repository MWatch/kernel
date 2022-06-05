//! Uop Logo state

use crate::{application::states::prelude::*, system::{System, input::InputEvent}};

use embedded_graphics::{image::Image16BPP, Drawing, pixelcolor::PixelColorU16};

pub struct UopState {}

impl Default for UopState {
    fn default() -> Self {
        Self {
            
        }
    }
}

impl State for UopState {
    fn render(&mut self, _system: &mut impl System, display: &mut impl Drawing<PixelColorU16>) -> Option<Signal> {
        display.draw(
               centre(Image16BPP::new(include_bytes!("../../../data/uop.raw"), 48, 64))
                   .into_iter(),
         );
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
