#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[macro_use]
extern crate cortex_m;
extern crate rtfm;
// extern crate panic_itm;
extern crate cortex_m_rt as rt;
extern crate embedded_graphics;
extern crate heapless;
extern crate hm11;
extern crate max17048;
#[cfg(not(test))]
extern crate panic_semihosting;
extern crate ssd1351;
#[macro_use]
extern crate log;

mod application;
mod ingress;
mod system;


use mwatch_kernel_api::{hal, BatteryManagementIC, LeftButton, MiddleButton, RightButton, Ssd1351, InputEvent};
use crate::hal::datetime::Date;
use crate::hal::delay::Delay;
use crate::hal::dma::{dma1, CircBuffer, Event};
use crate::hal::i2c::I2c;
use crate::hal::prelude::*;
use crate::hal::rtc::Rtc;
use crate::hal::serial::{Event as SerialEvent, Serial};
use crate::hal::spi::Spi;
use crate::hal::timer::{Event as TimerEvent, Timer};
use crate::hal::tsc::{
    ClockPrescaler as TscClockPrescaler, Config as TscConfig, Tsc,
};
use crate::rt::exception;
use crate::rt::ExceptionFrame;
use heapless::consts::*;
use heapless::spsc::Queue;
use rtfm::app;

use ssd1351::builder::Builder;
use ssd1351::mode::GraphicsMode;
use ssd1351::prelude::*;
use ssd1351::properties::DisplayRotation;

use cortex_m::asm;
use cortex_m::peripheral::DWT;
use hm11::command::Command;
use hm11::Hm11;
use max17048::Max17048;

use crate::ingress::ingress_manager::IngressManager;
use crate::system::notification::BUFF_COUNT;
use crate::system::notification::Notification;
use crate::system::notification::NotificationManager;

use crate::application::application_manager::ApplicationManager;
use crate::system::input::InputManager;
use crate::system::bms::BatteryManagement;
use crate::system::system::System;

use crate::application::wm::WindowManager;

use cortex_m_log::log::{Logger, trick_init};
use cortex_m_log::destination::Itm as ItmDestination;
use cortex_m_log::printer::itm::InterruptSync as InterruptSyncItm;


type LoggerType = cortex_m_log::log::Logger<cortex_m_log::printer::itm::ItmSync<cortex_m_log::modes::InterruptFree>>;

type ChargeStatusPin = hal::gpio::gpioa::PA12<hal::gpio::Input<hal::gpio::PullUp>>;
type StandbyStatusPin = hal::gpio::gpioa::PA11<hal::gpio::Input<hal::gpio::PullUp>>;
type TouchSenseController = hal::tsc::Tsc<hal::gpio::gpiob::PB4<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::OpenDrain>>>>;
type BluetoothConnectedPin = hal::gpio::gpioa::PA8<hal::gpio::Input<hal::gpio::Floating>>;

const DMA_HAL_SIZE: usize = 64;
const SYS_CLK: u32 = 16_000_000;
const CPU_USAGE_POLL_FREQ: u32 = 1; // hz

#[cfg(feature = "itm")]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;
#[cfg(not(feature = "itm"))]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Off;

