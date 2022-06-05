//! System
//! 
//! Performs housekeeping of system hardware and provides a nice sofware abstraction to read / manipulate it

use mwatch_kernel::{system::{notification::NotificationManager}, application::Table};
use stm32l4xx_hal::rtc::Rtc;
use crate::{application::application_manager::ApplicationManager, types::{BatteryManagementInterface, StandbyStatusPin, ChargeStatusPin}, bms::BatteryManagement};


pub const DMA_HALF_BYTES: usize = 64;

pub const CPU_USAGE_POLL_HZ: u32 = 1; // hz
pub const SYSTICK_HZ: u32 = 3; // hz
pub const TSC_HZ: u32 = 8 * 3; // 8 polls per second (for 3 inputs)

pub const SYS_CLK_HZ: u32 = 16_000_000;
pub const SPI_MHZ: u32 = SYS_CLK_HZ / 2_000_000; // spi is always half of sysclock
pub const I2C_KHZ: u32 = 100;

pub const IDLE_TIMEOUT_SECONDS: u32 = 15;

/// A grouping of core sysem peripherals
pub struct System {
    rtc: Rtc,
    bms: BatteryManagement<BatteryManagementInterface, ChargeStatusPin, StandbyStatusPin>,
    nm: NotificationManager,
    am: ApplicationManager,
    stats: Stats,
}

impl mwatch_kernel::system::System for System {

    fn nm(&mut self) -> &mut NotificationManager {
        self.nm()
    }

    fn is_idle(&mut self) -> bool {
        (self.ss().idle_count / SYSTICK_HZ) > IDLE_TIMEOUT_SECONDS
    }
}

impl mwatch_kernel::system::ApplicationInterface for System {
    unsafe fn install_os_table(&mut self) {
        static mut TBL : Table = Table {
            draw_pixel: abi::draw_pixel,
            print: abi::print,
        };
        Table::install(&mut TBL)
    }

    fn am(&mut self) -> &mut ApplicationManager {
        self.am()
    }
}

mod abi {
    use mwatch_kernel::application::Context;
    use crate::types::Ssd1351;

    /// Assumes control over the display, it is up to use to make sure the display is not borrowed by anything else
    pub unsafe extern "C" fn draw_pixel(context: *mut Context, x: u8, y: u8, colour: u16) -> i32 {
        let ctx = &mut *context;
        if let Some(display) = ctx.display {
            // TODO verify this is correct!
            // first cast the void pointer back to the concrete type
            // dereference to get back to the mutable reference
            let display = &mut *(display as *mut Ssd1351); 
            display.set_pixel(u32::from(x), u32::from(y), colour);
        } else {
            panic!("Display invoked in an invalid state. Applications can only use the display within update.")
        }
        0
    }

    pub unsafe extern "C" fn print(_context: *mut Context, ptr: *const u8, len: usize) -> i32 {
        info!("[APP] - {}", core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)));
        0
    }
}

impl mwatch_kernel::system::Clock for System {
    fn get_time(&self) -> stm32l4xx_hal::datetime::Time {
        self.rtc.get_time()
    }

    fn set_time(&mut self, t: &stm32l4xx_hal::datetime::Time) {
        self.rtc.set_time(t)
    }

    fn get_date(&self) -> stm32l4xx_hal::datetime::Date {
        self.rtc.get_date()
    }

    fn set_date(&mut self, d: &stm32l4xx_hal::datetime::Date) {
        self.rtc.set_date(d)
    }
}

impl mwatch_kernel::system::bms::BatteryManagement for System {
    fn state(&self) -> mwatch_kernel::system::bms::State {
        self.bms.state()
    }

    fn soc(&mut self) -> u16 {
        self.bms.soc()
    }
}

impl System {
    pub fn new(rtc: Rtc, bms: BatteryManagement<BatteryManagementInterface, ChargeStatusPin, StandbyStatusPin>, nm: NotificationManager, am: ApplicationManager) -> Self {
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
    pub fn bms(&mut self) -> &mut BatteryManagement<BatteryManagementInterface, ChargeStatusPin, StandbyStatusPin> {
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
    pub tsc_events: u32,
    pub idle_count: u32,
    pub tsc_threshold: u16,
}
    

impl Default for Stats {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            tsc_events: 0,
            idle_count: 0,
            tsc_threshold: 0,
        }
    }
}

