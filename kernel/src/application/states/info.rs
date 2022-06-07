//! Debug info state

use crate::application::states::prelude::*;
use crate::system::Display;
use crate::system::System;
use crate::system::input::InputEvent;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X12;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::text::Text;
use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::prelude::*;

pub struct InfoState {
    buffer: String<U256>,
}

impl Default for InfoState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl State for InfoState {
    fn render(&mut self, _system: &mut impl System, display: &mut impl Display) -> Option<Signal> {
        let style = MonoTextStyle::new(&FONT_6X12, RawU16::new(0x02D4).into());

        // write!(self.buffer, "CPU_USAGE: {:.02}%", system.ss().cpu_usage).unwrap();
        // display.draw(
        //     Font6x12::render_str(self.buffer.as_str())
        //         .translate(Coord::new(0, 12))
        //         .with_stroke(Some(0xF818_u16.into()))
        //         .into_iter(),
        // );
        // self.buffer.clear();
        // write!(self.buffer, "TSC EVENTS: {}/s", system.ss().tsc_events).unwrap();
        // display.draw(
        //     Font6x12::render_str(self.buffer.as_str())
        //         .translate(Coord::new(0, 36))
        //         .with_stroke(Some(0xF818_u16.into()))
        //         .into_iter(),
        // );
        // self.buffer.clear();
        // write!(self.buffer, "TSC THRES: {}", system.ss().tsc_threshold).unwrap();
        // display.draw(
        //     Font6x12::render_str(self.buffer.as_str())
        //         .translate(Coord::new(0, 48))
        //         .with_stroke(Some(0xF818_u16.into()))
        //         .into_iter(),
        // );
        // self.buffer.clear();
        write!(self.buffer, "Stats go here").unwrap();
        Text::new(self.buffer.as_str(), Point::new(0, 12), style).draw(display).ok();
        self.buffer.clear();

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

impl StaticState for InfoState {}