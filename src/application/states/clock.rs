

use crate::application::wm::{State, StaticState, Signal};
use crate::Ssd1351;
use crate::system::system::System;

use heapless::String;
use heapless::consts::*;
use crate::system::bms::State as BmsState;
use core::fmt::Write;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font6x12;
// use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;

use mwatch_kernel_api::InputEvent;

use crate::application::seven_segment::SevenSegments;

pub struct ClockState {
    buffer: String<U256>,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl State for ClockState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        let time = system.rtc().get_time();
        let date = system.rtc().get_date();
        let soc = system.bms().soc();
        let bms_state = system.bms().state();
        {
            let mut clock_digits = SevenSegments::new(display, 6, 40, 0x2C78);
            write!(
                self.buffer,
                "{:02}{:02}",
                time.hours, time.minutes
            ).unwrap();
            for (idx, digit) in self.buffer.as_bytes().iter().enumerate() {
                clock_digits.digit(digit - b'0');
                if idx == (self.buffer.len() / 2) - 1 { // put a colon between hours and mins
                    clock_digits.colon();
                }
            }

            self.buffer.clear(); // reset the buffer
        }
        write!(self.buffer, "{:02}/{:02}/{:04}", date.date, date.month, date.year).unwrap();
        display.draw(
            Font6x12::render_str(self.buffer.as_str())
                .translate(Coord::new(30, 128 - 12))
                .with_stroke(Some(0x2679_u16.into()))
                .into_iter(),
        );
        self.buffer.clear();
        write!(self.buffer, "{:02}%", soc).unwrap();
        display.draw(
            Font6x12::render_str(self.buffer.as_str())
                .translate(Coord::new(110, 12))
                .with_stroke(Some(0x2679_u16.into()))
                .into_iter(),
        );
        self.buffer.clear(); // reset the buffer
        match bms_state {
            BmsState::Charging => {
                write!(self.buffer, "CHARGING").unwrap();
            },
            BmsState::Draining => {
                write!(self.buffer, "DRAINING").unwrap();
            },
            BmsState::Charged => {
                write!(self.buffer, "DONE").unwrap();
            },
        }
        display.draw(
            Font6x12::render_str(self.buffer.as_str())
                .translate(Coord::new(0, 12))
                .with_stroke(Some(0x2679_u16.into()))
                .into_iter(),
        );
        self.buffer.clear(); // reset the buffer
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

impl StaticState for ClockState {}
