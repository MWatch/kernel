//! Window manager
//!
//! Handles app switching, between built in apps and custom apps

use crate::Ssd1351;
use crate::system::system::System;
use crate::application::states::{
                                    clock::ClockState,
                                    info::InfoState,
                                    app::AppState,
                                    uop::UopState,
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

/// Marker trait for static states
pub trait StaticState: State {

}

/// This state only exists whilst its running, and is destroyed on exit
pub trait ScopedState: State {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, system: &mut System, display: &mut Ssd1351) -> Option<Signal>;
    /// Start 
    fn start(&mut self, system: &mut System);
    /// Is the application running yet?
    fn is_running(&self, system: &mut System) -> bool;
    /// Stop
    fn stop(&mut self, system: &mut System);
}

const MAX_STATES: i8 = 3;

pub struct WindowManager 
{
    state_idx: i8,
    clock_state: ClockState,
    info_state: InfoState,
    app_state: AppState,
    uop_state: UopState,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self {
            state_idx: 0,
            clock_state: ClockState::default(),
            info_state: InfoState::default(),
            app_state: AppState::default(),
            uop_state: UopState::default(),
        }
    }
}

impl WindowManager
{

    pub fn process(&mut self, system: &mut System, display: &mut Ssd1351) {
        let signal = match self.state_idx {
            0 => {
                WindowManager::static_state_render(&mut self.clock_state, system, display)
            },
            1 => {
                WindowManager::scoped_state_render(&mut self.app_state, system, display)
            },
            2 => {
                WindowManager::static_state_render(&mut self.info_state, system, display)
            },
            3 => {
                WindowManager::static_state_render(&mut self.uop_state, system, display)
            },
            _ => panic!("Unhandled state")
        };

        if let Some(signal) = signal {
            self.handle_exit(signal);
        }
    }

    pub fn service_input(&mut self, system: &mut System, display: &mut Ssd1351, input: InputEvent) {
        let signal = match self.state_idx {
            0 => {
                WindowManager::static_state_input(&mut self.clock_state, system, display, input)
            },
            1 => {
                WindowManager::scoped_state_input(&mut self.app_state, system, display, input)
            }
            2 => {
                WindowManager::static_state_input(&mut self.info_state, system, display, input)
            },
            3  => {
                WindowManager::static_state_input(&mut self.uop_state, system, display, input)
            },

            _ => panic!("Unhandled state")
        };

        if let Some(signal) = signal {
            self.handle_exit(signal);
        }
    }

    fn handle_exit(&mut self, code: Signal) {
        match code {
            Signal::Next => self.next(),
            Signal::Previous => self.prev(),
            Signal::Home => self.state_idx = 0,
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

    fn static_state_render<S>(state: &mut S, system: &mut System, display: &mut Ssd1351) -> Option<Signal> 
        where S : StaticState
    {
        state.render(system, display)
    }

    fn scoped_state_render<S>(state: &mut S, system: &mut System, display: &mut Ssd1351) -> Option<Signal> 
        where S : ScopedState
    {
        if state.is_running(system) {
            state.render(system, display)
        } else {
            state.preview(system, display)
        }
    }

    fn static_state_input<S>(state: &mut S, system: &mut System, display: &mut Ssd1351, input: InputEvent) -> Option<Signal> 
        where S : StaticState
    {
        state.input(system, display, input)
    }

    fn scoped_state_input<S>(state: &mut S, system: &mut System, display: &mut Ssd1351, input: InputEvent) -> Option<Signal> 
        where S : ScopedState
    {
        if state.is_running(system) {
            state.input(system, display, input)
        } else {
            match input {
                InputEvent::Middle => {
                    state.start(system);
                    None
                }
                InputEvent::Left => Some(Signal::Previous),
                InputEvent::Right => Some(Signal::Next),
                _ => None
            }
        }
    }
}
