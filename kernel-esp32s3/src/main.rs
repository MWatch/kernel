#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use embassy_time::{Timer, Duration};
use esp32s3_hal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc, embassy};
use esp_backtrace as _;
use esp_println;
use static_cell::StaticCell;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[xtensa_lx_rt::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    esp_println::logger::init_logger(log::LevelFilter::Info);

    log::info!("\r\n\r\n  /\\/\\/ / /\\ \\ \\__ _| |_ ___| |__  \r\n /    \\ \\/  \\/ / _` | __/ __| '_ \\ \r\n/ /\\/\\ \\  /\\  / (_| | || (__| | | |\r\n\\/    \\/\\/  \\/ \\__,_|\\__\\___|_| |_|\r\n                                   \r\n");
    log::info!("Copyright Scott Mabin 2022");

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    embassy::init(
        &clocks,
        esp32s3_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(idle()).ok();
    })
}

#[embassy_executor::task]
async fn idle() {
    loop {
        log::info!("Bing!");
        Timer::after(Duration::from_millis(3_000)).await;
    }
}
