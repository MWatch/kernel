

use crate::application::wm::{State, ExitCode};
use crate::Ssd1351;
use crate::system::system::System;
use stm32l4xx_hal::datetime::Time;

use heapless::String;
use heapless::consts::*;
use crate::system::bms::State as BmsState;
use core::fmt::Write;
use stm32l4xx_hal::prelude::*;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font12x16;
use embedded_graphics::fonts::Font6x12;
// use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;

use mwatch_kernel_api::InputEvent;

pub struct ClockState {
    buffer: String<U256>,
    time: Time,
    soc: u16,
    bms_state: BmsState,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            time: Time::new(0u32.hours(), 0u32.minutes(), 0u32.seconds(), false),
            soc: 0,
            bms_state: BmsState::Draining,
        }
    }
}

impl State for ClockState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Result<(), ExitCode> {
        self.time = system.rtc().get_time();
        self.soc = system.bms().soc();
        self.bms_state = system.bms().state();
        write!(
            self.buffer,
            "{:02}:{:02}:{:02}",
            self.time.hours, self.time.minutes, self.time.seconds
        )
        .unwrap();
        display.draw(
            Font12x16::render_str(self.buffer.as_str())
                .translate(Coord::new(10, 40))
                .with_stroke(Some(0x2679_u16.into()))
                .into_iter(),
        );
        self.buffer.clear(); // reset the buffer
                        // write!(buffer, "{:02}:{:02}:{:04}", date.date, date.month, date.year).unwrap();
        write!(self.buffer, "{:02}%", self.soc).unwrap();
        display.draw(
            Font6x12::render_str(self.buffer.as_str())
                .translate(Coord::new(110, 12))
                .with_stroke(Some(0x2679_u16.into()))
                .into_iter(),
        );
        self.buffer.clear(); // reset the buffer
        match self.bms_state {
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
        Ok(())
    }

    fn input(&mut self, _system: &mut System, _display: &mut Ssd1351, input: InputEvent) -> Result<(), ExitCode> {
        match input {
            InputEvent::Left => Err(ExitCode::Previous),
            InputEvent::Right => Err(ExitCode::Next),
            _ => Ok(())
        }
    }
}