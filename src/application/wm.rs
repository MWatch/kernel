//! Window manager
//!
//! Handles app switching, between built in apps and custom apps

use crate::Ssd1351;
use crate::system::system::System;
use crate::application::states::clock::ClockState;

/// All built in states must implement this trait to be renderable by the WM
pub trait State: Default {
    fn render(&mut self, display: &mut Ssd1351);
    fn service(&mut self, system: &mut System);
}


pub struct WindowManager 
{
    state_idx: u8,
    clock_state: ClockState
}

impl WindowManager
{
    pub fn new() -> Self {
        Self {
            state_idx: 0,
            clock_state: ClockState::default()
        }
    }

    /// Move to the next state, wrapping if necessary
    pub fn next(&mut self) {
        
    }

    /// Move to the previous state, wrapping if necessary
    pub fn prev(&mut self) {

    }

    pub fn process(&mut self, display: &mut Ssd1351, system: &mut System) {
        match self.state_idx {
            0 => {
                self.clock_state.service(system);
                self.clock_state.render(display)
            }
            _ => panic!("Unhandled state")
        }
    }
}