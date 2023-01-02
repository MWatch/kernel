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

pub struct System<H: Host> {
    pub clock: H::Time,
    pub bms: H::BatteryManager,
    pub nm: NotificationManager,
    pub am: ApplicationManager,
    pub stats: H::RuntimeStatistics,
}

impl<H: Host> System<H> {
    pub fn new(time: H::Time, bms: H::BatteryManager, stats: H::RuntimeStatistics, am: ApplicationManager) -> Self {
        Self {
            clock: time,
            bms,
            stats,
            am,
            nm: NotificationManager::new(),
        }
    }
}

pub trait Host {
    type BatteryManager: BatteryManagement;
    type Time: Clock;
    type RuntimeStatistics: Statistics;
}

pub trait Display {
    fn framebuffer(&mut self) -> FrameBuffer;
}

pub trait Statistics {
    // TODO: below did not work :(
    // look into below in the future
    // https://github.com/jan-auer/dynfmt
    // type Statistics: Iterator<Item = (&'static str, &'a dyn core::fmt::Display)>;

    // TODO ideally formatting would be done inside the kernel, instead of passing this buffer but
    // we are limited by the technology of our time
    type Statistics: Iterator<Item = String<128>>; // TODO constrain to size of display

    fn stats(&self) -> Self::Statistics;

    fn is_idle(&mut self) -> bool {
        false
    }
}

pub trait NotificationInterface {
    fn nm(&mut self) -> &mut NotificationManager;
}
