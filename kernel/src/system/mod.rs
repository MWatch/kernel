use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb565};
use time::{Time, Date};

use crate::application::{application_manager::ApplicationManager, FrameBuffer};

use heapless::{String, consts::*};

use self::{bms::BatteryManagement, notification::NotificationManager};

pub mod bms;
pub mod input;
pub mod notification;
pub mod syscall;

pub trait Clock {
    // TODO don't use st hal concrete types
    fn get_time(&self) -> Time;
    fn set_time(&mut self, t: &Time);

    fn get_date(&self) -> Date;
    fn set_date(&mut self, t: &Date);
}

pub trait System:
    ApplicationInterface + BatteryManagement + Clock + NotificationInterface + Statistics
{
    fn is_idle(&mut self) -> bool {
        false
    }
}

pub trait ApplicationInterface {
    fn am(&mut self) -> &mut ApplicationManager;
}

pub trait Display: DrawTarget<Color = Rgb565> {
    fn framebuffer(&mut self) -> FrameBuffer;
}

pub trait Statistics {
    // TODO: below did not work :(
    // look into below in the future
    // https://github.com/jan-auer/dynfmt
    // type Statistics: Iterator<Item = (&'static str, &'a dyn core::fmt::Display)>;

    // TODO ideally formatting would be done inside the kernel, instead of passing this buffer but 
    // we are limited by the technology of our time
    type Statistics: Iterator<Item = String<U128>>; // TODO constrain to size of display

    fn stats(&self) -> Self::Statistics;
}

pub trait NotificationInterface {
    fn nm(&mut self) -> &mut NotificationManager;
}
