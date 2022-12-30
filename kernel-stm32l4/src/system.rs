//! System
//!
//! Performs housekeeping of system hardware and provides a nice sofware abstraction to read / manipulate it

use crate::{
    bms::BatteryManagement,
    types::{BatteryManagementInterface, ChargeStatusPin, Ssd1351, StandbyStatusPin},
};
use core::fmt::Write;
use embedded_graphics::{pixelcolor::Rgb565, prelude::OriginDimensions};
use heapless::String;
use mwatch_kernel::system::{Display, Host};
use stm32l4xx_hal::{prelude::_stm32l4_hal_datetime_U32Ext, rtc::Rtc};
use time::{Date, Time};

pub const DMA_HALF_BYTES: usize = 64;

pub const CPU_USAGE_POLL_HZ: u32 = 1; // hz
pub const SYSTICK_HZ: u32 = 10; // hz
pub const TSC_HZ: u32 = 8 * 3; // 8 polls per second (for 3 inputs)

pub const SYS_CLK_HZ: u32 = 80_000_000;
pub const SPI_MHZ: u32 = SYS_CLK_HZ / 20_000_000;
pub const I2C_KHZ: u32 = 100;

pub const IDLE_TIMEOUT_SECONDS: u32 = 15;


pub struct KernelHost;

impl Host for KernelHost {
    type BatteryManager = BatteryManagement<BatteryManagementInterface, ChargeStatusPin, StandbyStatusPin>;
    type Time = RtcWrapper;
    type RuntimeStatistics = Stats;
}

pub mod abi {
    use embedded_graphics::draw_target::DrawTarget;
    use mwatch_kernel::application::Context;

    /// Assumes control over the display, it is up to us to make sure the display is not borrowed by anything else
    pub unsafe extern "C" fn draw_pixel(context: *mut Context, x: u8, y: u8, colour: u16) -> i32 {
        let ctx = &mut *context;
        if let Some(ref mut display) = ctx.framebuffer {
            display.draw_iter(
                [embedded_graphics::Pixel(
                    embedded_graphics::prelude::Point::new(x as i32, y as i32),
                    embedded_graphics::pixelcolor::raw::RawU16::from(colour).into(),
                )]
                .into_iter(),
            ).ok(); // TODO handle error
        } else {
            panic!("Display invoked in an invalid state. Applications can only use the display within update.")
        }
        0
    }

    pub unsafe extern "C" fn print(_context: *mut Context, ptr: *const u8, len: usize) -> i32 {
        info!(
            "[APP] - {}",
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len))
        );
        0
    }
}

#[repr(transparent)]
pub struct RtcWrapper(pub Rtc);

impl mwatch_kernel::system::Clock for RtcWrapper {
    fn get_time(&self) -> Time {
        let t = self.0.get_time();
        // NOTE(unwrap): we assume rtc gives us a valid reading always
        Time::from_hms(t.hours as u8, t.minutes as u8, t.seconds as u8).unwrap()
    }

    fn set_time(&mut self, t: &Time) {
        let t = stm32l4xx_hal::datetime::Time::new(
            (t.hour() as u32).hours(),
            (t.minute() as u32).minutes(),
            (t.second() as u32).seconds(),
            false,
        );
        self.0.set_time(&t);
    }

    fn get_date(&self) -> Date {
        let d = self.0.get_date();
        // NOTE(unwrap): we assume rtc gives us a valid reading always
        Date::from_calendar_date(
            d.year as i32,
            (d.month as u8).try_into().unwrap(),
            d.date as u8,
        )
        .unwrap()
    }

    fn set_date(&mut self, d: &Date) {
        let d = stm32l4xx_hal::datetime::Date::new(
            0.day(),
            (d.day() as u32).date(),
            (d.month() as u32).month(),
            (d.year() as u32).year(),
        );
        self.0.set_date(&d);
    }
}

impl mwatch_kernel::system::bms::BatteryManagement for BatteryManagement<BatteryManagementInterface, ChargeStatusPin, StandbyStatusPin> {
    fn state(&self) -> mwatch_kernel::system::bms::State {
        self.state()
    }

    fn soc(&mut self) -> u16 {
        self.soc()
    }
}

impl mwatch_kernel::system::Statistics for Stats {
    type Statistics = StatsIter;

    fn stats(&self) -> Self::Statistics {
        StatsIter {
            stats: *self,
            index: 0,
        }
    }

    fn is_idle(&mut self) -> bool {
        (self.idle_count / SYSTICK_HZ) > IDLE_TIMEOUT_SECONDS
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
    type Item = String<128>;

    /// Anything that needs to be printed should be produced by this iterator
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = String::new();
        match self.index {
            0 => write!(buffer, "CPU_USAGE: {:.02}%", self.stats.cpu_usage).unwrap(),
            1 => write!(buffer, "TSC EVENTS: {}/s", self.stats.tsc_events).unwrap(),
            2 => write!(buffer, "TSC THRES: {}", self.stats.tsc_threshold).unwrap(),
            _ => return None,
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

        mwatch_kernel::application::FrameBuffer::new(
            buffer.as_mut_ptr(),
            buffer.len(),
            size.width as u8,
            size.height as u8,
        )
    }
}

impl embedded_graphics::draw_target::DrawTarget for DisplayWrapper {
    type Color = Rgb565;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
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
