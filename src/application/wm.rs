//! Window manager
//!
//! Handles app switching, between built in apps and custom apps

use crate::Ssd1351;
use crate::system::system::System;
use crate::application::states::{
                                    clock::ClockState,
                                    info::InfoState,
                                    app::AppState,
                                };

use mwatch_kernel_api::InputEvent;

/// All built in states must implement this trait to be renderable by the WM
pub trait State: Default {
    /// To draw the state to the display
    fn render(&mut self, system: &mut System, display: &mut Ssd1351);
    /// Allows the WM to process logic only, usefull if the operations are expensive and you want to cache the results
    fn service(&mut self, system: &mut System);
}

pub trait InputState: State {
    /// Allows the state to take control of inputs from the kernel
    fn service_input(&mut self, system: &mut System, display: &mut Ssd1351, input: InputEvent); //TODO can we remove the need for the display?
}

const MAX_STATES: i8 = 3;

pub struct WindowManager 
{
    state_idx: i8,
    input_control: bool,
    clock_state: ClockState,
    info_state: InfoState,
    app_state: AppState,
}

impl WindowManager
{
    pub fn new() -> Self {
        Self {
            state_idx: 0,
            input_control: false,
            clock_state: ClockState::default(),
            info_state: InfoState::default(),
            app_state: AppState::default(),
        }
    }

    pub fn service_input(&mut self, display: &mut Ssd1351, system: &mut System, input: InputEvent) {
        if !self.input_control { // Not in a state, just traversing through states
            match input {
                InputEvent::Left => {
                    self.state_idx -= 1;
                    if self.state_idx < 0 {
                        self.state_idx = MAX_STATES - 1;
                    }
                }
                InputEvent::Middle => {
                    system.am().execute().unwrap(); //TODO make this generic
                    self.input_control = true;
                }
                InputEvent::Right => {
                    self.state_idx += 1;
                    if self.state_idx > MAX_STATES - 1 {
                        self.state_idx = 0;
                    }
                }
                InputEvent::Multi => {}
                _ => warn!("User clicked {:?} but it was unhandled", input)
            }
        } else {
            match self.state_idx {
            0 => {
                self.app_state.service_input(system, display, input)
            }
            _ => panic!("Unhandled state")
        }
        }
        
    }

    pub fn process(&mut self, display: &mut Ssd1351, system: &mut System) {
        match self.state_idx {
            2 => {
                self.clock_state.service(system);
                self.clock_state.render(system, display)
            },
            1 => {
                self.info_state.service(system);
                self.info_state.render(system, display)
            },
            0 => {
                self.app_state.service(system);
                self.app_state.render(system, display)
            }
            _ => panic!("Unhandled state")
        }
    }
}