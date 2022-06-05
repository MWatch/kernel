//! Built in states

pub mod prelude;

pub mod clock;
// pub mod info;
pub mod app;
pub mod mwatch;
pub mod uop;
pub mod notifications;



use embedded_graphics::{Drawing, pixelcolor::PixelColorU16};
use prelude::*;

use crate::system::{System, input::InputEvent};

/// All built in states must implement this trait to be renderable by the WM
pub trait State: Default {
    /// To draw the state to the display
    fn render(&mut self, system: &mut impl System, display: &mut impl Drawing<PixelColorU16>) -> Option<Signal>;
    /// Allows the state to take control of inputs from the kernel
    fn input(&mut self, system: &mut impl System, input: InputEvent) -> Option<Signal>;
}

/// Marker trait for static states
pub trait StaticState: State {}

/// This state only exists whilst its running, and is destroyed on exit
pub trait ScopedState: State {
    /// Render a preview or Icon before launching the whole application
    fn preview(&mut self, system: &mut impl System, display: &mut impl Drawing<PixelColorU16>) -> Option<Signal>;
    /// Start 
    fn start(&mut self, system: &mut impl System);
    /// Is the application running yet?
    fn is_running(&self, system: &mut impl System) -> bool;
    /// Stop
    fn stop(&mut self, system: &mut impl System);
}