#[cfg_attr(not(test), app(device = crate::hal::stm32))]
const APP: () = {
    #[cfg(not(test))]
    static mut CB: CircBuffer<&'static mut [[u8; DMA_HAL_SIZE]; 2], dma1::C6> = ();
    #[cfg(not(test))]
    static mut IMNG: IngressManager = ();
    #[cfg(not(test))]
    static mut INPUT_MGR: InputManager = ();
    #[cfg(not(test))]
    static mut WMNG: WindowManager = ();
    static mut NOTIFICATIONS: [Notification; crate::BUFF_COUNT] =
        [Notification::default(); crate::BUFF_COUNT];
    static mut RB: Option<Queue<u8, heapless::consts::U512>> = None;
    #[cfg(not(test))]
    static mut USART2_RX: hal::serial::Rx<hal::stm32l4::stm32l4x2::USART2> = ();
    #[cfg(not(test))]
    static mut DISPLAY: Ssd1351 = ();

    #[cfg(not(test))]
    static mut BT_CONN: BluetoothConnectedPin = ();
    #[cfg(not(test))]
    static mut SYSTEM: System = ();
    static mut DMA_BUFFER: [[u8; crate::DMA_HAL_SIZE]; 2] = [[0; crate::DMA_HAL_SIZE]; 2];
    #[cfg(not(test))]
    static mut SYS_TICK: hal::timer::Timer<hal::stm32::TIM2> = ();
    #[cfg(not(test))]
    static mut TIM6: hal::timer::Timer<hal::stm32::TIM6> = ();

    static mut SLEEP_TIME: u32 = 0;
    #[cfg(not(test))]
    static mut TIM7: hal::timer::Timer<hal::stm32::TIM7> = ();
    static mut TSC_EVENTS: u32 = 0;

    static mut LOGGER: Option<LoggerType> = None;

    #[link_section = ".fb_section.fb"]
    static mut FRAME_BUFFER: [u8; 32 * 1024] = [0u8; 32 * 1024];
    #[link_section = ".app_section.data"]
    static mut APPLICATION_RAM: [u8; 16 * 1024] = [0u8; 16 * 1024];
    #[cfg(not(test))]
    #[init(resources = [RB, NOTIFICATIONS, DMA_BUFFER, APPLICATION_RAM, FRAME_BUFFER, LOGGER])]
    fn init() {
        core.DCB.enable_trace(); // required for DWT cycle clounter to work when not connected to the debugger
        core.DWT.enable_cycle_counter();
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        // let clocks = rcc
        //     .cfgr
        //     .sysclk(SYS_CLK.hz())
        //     .pclk1(32.mhz())
        //     .pclk2(32.mhz())
        //     .freeze(&mut flash.acr); // 31% cpu usage~
        
        // let clocks = rcc
        //     .cfgr
        //     .sysclk(2.mhz())
        //     .pclk1(2.mhz())
        //     .pclk2(2.mhz())
        //     .lsi(true)
        //     .msi(stm32l4xx_hal::rcc::MsiFreq::RANGE2M)
        //     .freeze(&mut flash.acr); // this config is too slow - cant use lprun etc
        
        let clocks = rcc.cfgr.lsi(true).freeze(&mut flash.acr); // 63% cpu usage~

        // initialize the logging framework
        let itm = core.ITM;
        let logger = Logger {
            inner: InterruptSyncItm::new(ItmDestination::new(itm)),
            level: LOG_LEVEL,
        };

        *resources.LOGGER = Some(logger);
        let log: &'static mut _ = resources.LOGGER.as_mut().unwrap();
        unsafe { trick_init(&log).unwrap(); }

        info!("\r\n\r\n  /\\/\\/ / /\\ \\ \\__ _| |_ ___| |__  \r\n /    \\ \\/  \\/ / _` | __/ __| '_ \\ \r\n/ /\\/\\ \\  /\\  / (_| | || (__| | | |\r\n\\/    \\/\\/  \\/ \\__,_|\\__\\___|_| |_|\r\n                                   \r\n");
        info!("Copyright Scott Mabin 2019");
        info!("Clocks: {:#?}", clocks);
        let mut gpioa = device.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.ahb2);
        let mut channels = device.DMA1.split(&mut rcc.ahb1);

        let mut pwr = device.PWR.constrain(&mut rcc.apb1r1);
        let rtc = Rtc::rtc(device.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1, clocks);

        let date = Date::new(1.day(), 7.date(), 10.month(), 2018.year());
        rtc.set_date(&date);

        /* Ssd1351 Display */
        let mut delay = Delay::new(core.SYST, clocks);
        let mut rst = gpiob
            .pb0
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        let dc = gpiob
            .pb1
            .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

        let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

        let spi = Spi::spi1(
            device.SPI1,
            (sck, miso, mosi),
            SSD1351_SPI_MODE,
            8.mhz(),
            clocks,
            &mut rcc.apb2,
        );
        let fb: &'static mut [u8] = resources.FRAME_BUFFER;
        let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc, fb).into();
        display.reset(&mut rst, &mut delay);
        display.init().unwrap();
        display.set_rotation(DisplayRotation::Rotate0).unwrap();
        display.clear(true);

        let tx = gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
        let rx = gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl);

        let mut serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            115_200.bps(),
            clocks,
            &mut rcc.apb1r1,
        );
        serial.listen(SerialEvent::Idle); // Listen to Idle Line detection, IT not enable until after init is complete
        let (tx, rx) = serial.split();

        delay.delay_ms(100_u8); // allow module to boot
        let mut hm11 = Hm11::new(tx, rx); // tx, rx into hm11 for configuration
        hm11.send_with_delay(Command::Test, &mut delay).unwrap();
        hm11.send_with_delay(Command::Notify(false), &mut delay).unwrap();
        hm11.send_with_delay(Command::SetName("MWatch"), &mut delay)
            .unwrap();
        hm11.send_with_delay(Command::SystemLedMode(true), &mut delay)
            .unwrap();
        hm11.send_with_delay(Command::Reset, &mut delay).unwrap();
        delay.delay_ms(100_u8); // allow module to reset
        hm11.send_with_delay(Command::Test, &mut delay).unwrap(); // has the module come back up?
        let (_, rx) = hm11.release();

        channels.6.listen(Event::HalfTransfer);
        channels.6.listen(Event::TransferComplete);

        /* Touch sense controller */
        let sample_pin =
            gpiob
                .pb4
                .into_touch_sample(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let right_button =
            gpiob
                .pb5
                .into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let middle_button =
            gpiob
                .pb6
                .into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let left_button =
            gpiob
                .pb7
                .into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let tsc_config = TscConfig {
            clock_prescale: Some(TscClockPrescaler::HclkDiv16),
            max_count_error: None,
            charge_transfer_high: None,
            charge_transfer_low: None,
        };
        let tsc = Tsc::tsc(device.TSC, sample_pin, &mut rcc.ahb1, Some(tsc_config));

        /* T4056 input pins */
        let stdby = gpioa
            .pa11
            .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);
        let chrg = gpioa
            .pa12
            .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);
        let bt_conn = gpioa
            .pa8
            .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);

        /* Fuel Guage */
        let mut scl = gpioa
            .pa9
            .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
        scl.internal_pull_up(&mut gpioa.pupdr, true);
        let scl = scl.into_af4(&mut gpioa.moder, &mut gpioa.afrh);

        let mut sda = gpioa
            .pa10
            .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);
        sda.internal_pull_up(&mut gpioa.pupdr, true);
        let sda = sda.into_af4(&mut gpioa.moder, &mut gpioa.afrh);

        let i2c = I2c::i2c1(device.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1r1);

        let max17048 = Max17048::new(i2c);

        let bms = BatteryManagement::new(max17048, chrg, stdby);

        /* Static RB for Msg recieving */
        *resources.RB = Some(Queue::new());
        let rb: &'static mut Queue<u8, U512> = resources.RB.as_mut().unwrap();
        let buffers: &'static mut [Notification; crate::BUFF_COUNT] = resources.NOTIFICATIONS;

        // Give the RB to the ingress manager
        let imgr = IngressManager::new(rb);

        /* Pass messages to the Message Manager */
        let nmgr = NotificationManager::new(buffers);

        /* Give the application manager its ram */
        let ram: &'static mut [u8] = resources.APPLICATION_RAM;
        let amgr = ApplicationManager::new(ram);

        let mut systick = Timer::tim2(device.TIM2, 3.hz(), clocks, &mut rcc.apb1r1);
        systick.listen(TimerEvent::TimeOut);

        let mut cpu = Timer::tim7(
            device.TIM7,
            CPU_USAGE_POLL_FREQ.hz(),
            clocks,
            &mut rcc.apb1r1,
        );
        cpu.listen(TimerEvent::TimeOut);

        // input 'thread' poll the touch buttons - could we impl a proper hardare solution with the TSC?
        let mut input = Timer::tim6(device.TIM6, (3 * 3).hz(), clocks, &mut rcc.apb1r1); // hz * button count
        #[cfg(not(feature = "disable-input"))]
        {
            input.listen(TimerEvent::TimeOut);
        }

        let buffer: &'static mut [[u8; crate::DMA_HAL_SIZE]; 2] = resources.DMA_BUFFER;

        let input_mgr = InputManager::new(tsc, left_button, middle_button, right_button);

        let wmng = WindowManager::default();

        let system = System::new(rtc, bms, nmgr, amgr);

        USART2_RX = rx;
        CB = rx.circ_read(channels.6, buffer);
        IMNG = imgr;
        DISPLAY = display;
        SYSTEM = system;
        BT_CONN = bt_conn;
        SYS_TICK = systick;
        TIM7 = cpu;
        TIM6 = input;
        INPUT_MGR = input_mgr;
        WMNG = wmng;
    }

    #[cfg(not(test))]
    #[idle(resources = [SLEEP_TIME])]
    fn idle() -> ! {
        loop {
            resources.SLEEP_TIME.lock(|sleep| {
                let before = DWT::get_cycle_count();
                asm::wfi();
                let after = DWT::get_cycle_count();
                *sleep += after.wrapping_sub(before);
            });

            // interrupts are serviced here
        }
    }

    #[cfg(not(test))]
    #[task(resources = [SYSTEM, DISPLAY, WMNG])]
    fn HANDLE_INPUT(input: InputEvent) {
        let mut display = resources.DISPLAY;
        resources.WMNG.service_input(&mut resources.SYSTEM, &mut display,  input);
    }

    /// Handles a full or hal full dma buffer of serial data,
    /// and writes it into the MessageManager rb
    #[cfg(not(test))]
    #[interrupt(resources = [CB, IMNG], priority = 2)]
    fn DMA1_CH6() {
        let mut mgr = resources.IMNG;
        resources
            .CB
            .peek(|buf, _half| {
                mgr.write(buf);
            })
            .unwrap();
    }

    #[cfg(not(test))]
    #[interrupt(resources = [TSC_EVENTS, INPUT_MGR], priority = 2, spawn = [HANDLE_INPUT])]
    fn TSC() {
        *resources.TSC_EVENTS += 1;
        let mut input_mgr = resources.INPUT_MGR;
        match input_mgr.process_result() {
            Ok(_) => {
                match input_mgr.output() {
                    Ok(input) => {
                        info!("Output => {:?}", input);
                        match spawn.HANDLE_INPUT(input) {
                            Ok(_) => {},
                            Err(e) => panic!("Failed to spawn input task. Input {:?}", e)
                        }
                    },
                    Err(e) => {
                        if e != system::input::Error::NoInput {
                            error!("Input Error, {:?}", e);
                        }
                    }
                }
            },
            Err(e) => {
                if e != system::input::Error::Incomplete {
                    panic!("process_result error: {:?}", e)
                }
            }
        }

        
    }

    /// Handles the intermediate state where the DMA has data in it but
    /// not enough to trigger a half or full dma complete
    #[cfg(not(test))]
    #[interrupt(resources = [CB, IMNG, USART2_RX], priority = 2)]
    fn USART2() {
        let mut mgr = resources.IMNG;
        if resources.USART2_RX.is_idle(true) {
            resources
                .CB
                .partial_peek(|buf, _half| {
                    let len = buf.len();
                    if len > 0 {
                        mgr.write(buf);
                    }
                    Ok((len, ()))
                })
                .unwrap();
        }
    }

    #[cfg(not(test))]
    #[interrupt(resources = [INPUT_MGR, TIM6], priority = 2)] // TIM6
    fn TIM6_DACUNDER() {
        match resources.INPUT_MGR.start_new() {
            Ok(_) => {},
            Err(e) => {
                if e != system::input::Error::AcquisitionInProgress {
                    panic!("{:?}", e);
                }
            }
        }
        resources.TIM6.wait().unwrap(); // this should never panic as if we are in the IT the uif bit is set
    }

    #[cfg(not(test))]
    #[interrupt(resources = [TIM7, SLEEP_TIME, TSC_EVENTS, SYSTEM])]
    fn TIM7() {
        // CPU_USE = ((TOTAL - SLEEP_TIME) / TOTAL) * 100.
        let mut system = resources.SYSTEM;
        let total = SYS_CLK / CPU_USAGE_POLL_FREQ;
        let cpu = ((total - *resources.SLEEP_TIME) as f32 / total as f32) * 100.0;
        trace!("CPU_USAGE: {}%", cpu);
        *resources.SLEEP_TIME = 0;
        system.ss().tsc_events = resources.TSC_EVENTS.lock(|val| {
            let value = *val;
            *val = 0; // reset the value
            value
        });
        system.ss().cpu_usage = cpu;
        resources.TIM7.wait().unwrap(); // this should never panic as if we are in the IT the uif bit is set
    }

    #[cfg(not(test))]
    #[interrupt(resources = [IMNG, SYSTEM, SYS_TICK], spawn = [WM])]
    fn TIM2() {
        let mut system = resources.SYSTEM;
        let mut mgr = resources.IMNG;
        system.bms().process();
        mgr.lock(|m| {
            m.process(&mut system);
        });
        spawn.WM().unwrap();
        resources.SYS_TICK.wait().unwrap(); // this should never panic as if we are in the IT the uif bit is set
    }

    #[cfg(not(test))]
    #[task(resources = [DISPLAY, SYSTEM, BT_CONN, WMNG])]
    fn WM() {
        let mut display = resources.DISPLAY;
        let mut wmng = resources.WMNG;
        let cs = crc::crc16::checksum_x25(display.fb());
        trace!("WM - CS before: {}", cs);
        display.clear(false);
        wmng.process(&mut resources.SYSTEM, &mut display);
        let cs_after = crc::crc16::checksum_x25(display.fb());
        trace!("WM - CS after: {}", cs_after);
        if cs != cs_after {
            display.flush();
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn EXTI0();
        fn EXTI1();
    }
};

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
