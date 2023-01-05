use crate::application::FrameBuffer;
use crate::application::states::prelude::*;
use crate::system::input::InputEvent;
use crate::system::{System, Host};

use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::raw::{LittleEndian, RawU16};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};

pub struct MWState {}

impl Default for MWState {
    fn default() -> Self {
        Self {}
    }
}

impl State for MWState {
    fn render(&mut self, _system: &mut System<impl Host>, display: &mut FrameBuffer) -> Option<Signal> {
        Image::new(
            &ImageRaw::<Rgb565, LittleEndian>::new(include_bytes!("../../../data/mwatch.raw"), 64),
            Point::new(32, 10),
        )
        .draw(display).ok();

        let size = display.bounding_box().size;
        let style = MonoTextStyle::new(&FONT_6X12, RawU16::from(0x02D4).into());

        Text::with_alignment(
            "Project by",
            Point::new(size.width as i32 / 2, 85),
            style,
            Alignment::Center,
        )
        .draw(display).ok();

        Text::with_alignment(
            "Scott Mabin 2019",
            Point::new(size.width as i32 / 2, 97),
            style,
            Alignment::Center,
        )
        .draw(display).ok();

        Text::with_alignment(
            "@MabezDev on Github",
            Point::new(size.width as i32 / 2, 116),
            style,
            Alignment::Center,
        )
        .draw(display).ok();

        None
    }

    fn input(&mut self, _system: &mut System<impl Host>, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Left => Some(Signal::Previous),
            InputEvent::Right => Some(Signal::Next),
            _ => None,
        }
    }
}

impl StaticState for MWState {}
