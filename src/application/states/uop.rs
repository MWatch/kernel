

use crate::application::states::prelude::*;

use embedded_graphics::Drawing;
use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;

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
               Image16BPP::new(include_bytes!("../../../data/uop.raw"), 48, 64)
                   .translate(Coord::new(32, 32))
                   .into_iter(),
         );
        None
    }

    fn input(&mut self, _system: &mut System, _display: &mut Ssd1351, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Left => Some(Signal::Previous),
            InputEvent::Right => Some(Signal::Next),
            _ => None
        }
    }
}

impl StaticState for UopState {}
