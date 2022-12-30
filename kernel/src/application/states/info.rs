//! Debug info state

use crate::application::states::prelude::*;
use crate::system::Display;
use crate::system::Host;
use crate::system::Statistics;
use crate::system::System;
use crate::system::input::InputEvent;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::text::Baseline;
use embedded_graphics::text::Text;
use embedded_graphics::prelude::*;

pub struct InfoState;

impl Default for InfoState {
    fn default() -> Self {
        Self
    }
}

impl State for InfoState {
    fn render(&mut self, system: &mut System<impl Host>, display: &mut impl Display) -> Option<Signal> {
        let style = MonoTextStyle::new(&FONT_6X12, RawU16::new(0x02D4).into());

        for (i, buffer) in system.stats.stats().enumerate() {
            Text::with_baseline(
                &buffer,
                Point::new(0, (i as i32 * 12) + 2),
                style,
                Baseline::Top
            )
            .draw(display).ok();
        }

        None
    }

    fn input(&mut self, _system: &mut System<impl Host>, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Left => Some(Signal::Previous),
            InputEvent::Right => Some(Signal::Next),
            _ => None
        }
    }
}

impl StaticState for InfoState {}