//! Window manager
//!
//! Handles app switching, between built in apps and custom apps

use crate::Ssd1351;
use crate::system::system::System;
use crate::application::states::{
                                    clock::ClockState,
                                    // info::InfoState,
                                    app::AppState,
                                };

use mwatch_kernel_api::InputEvent;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Signal {
    /// Next window
    Next,
    /// Previous window
    Previous,
    /// Home - return to the index 0
    Home
}

/// All built in states must implement this trait to be renderable by the WM
pub trait State: Default {
    /// To draw the state to the display
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal>;
    /// Allows the state to take control of inputs from the kernel
    fn input(&mut self, system: &mut System, display: &mut Ssd1351, input: InputEvent) -> Option<Signal>; //TODO can we remove the need for the display?
}

/// This state only exists whilst its running, and is destroyed on exit
pub trait ScopedState: State {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal>;
    /// Start 
    fn start(&mut self, system: &mut System);
    /// Is the application running yet?
    fn is_running(&self) -> bool;
    /// Stop
    fn stop(&mut self, system: &mut System);
}

const MAX_STATES: i8 = 2;

pub struct WindowManager 
{
    state_idx: i8,
    clock_state: ClockState,
    // info_state: InfoState,
    app_state: AppState,
}

impl WindowManager
{
    pub fn new() -> Self {
        Self {
            state_idx: 0,
            clock_state: ClockState::default(),
            // info_state: InfoState::default(),
            app_state: AppState::default(),
        }
    }

    pub fn process(&mut self, display: &mut Ssd1351, system: &mut System) {
        // TODO can we automate this boiler plate with a macro?
        match self.state_idx {
            0 => {
                if let Some(exit_code) = self.clock_state.render(system, display) {
                    self.handle_exit(exit_code);
                }
            },
            1 => {
                let exit_code = if self.app_state.is_running() {
                    self.app_state.render(system, display)
                } else {
                    self.app_state.preview(system, display)
                };
                if let Some(exit_code) = exit_code {
                    self.handle_exit(exit_code);
                }
            },
            _ => panic!("Unhandled state")
        }
    }

    pub fn service_input(&mut self, display: &mut Ssd1351, system: &mut System, input: InputEvent) {
        // TODO can we automate this boiler plate with a macro?
        match self.state_idx {
            0 => {
                if let Some(exit_code) = self.clock_state.input(system, display, input) {
                    self.handle_exit(exit_code);
                }
            },
            1 => {
                let exit_code = if self.app_state.is_running() {
                    self.app_state.render(system, display)
                } else {
                    match input {
                        InputEvent::Middle => {
                            self.app_state.start(system);
                            None
                        }
                        InputEvent::Left => Some(Signal::Previous),
                        InputEvent::Right => Some(Signal::Next),
                        _ => None
                    }
                };
                
                if let Some(exit_code) = exit_code {
                    self.handle_exit(exit_code);
                }
            }
            _ => panic!("Unhandled state")
        }
    }

    fn handle_exit(&mut self, code: Signal) {
        match code {
            Signal::Next => self.next(),
            Signal::Previous => self.prev(),
            Signal::Home => unimplemented!(),
        }
    }

    fn prev(&mut self) {
        self.state_idx -= 1;
        if self.state_idx < 0 {
            self.state_idx = MAX_STATES - 1;
        }
    }

    fn next(&mut self) {
        self.state_idx += 1;
        if self.state_idx > MAX_STATES - 1 {
            self.state_idx = 0;
        }
    }
}