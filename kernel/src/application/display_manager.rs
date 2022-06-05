//! Window manager
//!
//! Handles app switching, between built in apps and custom apps

use embedded_graphics::{pixelcolor::PixelColorU16, Drawing};

use crate::{application::{
    states::{
        clock::ClockState,
        info::InfoState,
        app::AppState,
        uop::UopState,
        mwatch::MWState,
        notifications::NotificationState,
    },
    states::prelude::*
}, system::{input::InputEvent, System}};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Signal {
    /// Next window
    Next,
    /// Previous window
    Previous,
    /// Home - return to the index 0
    Home
}

const MAX_STATES: i8 = 6;

/// The display manager
pub struct DisplayManager 
{
    state_idx: i8,
    clock_state: ClockState,
    info_state: InfoState,
    app_state: AppState,
    uop_state: UopState,
    mwatch_state: MWState,
    notification_state: NotificationState,
}

impl Default for DisplayManager {

    /// Create the display manager
    fn default() -> Self {
        Self {
            state_idx: 0,
            clock_state: ClockState::default(),
            info_state: InfoState::default(),
            app_state: AppState::default(),
            uop_state: UopState::default(),
            mwatch_state: MWState::default(),
            notification_state: NotificationState::default(),
        }
    }
}

impl DisplayManager
{

    /// Services the current application
    pub fn process(&mut self, system: &mut impl System, display: &mut impl Drawing<PixelColorU16>) {
        let signal = match self.state_idx {
            0 => {
                DisplayManager::static_state_render(&mut self.clock_state, system, display)
            },
            1 => {
                DisplayManager::scoped_state_render(&mut self.app_state, system, display)
            },
            2 => {
                DisplayManager::scoped_state_render(&mut self.notification_state, system, display)
            },
            3 => {
                DisplayManager::static_state_render(&mut self.mwatch_state, system, display)
            },
            4 => {
                DisplayManager::static_state_render(&mut self.uop_state, system, display)
            },
            5 => {
                DisplayManager::static_state_render(&mut self.info_state, system, display)
            },
            _ => panic!("Unhandled state")
        };

        if let Some(signal) = signal {
            self.handle_exit(signal);
        }
    }

    /// Services input to the current application
    pub fn service_input(&mut self, system: &mut impl System, input: InputEvent) {
        let signal = match self.state_idx {
            0 => {
                DisplayManager::static_state_input(&mut self.clock_state, system, input)
            },
            1 => {
                DisplayManager::scoped_state_input(&mut self.app_state, system, input)
            }
            2 => {
                DisplayManager::scoped_state_input(&mut self.notification_state, system, input)
            },
            3  => {
                DisplayManager::static_state_input(&mut self.mwatch_state, system, input)
            },
            4  => {
                DisplayManager::static_state_input(&mut self.uop_state, system, input)
            },
            5 => {
                DisplayManager::static_state_input(&mut self.info_state, system, input)
            },
            _ => panic!("Unhandled state")
        };

        if let Some(signal) = signal {
            self.handle_exit(signal);
        }
    }

    /// Handle the exit code of a running application
    fn handle_exit(&mut self, code: Signal) {
        match code {
            Signal::Next => self.next(),
            Signal::Previous => self.prev(),
            Signal::Home => self.state_idx = 0,
        }
    }

    /// Move to the previous state in a wrapping fashion
    fn prev(&mut self) {
        self.state_idx -= 1;
        if self.state_idx < 0 {
            self.state_idx = MAX_STATES - 1;
        }
    }

    /// Move to the next state in a wrapping fashion
    fn next(&mut self) {
        self.state_idx += 1;
        if self.state_idx > MAX_STATES - 1 {
            self.state_idx = 0;
        }
    }

    /// Render a static state
    fn static_state_render<S>(state: &mut S, system: &mut impl System, display: &mut impl Drawing<PixelColorU16>) -> Option<Signal> 
        where S : StaticState
    {
        state.render(system, display)
    }

    /// Render a scoped state, this state may or may not be running hence we have different functionality
    /// depending on the `is_running()` state
    fn scoped_state_render<S>(state: &mut S, system: &mut impl System, display: &mut impl Drawing<PixelColorU16>) -> Option<Signal> 
        where S : ScopedState
    {
        if state.is_running(system) {
            state.render(system, display)
        } else {
            state.preview(system, display)
        }
    }

    /// Handle input for a static state
    fn static_state_input<S>(state: &mut S, system: &mut impl System, input: InputEvent) -> Option<Signal> 
        where S : StaticState
    {
        state.input(system, input)
    }

    /// Handle the input for a scoped state, this state may or may not be running hence we have different functionality
    /// depending on the `is_running()` state
    fn scoped_state_input<S>(state: &mut S, system: &mut impl System, input: InputEvent) -> Option<Signal> 
        where S : ScopedState
    {
        if state.is_running(system) {
            state.input(system, input)
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


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn dm_state_wraps() {
        let mut dm = DisplayManager::default();
        for _ in 0..MAX_STATES {
            dm.next();
        }
        // after we iterate through all states, we should be back at the begining
        assert_eq!(dm.state_idx, 0)
    }

    #[test]
    fn dm_state_prev_wraps() {
        let mut dm = DisplayManager::default();
        dm.prev();
        // going back on 0 should put us at the last state, of couse the index starts at zero so we take one
        assert_eq!(dm.state_idx, MAX_STATES - 1)
    }
}