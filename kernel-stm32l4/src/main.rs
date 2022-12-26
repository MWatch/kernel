#![no_std]
#![no_main]

#[cfg(feature = "itm")]
extern crate panic_itm;
#[cfg(feature = "rtt")]
extern crate panic_rtt_target;
#[cfg(feature = "semihosting")]
extern crate panic_semihosting;
#[macro_use]
extern crate log;

mod bms;
mod system;
mod tsc;
mod types;

use crate::{
    bms::BatteryManagement,
    system::DisplayWrapper,
    tsc::TscManager,
    types::{hal, BluetoothConnectedPin, LoggerType},
};
use mwatch_kernel::{application, ingress, system::input::InputEvent};

use crate::hal::{
    datetime::Date,
    delay::Delay,
    dma::{dma1, CircBuffer, Event},
    i2c::I2c,
    prelude::*,
    rtc::Rtc,
    serial::{Config, Event as SerialEvent, Serial},
    spi::Spi,
    timer::{Event as TimerEvent, Timer},
    tsc::{Config as TscConfig, Tsc},
};

use ssd1351::{builder::Builder, mode::GraphicsMode, prelude::*, properties::DisplayRotation};

#[cfg(feature = "itm")]
use cortex_m_log::{
    destination::Itm as ItmDestination, log::Logger,
    printer::itm::InterruptSync as InterruptSyncItm,
};

use cortex_m::{asm, peripheral::DWT};
use cortex_m_rt::{exception, ExceptionFrame};
use hm11::{command::Command, Hm11};
use max17048::Max17048;
use rtic::app;

use crate::application::{
    application_manager::{ApplicationManager, Ram},
    display_manager::DisplayManager,
};
use crate::ingress::ingress_manager::IngressManager;

use crate::system::{
    System, CPU_USAGE_POLL_HZ, DMA_HALF_BYTES, I2C_KHZ, SPI_MHZ, SYSTICK_HZ, SYS_CLK_HZ, TSC_HZ,
};
use mwatch_kernel::system::{input::InputManager, notification::NotificationManager};

#[cfg(any(feature = "itm", feature = "rtt"))]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;
#[cfg(not(any(feature = "itm", feature = "rtt")))]
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Off;

