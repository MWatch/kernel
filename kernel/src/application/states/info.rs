//! Debug info state

use crate::application::states::prelude::*;
use crate::system::Display;
use crate::system::System;
use crate::system::input::InputEvent;
use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::fonts::Font6x12;
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
        display.draw(
            Font6x12::render_str(self.buffer.as_str())
                .translate(Coord::new(0, 12))
                .with_stroke(Some(0xF818_u16.into()))
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

impl StaticState for InfoState {}