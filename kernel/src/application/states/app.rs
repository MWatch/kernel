//! Application state
//!
//! Wraps the application manager in a display manager state
//!  

use crate::application::states::prelude::*;
use crate::system::input::InputEvent;
use crate::system::Display;
use crate::system::System;
use core::fmt::Write;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::{mono_font::{MonoTextStyle, ascii::FONT_6X10}, pixelcolor::Rgb565, text::{Alignment, Text}};
use heapless::String;

use embedded_graphics::prelude::*;

pub struct AppState {
    buffer: String<256>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl State for AppState {
    fn render(&mut self, system: &mut impl System, display: &mut impl Display) -> Option<Signal> {
        system.am().service(display).unwrap_or_else(|err| {
            error!("Failed to render app {:?}", err);
        });
        None
    }

    fn input(&mut self, system: &mut impl System, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Multi => {
                system.am().pause();
                Some(Signal::Home) // signal to dm to go home
            }
            _ => {
                system.am().service_input(input).unwrap_or_else(|err| {
                    error!("Failed to service input for app {:?}", err);
                });
                None
            }
        }
    }
}

impl ScopedState for AppState {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, system: &mut impl System, display: &mut impl Display) -> Option<Signal> {
        self.buffer.clear();
        let status = system.am().status();
        if status.is_loaded {
            write!(self.buffer, "Open loaded App").unwrap();
        } else {
            write!(self.buffer, "No App loaded!").unwrap();
        }

        let size = display.bounding_box().size;
        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::from(RawU16::from(0x02D4)));
        Text::with_alignment(self.buffer.as_str(), Point::new(size.width as i32 / 2, size.height as i32 / 2), style, Alignment::Center).draw(display).ok();

        None
    }

    fn is_running(&self, system: &mut impl System) -> bool {
        system.am().status().is_running
    }

    /// Start
    fn start(&mut self, system: &mut impl System) {
        match system.am().execute() {
            Ok(_) => {}
            Err(err) => error!("Failed to launch application {:?}", err),
        }
    }

    /// Stop
    fn stop(&mut self, system: &mut impl System) {
        system.am().kill().unwrap_or_else(|err| {
            error!("Failed to kill app {:?}", err);
        });
    }
}