#[app(device = crate::hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        CB: CircBuffer<&'static mut [[u8; DMA_HALF_BYTES]; 2], dma1::C6>,
        IMNG: IngressManager,
        INPUT_MGR: InputManager,
        TSC_MGR: TscManager,
        DMNG: DisplayManager,
        USART2_RX: hal::serial::Rx<hal::stm32l4::stm32l4x2::USART2>,
        DISPLAY: DisplayWrapper,
        BT_CONN: BluetoothConnectedPin,
        SYSTEM: System,
        SYSTICK: hal::timer::Timer<hal::stm32::TIM2>,
        TIM6: hal::timer::Timer<hal::stm32::TIM6>,
        TIM7_HANDLE: hal::timer::Timer<hal::stm32::TIM7>,
        #[init([[0u8; crate::DMA_HALF_BYTES]; 2])]
        DMA_BUFFER: [[u8; crate::DMA_HALF_BYTES]; 2],
        #[init(0)]
        SLEEP_TIME: u32,
        #[init(0)]
        TSC_EVENTS: u32,
        #[init(0)]
        IDLE_COUNT: u32,
        #[init(0)]
        LAST_BATT_PERCENT: u16,
        #[init(None)]
        LOGGER: Option<LoggerType>,
        #[init([0u8; 32 * 1024])]
        #[link_section = ".fb_section.fb"]
        FRAME_BUFFER: [u8; 32 * 1024],
        #[init([0u8; 16 * 1024])]
        #[link_section = ".app_section.data"]
        APPLICATION_RAM: [u8; 16 * 1024],
    }

    /// Intialization of the hardware and the kernel - mostly boiler plate init's from libraries
    #[init(resources = [DMA_BUFFER, APPLICATION_RAM, FRAME_BUFFER, LOGGER])]
    fn init(mut cx: init::Context) -> init::LateResources {
        cx.core.DCB.enable_trace(); // required for DWT cycle clounter to work when not connected to the debugger
        cx.core.DWT.enable_cycle_counter();
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        // let clocks = rcc.cfgr.lsi(true).freeze(&mut flash.acr);
        let clocks = rcc
            .cfgr
            .sysclk(80u32.mhz())
            .pclk1(80u32.mhz())
            .pclk2(80u32.mhz())
            .lsi(true)
            .freeze(&mut flash.acr);

        // initialize the logging framework
        #[cfg(feature = "itm")]
        let logger = Logger {
            inner: InterruptSyncItm::new(ItmDestination::new(cx.core.ITM)),
            level: LOG_LEVEL,
        };

        #[cfg(feature = "rtt")]
        let logger = rtt::RttLog::new(LOG_LEVEL);

        *cx.resources.LOGGER = Some(logger);

        let log: &'static mut _ = cx.resources.LOGGER.as_mut().unwrap_or_else(|| {
            panic!("Failed to get the static logger");
        });

        unsafe {
            log::set_logger_racy(log)
                .map(|()| log::set_max_level(LOG_LEVEL))
                .unwrap()
        };

        info!("\r\n\r\n  /\\/\\/ / /\\ \\ \\__ _| |_ ___| |__  \r\n /    \\ \\/  \\/ / _` | __/ __| '_ \\ \r\n/ /\\/\\ \\  /\\  / (_| | || (__| | | |\r\n\\/    \\/\\/  \\/ \\__,_|\\__\\___|_| |_|\r\n                                   \r\n");
        info!("Copyright Scott Mabin 2019");
        info!("Clocks: {:#?}", clocks);
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let mut channels = cx.device.DMA1.split(&mut rcc.ahb1);

        let mut pwr = cx.device.PWR.constrain(&mut rcc.apb1r1);
        let rtc = Rtc::rtc(
            cx.device.RTC,
            &mut rcc.apb1r1,
            &mut rcc.bdcr,
            &mut pwr.cr1,
            clocks,
        );

        let date = Date::new(1.day(), 7.date(), 10.month(), 2018.year());
        rtc.set_date(&date);

        /* Ssd1351 Display */
        let mut delay = Delay::new(cx.core.SYST, clocks);
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
            cx.device.SPI1,
            (sck, miso, mosi),
            SSD1351_SPI_MODE,
            SPI_MHZ.mhz(),
            clocks,
            &mut rcc.apb2,
        );
        let fb: &'static mut [u8] = cx.resources.FRAME_BUFFER;
        let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc, fb).into();
        display
            .reset(&mut rst, &mut delay)
            .expect("Failed to reset display");
        display.init().expect("Failed to initialize the display");
        display
            .set_rotation(DisplayRotation::Rotate0)
            .expect("Failed to set the display rotation");
        display.clear(true);
        let display = DisplayWrapper(display);

        let tx = gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
        let rx = gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl);

        let mut serial = Serial::usart2(
            cx.device.USART2,
            (tx, rx),
            /* Config::default().baudrate(115_200.bps()), */
            Config::default().baudrate(115_200.bps()),
            clocks,
            &mut rcc.apb1r1,
        );
        serial.listen(SerialEvent::Idle); // Listen to Idle Line detection, IT not enable until after init is complete
        let (tx, rx) = serial.split();

        delay.delay_ms(100_u8); // allow module to boot
        let mut hm11 = Hm11::new(tx, rx); // tx, rx into hm11 for configuration
        hm11.send_with_delay(Command::Test, &mut delay)
            .expect("HM11 - Not communicating, is the baud correct?");
        hm11.send_with_delay(Command::Notify(false), &mut delay)
            .expect("HM11 - Failed to turn off connection notification");
        hm11.send_with_delay(Command::SetName("MWatch"), &mut delay)
            .expect("Failed to set name to MWatch");
        hm11.send_with_delay(Command::SystemLedMode(true), &mut delay)
            .expect("HM11 - Failed to set GPIO mode");
        hm11.send_with_delay(Command::Reset, &mut delay)
            .expect("HM11 - Failed to reset module");
        delay.delay_ms(100_u8); // allow module to reset
        hm11.send_with_delay(Command::Test, &mut delay)
            .expect("HM11 - Module did not responde after reboot");
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
        #[cfg(feature = "dyn-tsc-cal")]
        let mut middle_button = middle_button;

        let left_button =
            gpiob
                .pb7
                .into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let tsc_config = TscConfig {
            clock_prescale: None, /* Some(TscClockPrescaler::HclkDiv2) */
            max_count_error: Some(hal::tsc::MaxCountError::U16383),
            charge_transfer_high: Some(hal::tsc::ChargeDischargeTime::C8),
            charge_transfer_low: Some(hal::tsc::ChargeDischargeTime::C8),
            spread_spectrum_deviation: None, /* Some(128u8) */
        };
        let tsc = Tsc::tsc(cx.device.TSC, sample_pin, &mut rcc.ahb1, Some(tsc_config));

        #[cfg(feature = "dyn-tsc-cal")]
        let tsc_threshold = {
            const TSC_SAMPLES: u16 = 12;
            // Acquire for rough estimate of capacitance
            let mut baseline = 0;
            for _ in 0..TSC_SAMPLES {
                baseline += tsc.acquire(&mut middle_button).unwrap_or_else(|err| {
                    panic!("Failed to calibrate tsc {:?}", err);
                });
                delay.delay_ms(20u8);
            }

            ((baseline / TSC_SAMPLES) / 100) * 98
        };

        #[cfg(not(feature = "dyn-tsc-cal"))]
        let tsc_threshold = {
            1060 // acquired through testing
        };

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

        let i2c = I2c::i2c1(
            cx.device.I2C1,
            (scl, sda),
            I2C_KHZ.khz(),
            clocks,
            &mut rcc.apb1r1,
        );
        let max17048 = Max17048::new(i2c);
        let bms = BatteryManagement::new(max17048, chrg, stdby);
        let imgr = IngressManager::new();
        let nmgr = NotificationManager::new();

        /* Give the application manager its ram */
        let ram: &'static mut [u8] = cx.resources.APPLICATION_RAM;
        static mut TBL: application::Table = application::Table {
            draw_pixel: system::abi::draw_pixel,
            print: system::abi::print,
        };
        let amgr = ApplicationManager::new(Ram::new(ram), unsafe { &mut TBL });

        let mut systick = Timer::tim2(cx.device.TIM2, SYSTICK_HZ.hz(), clocks, &mut rcc.apb1r1);
        systick.listen(TimerEvent::TimeOut);

        let mut cpu = Timer::tim7(
            cx.device.TIM7,
            CPU_USAGE_POLL_HZ.hz(),
            clocks,
            &mut rcc.apb1r1,
        );
        cpu.listen(TimerEvent::TimeOut);

        let mut input = Timer::tim6(cx.device.TIM6, TSC_HZ.hz(), clocks, &mut rcc.apb1r1);
        #[cfg(not(feature = "disable-input"))]
        {
            input.listen(TimerEvent::TimeOut);
        }

        let buffer: &'static mut [[u8; crate::DMA_HALF_BYTES]; 2] = cx.resources.DMA_BUFFER;
        let tsc_mgr = TscManager::new(tsc, tsc_threshold, left_button, middle_button, right_button);
        let input_mgr = InputManager::new();
        let dmng = DisplayManager::default();
        let mut system = System::new(rtc, bms, nmgr, amgr);
        system.ss().tsc_threshold = tsc_mgr.threshold();

        // To preload a path, add include the path here
        // let app = include_bytes!(/* PATH */);
        // let cs = crc::crc32::checksum_ieee(app);
        // let bytes = cs.to_be_bytes();
        // for b in bytes {
        //     system.am().write_checksum_byte(b).unwrap()
        // }
        // for &b in app {
        //     system.am().write_ram_byte(b).unwrap()
        // }
        // system.am().verify().unwrap();

        // Resources that need to be initialized are passed back here
        init::LateResources {
            CB: rx.circ_read(channels.6, buffer),
            USART2_RX: rx,
            IMNG: imgr,
            DISPLAY: display,
            SYSTEM: system,
            BT_CONN: bt_conn,
            SYSTICK: systick,
            TIM7_HANDLE: cpu,
            TIM6: input,
            INPUT_MGR: input_mgr,
            TSC_MGR: tsc_mgr,
            DMNG: dmng,
        }
    }

    /*
        Hardware threads
    */

    /// Idle thread - Captures the time the cpu is asleep to calculate cpu uasge
    #[idle(resources = [SLEEP_TIME])]
    fn idle(mut cx: idle::Context) -> ! {
        loop {
            cx.resources.SLEEP_TIME.lock(|sleep| {
                let before = DWT::get_cycle_count();
                asm::wfi(); /* CPU is idle here waiting for interrupt */
                let after = DWT::get_cycle_count();
                *sleep += after.wrapping_sub(before);
            });
            // interrupts are serviced here
        }
    }

    /// The main thread of the watch, this is called `SYSTICK_HZ` times a second, to perform
    /// housekeeping operations
    #[task(binds = TIM2, resources = [IMNG, SYSTEM, SYSTICK, IDLE_COUNT], spawn = [display_manager])]
    fn systemtick(cx: systemtick::Context) {
        let mut system = cx.resources.SYSTEM;
        let mut mgr = cx.resources.IMNG;
        let mut idle = cx.resources.IDLE_COUNT;

        cx.spawn.display_manager().unwrap_or_else(|_err| {
            error!("Failed to spawn display manager");
        });

        system.lock(|system| {
            system.bms().process();
            system.ss().idle_count = idle.lock(|val| {
                let value = *val;
                *val += 1; // append to idle count
                value
            });
            mgr.lock(|m| m.process(system));
        });
        cx.resources
            .SYSTICK
            .wait()
            .expect("systick timer was already cleared"); // this should never panic as if we are in the IT the uif bit is set
    }

    /// Hardware timer, initiates tsc aquisitions
    #[task(binds = TIM6_DACUNDER, resources = [TSC_MGR, TIM6, INPUT_MGR], priority = 3)] // TIM6
    fn tsc_initiator(cx: tsc_initiator::Context) {
        let pin = cx.resources.INPUT_MGR.current_pin();
        match cx.resources.TSC_MGR.start(pin as u8) {
            Ok(_) => {}
            Err(e) => {
                if e != crate::tsc::Error::AcquisitionInProgress {
                    panic!("{:?}", e);
                }
            }
        }
        // this should never panic as if we are in the IT the uif bit is set
        cx.resources.TIM6.wait().expect("TIM6 clear() failed");
    }

    /// Thread runs once a second and collates stats about the system
    #[task(binds = TIM7, resources = [TIM7_HANDLE, SLEEP_TIME, TSC_EVENTS, LAST_BATT_PERCENT, SYSTEM])]
    fn status(cx: status::Context) {
        // CPU_USE = ((TOTAL - SLEEP_TIME) / TOTAL) * 100.
        let mut systemr = cx.resources.SYSTEM;
        let mut tsc_ev = cx.resources.TSC_EVENTS;
        let total = SYS_CLK_HZ / CPU_USAGE_POLL_HZ;
        let cpu = ((total - *cx.resources.SLEEP_TIME) as f32 / total as f32) * 100.0;
        trace!("CPU_USAGE: {}%", cpu);
        *cx.resources.SLEEP_TIME = 0;

        let current_soc = systemr.lock(|system| {
            system.ss().tsc_events = tsc_ev.lock(|val| {
                let value = *val;
                *val = 0; // reset the value
                value
            });
            system.ss().cpu_usage = cpu;
            system.bms().soc()
        });

        let last_soc = *cx.resources.LAST_BATT_PERCENT;
        if current_soc != last_soc {
            info!(
                "SoC has {} to {}",
                if current_soc < last_soc {
                    "fallen"
                } else {
                    "risen"
                },
                current_soc
            );
            *cx.resources.LAST_BATT_PERCENT = current_soc;
        }
        cx.resources.TIM7_HANDLE.wait().expect("TIM7 wait() failed");
    }

    /*
        Hardware Interrupt service routines
    */

    /// When a TSC aquisition completes, the result is processed by the input manager
    /// If the result is a valid output, the input handler task is spawned to act upon it
    #[task(binds = TSC, resources = [TSC_EVENTS, INPUT_MGR, IDLE_COUNT, TSC_MGR], priority = 3, spawn = [input_handler])]
    fn tsc_result(cx: tsc_result::Context) {
        *cx.resources.TSC_EVENTS += 1;
        let input_mgr = cx.resources.INPUT_MGR;
        input_mgr.update_input(cx.resources.TSC_MGR.result(input_mgr.current_pin() as u8));

        match input_mgr.output() {
            Ok(input) => {
                *cx.resources.IDLE_COUNT = 0; // we are no longer idle
                info!("Output => {:?}", input);
                match cx.spawn.input_handler(input) {
                    Ok(_) => {}
                    Err(e) => panic!("Failed to spawn input task. Input {:?}", e),
                }
            }
            Err(e) => {
                if !matches!(
                    e,
                    mwatch_kernel::system::input::Error::NoInput
                        | mwatch_kernel::system::input::Error::Incomplete
                ) {
                    error!("Input Error, {:?}", e);
                }
            }
        }
    }

    /// Handles the intermediate state where the DMA has data in it but
    /// not enough to trigger a half or full dma complete
    #[task(binds = USART2, resources = [CB, IMNG, USART2_RX], priority = 3)]
    fn serial_partial_dma(cx: serial_partial_dma::Context) {
        let mgr = cx.resources.IMNG;
        // If the idle flag is set then we take what we have and push
        // it into the ingress manager
        if cx.resources.USART2_RX.is_idle(true) {
            cx.resources
                .CB
                .partial_peek(|buf, _half| {
                    let len = buf.len();
                    if len > 0 {
                        mgr.write(buf);
                    }
                    Ok((len, ()))
                })
                .unwrap_or_else(|err| {
                    error!("Failed to partial peek into circular buffer {:?}", err);
                });
        }
    }

    /// Handles a full or hal full dma buffer of serial data,
    /// and writes it into the MessageManager rb
    #[task(binds = DMA1_CH6, resources = [CB, IMNG], priority = 3)]
    fn serial_full_dma(cx: serial_full_dma::Context) {
        let mgr = cx.resources.IMNG;
        cx.resources
            .CB
            .peek(|buf, _half| {
                mgr.write(buf);
            })
            .unwrap_or_else(|err| {
                error!("Failed to full peek into circular buffer {:?}", err);
            });
    }

    /*
        Software tasks
    */

    /// Task that services the display manager
    #[task(resources = [DISPLAY, SYSTEM, BT_CONN, DMNG])]
    fn display_manager(cx: display_manager::Context) {
        let display = cx.resources.DISPLAY;
        let mut dmngr = cx.resources.DMNG;
        let mut sys = cx.resources.SYSTEM;
        // let mut system = cx.resources.SYSTEM;
        dmngr.lock(|dmng| {
            #[cfg(feature = "crc-fb")]
            {
                let is_idle = sys.lock(|system| system.is_idle());
                if is_idle {
                    let cs = crc::crc16::checksum_x25(display.fb());
                    trace!("DM - CS before: {}", cs);
                    display.clear(false);
                    sys.lock(|system| {
                        dmng.process(system, display);
                    });
                    let cs_after = crc::crc16::checksum_x25(display.fb());
                    trace!("DM - CS after: {}", cs_after);
                    if cs != cs_after {
                        display.flush();
                    }
                } else {
                    display.clear(false);
                    sys.lock(|system| {
                        dmng.process(system, display);
                    });
                    display.flush();
                }
            }
            #[cfg(not(feature = "crc-fb"))]
            {
                display.clear(false);
                sys.lock(|system| {
                    dmng.process(system, display);
                });
                display.flush();
            }
        });
    }

    /// This task is dispatched via the hardware TSC isr - allow up to 3 to be spawned at any time
    /// This task is very cheap, hence we can have 3 of them running at anytime
    #[task(resources = [SYSTEM, DMNG], priority = 2, capacity = 3)]
    fn input_handler(cx: input_handler::Context, input: InputEvent) {
        cx.resources.DMNG.service_input(cx.resources.SYSTEM, input);
    }

    /// Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn EXTI0();
        fn EXTI1();
    }
};

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[cfg(feature = "rtt")]
mod rtt {
    use rtt_target::{rprintln, rtt_init_print};

    pub struct RttLog {
        level: log::LevelFilter,
    }

    impl RttLog {
        pub fn new(level: log::LevelFilter) -> Self {
            rtt_init_print!();
            rprintln!("RttLog initialized!");
            Self { level }
        }
    }

    impl log::Log for RttLog {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= self.level
        }

        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                rprintln!(
                    "{:<5} {}:{} - {}\n",
                    record.level(),
                    record.file().unwrap_or("UNKNOWN"),
                    record.line().unwrap_or(0),
                    record.args()
                );
            }
        }

        fn flush(&self) {}
    }
}
