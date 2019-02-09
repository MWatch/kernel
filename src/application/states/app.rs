//! System info state - debugging

use crate::application::wm::{State, ScopedState, ExitCode};
use crate::Ssd1351;
use crate::system::system::System;

use mwatch_kernel_api::InputEvent;

use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::Drawing;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::prelude::*;

pub struct AppState {
    buffer: String<U256>,
    running: bool
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            running: false,
        }
    }
}

impl State for AppState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Result<(), ExitCode> {
        system.am().service(display).unwrap();
        Ok(())     
    }

    fn input(&mut self, system: &mut System, display: &mut Ssd1351, input: InputEvent) -> Result<(), ExitCode> {
        if self.is_running() {
            system.am().service_input(display, input).unwrap();
            Ok(())
        } else {
            match input {
                InputEvent::Left => Err(ExitCode::Previous),
                InputEvent::Right => Err(ExitCode::Next),
                InputEvent::Middle => {
                    self.start(system);
                    Ok(())
                }
                _ => Ok(())
            }
            
        }
    }
}

impl ScopedState for AppState {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, _system: &mut System, display: &mut Ssd1351) -> Result<(), ExitCode> {
        self.buffer.clear();
        write!(self.buffer, "Open App").unwrap();
        display.draw(
            Font6x12::render_str(self.buffer.as_str())
                .translate(Coord::new(24, 24))
                .with_stroke(Some(0xF818_u16.into()))
                .into_iter(),
        );
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    /// Start 
    fn start(&mut self, system: &mut System) {
        match system.am().execute() {
            Ok(_) => self.running = true,
            Err(err) => error!("Failed to launch application {:?}", err)
        }
        
    }

    /// Stop
    fn stop(&mut self, system: &mut System) {
        system.am().stop().unwrap();
        self.running = false;
    }
}