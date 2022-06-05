use stm32l4xx_hal::datetime::{Time, Date};

use self::notification::NotificationManager;

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

pub trait BatteryManagement {
    fn state(&self) -> self::bms::State;
    fn soc(&self) -> u16;
}

pub trait System: Clock + BatteryManagement {

    fn is_idle(&self) -> bool {
        false
    }

    fn nm(&mut self) -> NotificationManager;
}