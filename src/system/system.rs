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
}

impl System {
    pub fn new(rtc: Rtc, bms: BatteryManagement, nm: NotificationManager, am: ApplicationManager) -> Self {
        Self {
            rtc: rtc,
            bms: bms,
            nm: nm,
            am: am
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

