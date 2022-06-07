//! System
//! 
//! Performs housekeeping of system hardware and provides a nice sofware abstraction to read / manipulate it

use embedded_graphics::{pixelcolor::{Rgb565}, prelude::OriginDimensions};
use heapless::{String, consts::*};
use mwatch_kernel::{system::{notification::NotificationManager, Display}, application::Table};
use stm32l4xx_hal::{rtc::Rtc, prelude::_stm32l4_hal_datetime_U32Ext};
use time::{Time, Date};
use crate::{application::application_manager::ApplicationManager, types::{BatteryManagementInterface, StandbyStatusPin, ChargeStatusPin, Ssd1351}, bms::BatteryManagement};
use core::fmt::Write;


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
    fn is_idle(&mut self) -> bool {
        (self.ss().idle_count / SYSTICK_HZ) > IDLE_TIMEOUT_SECONDS
    }
}

impl mwatch_kernel::system::NotificationInterface for System {
    fn nm(&mut self) -> &mut NotificationManager {
        self.nm()
    }
}

impl mwatch_kernel::system::ApplicationInterface for System {
    unsafe fn install_os_table(&mut self) {
        static mut TBL : Table = Table {
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

    pub unsafe extern "C" fn print(_context: *mut Context, ptr: *const u8, len: usize) -> i32 {
        info!("[APP] - {}", core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)));
        0
    }
}

impl mwatch_kernel::system::Clock for System {
    fn get_time(&self) -> Time {
        let t = self.rtc.get_time();
        // NOTE(unwrap): we assume rtc gives us a valid reading always
        Time::from_hms(t.hours as u8, t.minutes as u8, t.seconds as u8).unwrap()
    }

    fn set_time(&mut self, t: &Time) {
        let t = stm32l4xx_hal::datetime::Time::new((t.hour() as u32).hours(), (t.minute() as u32).minutes(), (t.second() as u32).seconds(), false);
        self.rtc.set_time(&t);
    }

    fn get_date(&self) -> Date {
        let d = self.rtc.get_date();
        // NOTE(unwrap): we assume rtc gives us a valid reading always
        Date::from_calendar_date(d.year as i32, (d.month as u8).try_into().unwrap(), d.date as u8).unwrap()
    }

    fn set_date(&mut self, d: &Date) {
        let d = stm32l4xx_hal::datetime::Date::new(0.day(), (d.day() as u32).date(), (d.month() as u32).month(), (d.year() as u32).year());
        self.rtc.set_date(&d);
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

impl mwatch_kernel::system::Statistics for System {
    type Statistics = StatsIter;

    fn stats(&self) -> Self::Statistics {
        StatsIter {
            stats: self.stats,
            index: 0
        }
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

pub struct StatsIter {
    index: usize,
    stats: Stats,
}

impl Iterator for StatsIter {
    type Item = String<U128>;

    /// Anything that needs to be printed should be produced by this iterator
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = String::new();
        match self.index {
            0 => write!(buffer, "CPU_USAGE: {:.02}%", self.stats.cpu_usage).unwrap(),
            1 => write!(buffer, "TSC EVENTS: {}/s", self.stats.tsc_events).unwrap(),
            2 => write!(buffer, "TSC THRES: {}", self.stats.tsc_threshold).unwrap(),
            _ => return None
        }
        self.index += 1;
        Some(buffer)
    }
}

pub struct DisplayWrapper(pub Ssd1351);

impl Display for DisplayWrapper {
    fn framebuffer(&mut self) -> mwatch_kernel::application::FrameBuffer {
        let size = self.0.size();
        let buffer = self.0.fb_mut();

        mwatch_kernel::application::FrameBuffer::new(buffer.as_mut_ptr(), buffer.len(), size.width as u8, size.height as u8)
    }
}

impl embedded_graphics::draw_target::DrawTarget for DisplayWrapper {
    type Color = Rgb565;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        self.0.draw_iter(pixels)
    }
}

impl embedded_graphics::geometry::Dimensions for DisplayWrapper {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        self.0.bounding_box()
    }
}

impl core::ops::Deref for DisplayWrapper {
    type Target = Ssd1351;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for DisplayWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}