//! Clock state
//!
//! The main home page

use crate::application::FrameBuffer;
use crate::application::states::prelude::*;
use crate::system::Clock;
use crate::system::Host;
use crate::system::Statistics;
use crate::system::System;

use crate::system::bms::BatteryManagement;
use crate::system::bms::State as BmsState;
use crate::system::input::InputEvent;
use core::fmt::Write;
use embedded_graphics::pixelcolor::raw::RawU16;
use heapless::String;

use embedded_graphics::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X12, MonoTextStyle},
    text::Text,
};

use seven_segment::SevenSegments;

pub struct ClockState {
    buffer: String<256>,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl State for ClockState {
    fn render(&mut self, system: &mut System<impl Host>, display: &mut FrameBuffer) -> Option<Signal> {
        let time = system.clock.get_time();
        let date = system.clock.get_date();
        let soc = system.bms.soc();
        let bms_state = system.bms.state();
        let mut clock_digits = SevenSegments::new(display, 18, 48, 0x2C78);
        write!(self.buffer, "{:02}{:02}", time.hour(), time.minute()).unwrap();
        for (idx, digit) in self.buffer.as_bytes().iter().enumerate() {
            clock_digits.digit(digit - b'0');
            if idx == (self.buffer.len() / 2) - 1 {
                // put a colon between hours and mins
                clock_digits.colon();
            }
        }

        self.buffer.clear(); // reset the buffer
        if !system.stats.is_idle() {
            let size = display.bounding_box().size;
            let style = MonoTextStyle::new(&FONT_6X12, RawU16::new(0x2C78).into());

            write!(
                self.buffer,
                "{:02}/{:02}/{:04}",
                date.day(), date.month(), date.year()
            )
            .unwrap();
            Text::new(
                self.buffer.as_str(),
                Point::new(30, size.height as i32 - 12),
                style,
            )
            .draw(display)
            .ok();
            self.buffer.clear();

            write!(self.buffer, "{soc:02}%").unwrap();
            Text::new(self.buffer.as_str(), Point::new(110, 12), style)
                .draw(display)
                .ok();
            self.buffer.clear(); // reset the buffer

            match bms_state {
                BmsState::Charging => {
                    write!(self.buffer, "CHARGING").unwrap();
                }
                BmsState::Draining => {
                    write!(self.buffer, "DRAINING").unwrap();
                }
                BmsState::Charged => {
                    write!(self.buffer, "DONE").unwrap();
                }
            }
            Text::new(self.buffer.as_str(), Point::new(0, 12), style)
                .draw(display)
                .ok();
            self.buffer.clear(); // reset the buffer
        }

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

impl StaticState for ClockState {}

mod seven_segment {
    use embedded_graphics::{
        pixelcolor::{raw::RawU16, Rgb565},
        prelude::*,
        primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    };

    pub struct SevenSegments<'a, D> {
        display: &'a mut D,
        width: i32,
        height: i32,
        thickness: i32,
        space: i32,
        x: i32,
        y: i32,
        colour: u16,
    }

    impl<'a, D> SevenSegments<'a, D>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        pub fn new(display: &'a mut D, x: i32, y: i32, colour: u16) -> Self {
            Self {
                display,
                width: 16,
                height: 35,
                thickness: 4,
                space: 5,
                x,
                y,
                colour,
            }
        }

        pub fn colon_space(&mut self) {
            self.x += self.thickness + self.space;
        }

        pub fn colon(&mut self) {
            let t = self.thickness;
            let intern = (self.height - 3 * t) / 2;
            let h1 = t + intern / 2 - t / 2;
            let h2 = self.height - t - intern / 2 - t / 2;
            self.draw_rect(0, h1, t - 1, h1 + t - 1);
            self.draw_rect(0, h2, t - 1, h2 + t - 1);

            self.colon_space();
        }

        pub fn digit_space(&mut self) {
            self.x += self.width + self.space;
        }

        pub fn digit(&mut self, c: u8) {
            fn s(s: u8) -> u8 {
                1 << s
            }
            let segments = match c {
                0 => s(0) | s(1) | s(2) | s(4) | s(5) | s(6),
                1 => s(2) | s(5),
                2 => s(0) | s(2) | s(3) | s(4) | s(6),
                3 => s(0) | s(2) | s(3) | s(5) | s(6),
                4 => s(1) | s(2) | s(3) | s(5),
                5 => s(0) | s(1) | s(3) | s(5) | s(6),
                6 => s(0) | s(1) | s(3) | s(4) | s(5) | s(6),
                7 => s(0) | s(2) | s(5),
                8 => s(0) | s(1) | s(2) | s(3) | s(4) | s(5) | s(6),
                9 => s(0) | s(1) | s(2) | s(3) | s(5) | s(6),
                _ => 0,
            };

            let (h, w, t) = (self.height, self.width, self.thickness);
            let h2 = (h - 3 * t) / 2 + t;
            if segments & 1 != 0 {
                self.draw_rect(0, 0, w - 1, t - 1);
            }
            if segments & (1 << 1) != 0 {
                self.draw_rect(0, 0, t - 1, h2 + t - 1);
            }
            if segments & (1 << 2) != 0 {
                self.draw_rect(w - t, 0, w - 1, h2 + t - 1);
            }
            if segments & (1 << 3) != 0 {
                self.draw_rect(t, h2, w - t - 1, h2 + t - 1);
            }
            if segments & (1 << 4) != 0 {
                self.draw_rect(0, h2, t - 1, h - 1);
            }
            if segments & (1 << 5) != 0 {
                self.draw_rect(w - t, h2, w - 1, h - 1);
            }
            if segments & (1 << 6) != 0 {
                self.draw_rect(0, h - t, w - 1, h - 1);
            }

            self.digit_space();
        }

        fn draw_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(RawU16::new(self.colour).into())
                .build();
            Rectangle::with_corners(Point::new(x1, y1), Point::new(x2, y2))
                .translate(Point::new(self.x, self.y))
                .draw_styled(&style, self.display)
                .ok();
        }
    }
}
