//! Window manager
//!
//! Handles app switching, between built in apps and custom apps

use crate::Ssd1351;
use crate::system::system::System;
use crate::application::states::{
                                    clock::ClockState,
                                    info::InfoState,
                                };

/// All built in states must implement this trait to be renderable by the WM
pub trait State: Default {
    /// To draw the state to the display
    fn render(&mut self, system: &mut System, display: &mut Ssd1351);
    /// Allows the WM to process logic only, usefull if the operations are expensive and you want to cache the results
    fn service(&mut self, system: &mut System);
}

const MAX_STATES: i8 = 2;

pub struct WindowManager 
{
    state_idx: i8,
    clock_state: ClockState,
    info_state: InfoState,
}

impl WindowManager
{
    pub fn new() -> Self {
        Self {
            state_idx: 0,
            clock_state: ClockState::default(),
            info_state: InfoState::default(),
        }
    }

    /// Move to the next state, wrapping if necessary
    pub fn next(&mut self) {
        self.state_idx += 1;
        if self.state_idx > MAX_STATES - 1 {
            self.state_idx = 0;
        }
    }

    /// Move to the previous state, wrapping if necessary
    pub fn prev(&mut self) {
        self.state_idx -= 1;
        if self.state_idx < 0 {
            self.state_idx = MAX_STATES - 1;
        }
    }

    pub fn process(&mut self, display: &mut Ssd1351, system: &mut System) {
        match self.state_idx {
            0 => {
                self.clock_state.service(system);
                self.clock_state.render(system, display)
            },
            1 => {
                self.info_state.service(system);
                self.info_state.render(system, display)
            }
            _ => panic!("Unhandled state")
        }
    }
}