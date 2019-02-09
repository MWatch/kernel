//! System info state - debugging

use crate::application::wm::{State, InputState};
use crate::Ssd1351;
use crate::system::system::System;

use mwatch_kernel_api::InputEvent;

use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::Drawing;
// use embedded_graphics::fonts::Font12x16;
use embedded_graphics::fonts::Font6x12;
// use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;

pub struct AppState {
    buffer: String<U256>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl State for AppState {
    fn render(&mut self, system: &mut System, display: &mut Ssd1351){
        if system.am().status().is_running {//TODO remove this when the WM has comeplete control over when apps start
            system.am().service(display).unwrap();
        } else {
            self.buffer.clear();
            write!(self.buffer, "No application Loaded!").unwrap();
            display.draw(
                Font6x12::render_str(self.buffer.as_str())
                    .translate(Coord::new(24, 24))
                    .with_stroke(Some(0xF818_u16.into()))
                    .into_iter(),
            );
        }
    }

    fn service(&mut self, _system: &mut System){
        
    }
}

impl InputState for AppState {
    fn service_input(&mut self, system: &mut System, display: &mut Ssd1351, input: InputEvent){
        if system.am().status().is_running {//TODO remove this when the WM has comeplete control over when apps start
            system.am().service_input(display, input).unwrap();
        }
    }
}