//! System info state - debugging

use crate::application::wm::State;
use crate::Ssd1351;
use crate::system::system::System;

use heapless::String;
use heapless::consts::*;
use core::fmt::Write;

use embedded_graphics::Drawing;
// use embedded_graphics::fonts::Font12x16;
use embedded_graphics::fonts::Font6x12;
// use embedded_graphics::image::Image16BPP;
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
    fn render(&mut self, _system: &mut System, display: &mut Ssd1351){
        // write!(buffer, "CPU_USAGE: {:.02}%", *resources.CPU_USAGE).unwrap();
        // display.draw(
        //     Font6x12::render_str(buffer.as_str())
        //         .translate(Coord::new(0, 12))
        //         .with_stroke(Some(0xF818_u16.into()))
        //         .into_iter(),
        // );
        self.buffer.clear();
        write!(self.buffer, "RAM: {} bytes", System::get_free_stack()).unwrap();
        display.draw(
            Font6x12::render_str(self.buffer.as_str())
                .translate(Coord::new(0, 24))
                .with_stroke(Some(0xF818_u16.into()))
                .into_iter(),
        );
    }

    fn service(&mut self, _system: &mut System){
        
    }
}