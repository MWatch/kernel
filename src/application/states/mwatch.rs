

use crate::application::states::prelude::*;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;

pub struct MWState {}

impl Default for MWState {
    fn default() -> Self {
        Self {
            
        }
    }
}

impl State for MWState {
    fn render(&mut self, _system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        display.draw(
                Image16BPP::new(include_bytes!("../../../data/mwatch.raw"), 64, 64)
                    .translate(Coord::new(32, 10))
                    .into_iter(),
                );
        let text: Font6x12<_> = Font6x12::render_str("Project by");
        display.draw(horizontal_centre(text, 80)
                     .with_stroke(Some(0x02D4_u16.into()))
                     .into_iter());

        let text: Font6x12<_> = Font6x12::render_str("Scott Mabin 2019");
        display.draw(horizontal_centre(text, 92)
                     .with_stroke(Some(0x02D4_u16.into()))
                     .into_iter());

        let text: Font6x12<_> = Font6x12::render_str("@MabezDev on Github");
        display.draw(horizontal_centre(text, 118)
                     .with_stroke(Some(0x02D4_u16.into()))
                     .into_iter());
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

impl StaticState for MWState {}
