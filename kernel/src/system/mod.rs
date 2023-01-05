use time::{Date, Time};

use crate::application::{application_manager::ApplicationManager, FrameBuffer};

use heapless::String;

use self::{bms::BatteryManagement, notification::NotificationManager};

pub mod bms;
pub mod input;
pub mod notification;
pub mod syscall;

pub trait Clock {
    fn get_time(&self) -> Time;
    fn set_time(&mut self, t: &Time);

    fn get_date(&self) -> Date;
    fn set_date(&mut self, t: &Date);
}

/// System
/// 
/// The [`System`] struct contains the interface to the [`Host`] system. The kernel uses this struct to operate.
pub struct System<H: Host> {
    pub clock: H::TimeProvider,
    pub bms: H::BatteryManager,
    pub stats: H::Statistics,
    pub nm: NotificationManager,
    pub am: ApplicationManager,
}

impl<H: Host> System<H> {
    pub fn new(time: H::TimeProvider, bms: H::BatteryManager, stats: H::Statistics, am: ApplicationManager) -> Self {
        Self {
            clock: time,
            bms,
            stats,
            am,
            nm: NotificationManager::new(),
        }
    }
}

/// Host
///
/// The main trait for defining the "host" system that the kernel runs on.
/// There are no trait methods here, just associated types which define what gets put into [`System`]
pub trait Host {
    type BatteryManager: BatteryManagement;
    type TimeProvider: Clock;
    type Statistics: Statistics;
    type Display: Display;
}

/// Display
/// 
/// Implement to retrieve the [`FrameBuffer`] for the [`Host`] display.
pub trait Display {
    fn framebuffer(&mut self) -> FrameBuffer;
}

pub trait Statistics {
    // TODO use GAT based design, see: https://gist.github.com/MabezDev/af1b05eb38aefebee6af1504ba54164f
    type Statistics: Iterator<Item = String<128>>;

    fn stats(&self) -> Self::Statistics;

    fn is_idle(&mut self) -> bool {
        false
    }
}
