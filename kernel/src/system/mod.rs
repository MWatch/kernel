use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb565};
use stm32l4xx_hal::datetime::{Time, Date};

use crate::application::{application_manager::ApplicationManager, FrameBuffer};

use self::{notification::NotificationManager, bms::BatteryManagement};

pub mod input;
pub mod bms;
pub mod notification;
pub mod syscall;

pub trait Clock {
    // TODO don't use st hal concrete types
    fn get_time(&self) -> Time;
    fn set_time(&mut self, t: &Time);

    fn get_date(&self) -> Date;
    fn set_date(&mut self, t: &Date);    
}

pub trait System: ApplicationInterface + BatteryManagement + Clock {

    fn is_idle(&mut self) -> bool {
        false
    }

    fn nm(&mut self) -> &mut NotificationManager;
}

pub trait ApplicationInterface {
    unsafe fn install_os_table(&mut self);

    fn am(&mut self) -> &mut ApplicationManager;
}

pub trait Display: DrawTarget<Color = Rgb565> {
    fn framebuffer(&mut self) -> FrameBuffer;
}