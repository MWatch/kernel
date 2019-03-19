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

pub struct NotificationState {
    buffer: String<U256>,
    is_running: bool,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            is_running: false,
        }
    }
}

impl State for NotificationState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        self.buffer.clear();
        write!(self.buffer, "Running [{}]", system.nm().idx()).unwrap();
        display.draw(centre(Font6x12::render_str(self.buffer.as_str()))
                .with_stroke(Some(0x02D4_u16.into()))
                .into_iter(),
        );
        None     
    }

    fn input(&mut self, system: &mut System, _display: &mut Ssd1351, input: InputEvent) -> Option<Signal> {
        match input {
            InputEvent::Multi => {
                self.stop(system);
                Some(Signal::Home) // signal to dm to go home
            }
            _ => {//TODO
                None
            }
        }
    }
}

impl ScopedState for NotificationState {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, _system: &mut System, display: &mut Ssd1351) -> Option<Signal> {
        self.buffer.clear();
        write!(self.buffer, "Notifications").unwrap(); 
        display.draw(horizontal_centre(Font6x12::render_str(self.buffer.as_str()), 24)
                .with_stroke(Some(0x02D4_u16.into()))
                .into_iter(),
        );
        None
    }

    fn is_running(&self, _system: &mut System) -> bool {
        self.is_running
    }

    /// Start 
    fn start(&mut self, _system: &mut System) {
        self.is_running = true;
    }

    /// Stop
    fn stop(&mut self, _system: &mut System) {
        self.is_running = false;
    }
}