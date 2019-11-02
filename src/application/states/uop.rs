//! Uop Logo state

use crate::application::states::prelude::*;

use embedded_graphics::Drawing;
use embedded_graphics::image::Image16BPP;

pub struct UopState {}

impl Default for UopState {
    fn default() -> Self {
        Self {
            
        }
    }
}

impl State for UopState {
    fn render(&mut self, _system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        display.draw(
               centre(Image16BPP::new(include_bytes!("../../../data/uop.raw"), 48, 64))
                   .into_iter(),
         );
        None
    }

    fn input(&mut self, _system: &mut System, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Left => Some(Signal::Previous),
            InputEvent::Right => Some(Signal::Next),
            _ => None
        }
    }
}

impl StaticState for UopState {}
