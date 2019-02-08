//! System.rs
//! 
//! Performs housekeeping of system hardware and provides a nice sofware abstraction to read / manipulate it


pub struct System {
    rtc: stm32l4xx_hal::rtc::Rtc,
    bms: crate::system::bms::BatteryManagement
}

fn get_free_stack() -> usize {
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