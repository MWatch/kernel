#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use embassy_futures::select::{select3, Either3};
use embassy_time::{Timer, Duration};
use esp32s3_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc, embassy, gpio::{PullUp, Input, Gpio4, Gpio5, Gpio6}, IO};
use esp_backtrace as _;
use esp_println;
use static_cell::StaticCell;
use embedded_hal_async::digital::Wait;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[xtensa_lx_rt::entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
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

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

     // Async requires the GPIO interrupt to wake futures
    esp32s3_hal::interrupt::enable(
        esp32s3_hal::peripherals::Interrupt::GPIO,
        esp32s3_hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    embassy::init(
        &clocks,
        esp32s3_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(idle()).ok();
        spawner.spawn(input(io.pins.gpio4.into_pull_up_input(), io.pins.gpio5.into_pull_up_input(), io.pins.gpio6.into_pull_up_input())).ok();
    })
}

#[embassy_executor::task]
async fn idle() {
    loop {
        log::info!("Bing!");
        Timer::after(Duration::from_millis(3_000)).await;
    }
}

#[embassy_executor::task]
async fn input(mut left: Gpio4<Input<PullUp>>, mut middle: Gpio5<Input<PullUp>>, mut right: Gpio6<Input<PullUp>>) {
    log::info!("Waiting for Inputs");
    loop {
        match select3(left.wait_for_falling_edge(), middle.wait_for_falling_edge(), right.wait_for_falling_edge()).await {
            Either3::First(_) => log::info!("LEFT!"),
            Either3::Second(_) => log::info!("MIDDLE!"),
            Either3::Third(_) => log::info!("RIGHT!"),
        };
    }
}