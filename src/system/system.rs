//! System.rs
//! 
//! Performs housekeeping of system hardware and provides a nice sofware abstraction to read / manipulate it

use stm32l4xx_hal::rtc::Rtc;
use crate::system::bms::BatteryManagement;

pub struct System {
    rtc: Rtc,
    bms: BatteryManagement
}

impl System {
    pub fn new(rtc: Rtc, bms: BatteryManagement) -> Self {
        Self {
            rtc: rtc,
            bms: bms,
        }
    }

    pub fn rtc(&mut self) -> &mut Rtc {
        &mut self.rtc
    }

    pub fn bms(&mut self) -> &mut BatteryManagement {
        &mut self.bms
    }

    pub fn process(&mut self) {
        self.bms.process();
    }

    pub fn get_free_stack(&self) -> usize {
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

