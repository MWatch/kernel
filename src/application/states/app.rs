//! Application state
//!
//! Wraps the application manager in a display manager state
//!  

use crate::application::states::prelude::*;

use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::prelude::*;

pub struct AppState {
    buffer: String<U256>
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl State for AppState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        system.am().service(display).unwrap_or_else(|err| {
            error!("Failed to render app {:?}", err);
        });
        None     
    }

    fn input(&mut self, system: &mut System, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Multi => {
                system.am().pause();
                Some(Signal::Home) // signal to dm to go home
            }
            _ => {
                system.am().service_input(input).unwrap_or_else(|err|{
                    error!("Failed to service input for app {:?}", err);
                });
                None
            }
        }
    }
}

impl ScopedState for AppState {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        self.buffer.clear();
        let status = system.am().status();
        if status.is_loaded {
            write!(self.buffer, "Open loaded App").unwrap();
        } else {
            write!(self.buffer, "No App loaded!").unwrap();
        }

        let text = Font6x12::render_str(self.buffer.as_str());
        display.draw(text
                .translate(Coord::new(64 - text.size().0 as i32 / 2, 24))
                .with_stroke(Some(0x02D4_u16.into()))
                .into_iter(),
        );
        None
    }

    fn is_running(&self, system: &mut System) -> bool {
        system.am().status().is_running
    }

    /// Start 
    fn start(&mut self, system: &mut System) {
        match system.am().execute() {
            Ok(_) => {},
            Err(err) => error!("Failed to launch application {:?}", err)
        }
    }

    /// Stop
    fn stop(&mut self, system: &mut System) {
        system.am().kill().unwrap_or_else(|err|{
            error!("Failed to kill app {:?}", err);
        });
    }
}