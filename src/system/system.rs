//! System.rs
//! 
//! Performs housekeeping of system hardware and provides a nice sofware abstraction to read / manipulate it

use stm32l4xx_hal::rtc::Rtc;
use crate::system::bms::BatteryManagement;
use crate::system::notification::NotificationManager;
use crate::application::application_manager::ApplicationManager;

pub struct System {
    rtc: Rtc,
    bms: BatteryManagement,
    nm: NotificationManager,
    am: ApplicationManager,
    stats: Stats,
}

impl System {
    pub fn new(rtc: Rtc, bms: BatteryManagement, nm: NotificationManager, am: ApplicationManager) -> Self {
        Self {
            rtc,
            bms,
            nm,
            am,
            stats: Stats::default(),
        }
    }

    /// Real time clock
    pub fn rtc(&mut self) -> &mut Rtc {
        &mut self.rtc
    }

    /// Battery management
    pub fn bms(&mut self) -> &mut BatteryManagement {
        &mut self.bms
    }

    /// Application manager
    pub fn am(&mut self) -> &mut ApplicationManager {
        &mut self.am
    }

    /// Notification Manager
    pub fn nm(&mut self) -> &mut NotificationManager {
        &mut self.nm
    }

    /// System stats
    pub fn ss(&mut self) -> &mut Stats {
        &mut self.stats
    }

    pub fn get_free_stack() -> usize {
        unsafe {
            extern "C" {
                static __ebss: u32;
                static __sdata: u32;
            }
            let ebss = &__ebss as *const u32 as usize;
            let start = &__sdata as *const u32 as usize;
            let total = ebss - start;
            (16 * 1024) - total // ram for stack in linker script
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub cpu_usage: f32,
    pub tsc_events: u32
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            tsc_events: 0,
        }
    }
}

