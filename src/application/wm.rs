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
pub enum ExitCode {
    /// Next window
    Next,
    /// Previous window
    Previous,
    /// None stop running the app, or do nothing
    Exit,
    /// Home - return to the index 0
    Home
}

/// All built in states must implement this trait to be renderable by the WM
pub trait State: Default {
    /// To draw the state to the display
    fn render(&mut self, system: &mut System, display: &mut Ssd1351) -> Result<(), ExitCode>;
    /// Allows the state to take control of inputs from the kernel
    fn input(&mut self, system: &mut System, display: &mut Ssd1351, input: InputEvent) -> Result<(), ExitCode>; //TODO can we remove the need for the display?
}

/// This state only exists whilst its running, and is destroyed on exit
pub trait ScopedState: State {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, system: &mut System, display: &mut Ssd1351) -> Result<(), ExitCode>;
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
        match self.state_idx {
            1 => {
                self.clock_state.render(system, display).unwrap_or_else(|exit_code|{
                    self.handle_exit(exit_code)
                });
            },
            0 => {
                let exit_code = if self.app_state.is_running() {
                    self.app_state.render(system, display)
                } else {
                    self.app_state.preview(system, display)
                };
                exit_code.unwrap_or_else(|exit_code| {
                    self.handle_exit(exit_code);
                });
            },
            _ => panic!("Unhandled state")
        }
    }

    pub fn service_input(&mut self, display: &mut Ssd1351, system: &mut System, input: InputEvent) {
        match self.state_idx {
            1 => {
                self.clock_state.input(system, display, input).unwrap_or_else(|exit_code|{
                    self.handle_exit(exit_code)
                });
            },
            0 => {
                self.app_state.input(system, display, input).unwrap_or_else(|exit_code|{
                    self.handle_exit(exit_code)
                });
            }
            // 2 => {
            //     self.info_state.service_input().unwrap_or_else(|exit_code|{
            //         self.handle_exit(exit_code)
            //     });
            // },
            _ => panic!("Unhandled state")
        }
    }

    fn handle_exit(&mut self, code: ExitCode) {
        match code {
            ExitCode::Next => self.next(),
            ExitCode::Previous => self.prev(),
            ExitCode::Exit => {},
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