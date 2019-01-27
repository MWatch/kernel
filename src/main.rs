#![no_std]
#![no_main]

#[macro_use]
extern crate cortex_m;
extern crate rtfm;
#[cfg(feature = "cpu-itm")]
extern crate panic_itm;
#[cfg(not(feature = "cpu-itm"))]
extern crate panic_semihosting;
extern crate cortex_m_rt as rt;
extern crate embedded_graphics;
extern crate heapless;
extern crate hm11;
extern crate max17048;
extern crate ssd1351;

mod application;
mod ingress;


use mwatch_kernel_api::{hal, BatteryManagementIC, LeftButton, MiddleButton, RightButton, Ssd1351};
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
    ClockPrescaler as TscClockPrescaler, Config as TscConfig, Event as TscEvent, Tsc,
};
use crate::rt::exception;
use crate::rt::ExceptionFrame;
use core::fmt::Write;
use embedded_graphics::Drawing;
use heapless::consts::*;
use heapless::spsc::Queue;
use heapless::String;
use rtfm::app;

use ssd1351::builder::Builder;
use ssd1351::mode::GraphicsMode;
use ssd1351::prelude::*;
use ssd1351::properties::DisplayRotation;

use embedded_graphics::fonts::Font12x16;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;

use cortex_m::asm;
use cortex_m::peripheral::DWT;
use hm11::command::Command;
use hm11::Hm11;
use max17048::Max17048;

use crate::ingress::ingress_manager::IngressManager;
use crate::ingress::ingress_manager::BUFF_COUNT;
use crate::ingress::notification::Notification;
use crate::ingress::notification::NotificationManager;

use crate::application::application_manager::ApplicationManager;


const DMA_HAL_SIZE: usize = 64;
const SYS_CLK: u32 = 32_000_000;
const CPU_USAGE_POLL_FREQ: u32 = 1; // hz

#[app(device = crate::hal::stm32)]
const APP: () = {
    static mut CB: CircBuffer<&'static mut [[u8; DMA_HAL_SIZE]; 2], dma1::C6> = ();
    static mut IMNG: IngressManager = ();
    static mut NMGR: NotificationManager = ();
    static mut AMGR: ApplicationManager = ();
    static mut NOTIFICATIONS: [Notification; crate::BUFF_COUNT] =
        [Notification::default(); crate::BUFF_COUNT];
    static mut RB: Option<Queue<u8, heapless::consts::U256>> = None;
    static mut USART2_RX: hal::serial::Rx<hal::stm32l4::stm32l4x2::USART2> = ();
    static mut DISPLAY: Ssd1351 = ();
    static mut RTC: hal::rtc::Rtc = ();
    static mut TOUCH: hal::tsc::Tsc<hal::gpio::gpiob::PB4<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::OpenDrain>>>> = ();
    static mut RIGHT_BUTTON: RightButton = ();
    static mut MIDDLE_BUTTON: MiddleButton = ();
    static mut LEFT_BUTTON: LeftButton = ();
    static mut CHRG: hal::gpio::gpioa::PA12<hal::gpio::Input<hal::gpio::PullUp>> = ();
    static mut STDBY: hal::gpio::gpioa::PA11<hal::gpio::Input<hal::gpio::PullUp>> = ();
    static mut BT_CONN: hal::gpio::gpioa::PA8<hal::gpio::Input<hal::gpio::Floating>> = ();
    static mut BMS: BatteryManagementIC = ();
    static mut TOUCH_THRESHOLD: u16 = ();
    static mut DMA_BUFFER: [[u8; crate::DMA_HAL_SIZE]; 2] = [[0; crate::DMA_HAL_SIZE]; 2];
    static mut WAS_TOUCHED: bool = false;
    static mut STATE: u8 = 0;
    static mut ITM: cortex_m::peripheral::ITM = ();
    static mut SYS_TICK: hal::timer::Timer<hal::stm32::TIM2> = ();
    static mut TIM6: hal::timer::Timer<hal::stm32::TIM6> = ();

    static mut SLEEP_TIME: u32 = 0;
    static mut CPU_USAGE: f32 = 0.0;
    static mut TIM7: hal::timer::Timer<hal::stm32::TIM7> = ();
    static mut INPUT_IT_COUNT: u32 = 0;
    static mut INPUT_IT_COUNT_PER_SECOND: u32 = 0;

    #[link_section = ".fb_section.fb"]
    static mut FRAME_BUFFER: [u8; 32 * 1024] = [0u8; 32 * 1024];
    #[link_section = ".app_section.data"]
    static mut APPLICATION_RAM: [u8; 16 * 1024] = [0u8; 16 * 1024];
    // static mut APPLICATION_RAM: Buffer = Buffer { payload: [0u8; RAM_SIZE], ..Buffer::default() }; // cant use buffer as the payload has to be at address

    #[init(resources = [RB, NOTIFICATIONS, DMA_BUFFER, APPLICATION_RAM, FRAME_BUFFER])]
    fn init() {
        core.DCB.enable_trace(); // required for DWT cycle clounter to work when not connected to the debugger
        core.DWT.enable_cycle_counter();
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .sysclk(16.mhz())
            .pclk1(16.mhz())
            .pclk2(16.mhz())
            .freeze(&mut flash.acr);
        // let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let mut gpioa = device.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = device.GPIOB.split(&mut rcc.ahb2);
        let mut channels = device.DMA1.split(&mut rcc.ahb1);

        let mut pwr = device.PWR.constrain(&mut rcc.apb1r1);
        let rtc = Rtc::rtc(device.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1);

        let date = Date::new(1.day(), 07.date(), 10.month(), 2018.year());
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

        /* Serial with DMA */
        // usart 1
        // let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
        // let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
        // let mut serial = Serial::usart1(device.USART1, (tx, rx), 9_600.bps(), clocks, &mut rcc.apb2);

        let tx = gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
        let rx = gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl);

        let mut serial = Serial::usart2(
            device.USART2,
            (tx, rx),
            115200.bps(),
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
        let mut middle_button =
            gpiob
                .pb6
                .into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let left_button =
            gpiob
                .pb7
                .into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let tsc_config = TscConfig {
            clock_prescale: Some(TscClockPrescaler::HclkDiv32),
            max_count_error: None,
        };
        let mut tsc = Tsc::tsc(device.TSC, sample_pin, &mut rcc.ahb1, Some(tsc_config));

        // Acquire for rough estimate of capacitance
        const NUM_SAMPLES: u16 = 25;
        let mut baseline = 0;
        for _ in 0..NUM_SAMPLES {
            baseline += tsc.acquire(&mut middle_button).unwrap();
        }
        let threshold = ((baseline / NUM_SAMPLES) / 100) * 90;

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

        /* Static RB for Msg recieving */
        *resources.RB = Some(Queue::new());
        let rb: &'static mut Queue<u8, U256> = resources.RB.as_mut().unwrap();
        let buffers: &'static mut [Notification; crate::BUFF_COUNT] = resources.NOTIFICATIONS;

        // Give the RB to the ingress manager
        let imgr = IngressManager::new(rb);

        /* Pass messages to the Message Manager */
        let nmgr = NotificationManager::new(buffers);

        /* Give the application manager its ram */
        let ram: &'static mut [u8] = resources.APPLICATION_RAM;
        let amgr = ApplicationManager::new(ram);

        let mut systick = Timer::tim2(device.TIM2, 4.hz(), clocks, &mut rcc.apb1r1);
        systick.listen(TimerEvent::TimeOut);

        let mut cpu = Timer::tim7(
            device.TIM7,
            CPU_USAGE_POLL_FREQ.hz(),
            clocks,
            &mut rcc.apb1r1,
        );
        cpu.listen(TimerEvent::TimeOut);

        // input 'thread' poll the touch buttons - could we impl a proper hardare solution with the TSC?
        let mut input = Timer::tim6(device.TIM6, (8 * 1).hz(), clocks, &mut rcc.apb1r1); // hz * button count
        input.listen(TimerEvent::TimeOut);

        tsc.listen(TscEvent::EndOfAcquisition);
        // tsc.listen(TscEvent::MaxCountError); // TODO
        // we do this to kick off the tsc loop - the interrupt starts a reading everytime one completes
        // rtfm::pend(stm32l4x2::Interrupt::TSC);
        let buffer: &'static mut [[u8; crate::DMA_HAL_SIZE]; 2] = resources.DMA_BUFFER;

        USART2_RX = rx;
        CB = rx.circ_read(channels.6, buffer);
        IMNG = imgr;
        DISPLAY = display;
        RTC = rtc;
        TOUCH = tsc;
        RIGHT_BUTTON = right_button;
        MIDDLE_BUTTON = middle_button;
        LEFT_BUTTON = left_button;
        TOUCH_THRESHOLD = threshold;
        BMS = max17048;
        STDBY = stdby;
        CHRG = chrg;
        BT_CONN = bt_conn;
        ITM = core.ITM;
        SYS_TICK = systick;
        TIM7 = cpu;
        TIM6 = input;
        NMGR = nmgr;
        AMGR = amgr;
    }

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

    #[task(resources = [AMGR, DISPLAY])]
    fn APP() {
        let mut amgr = resources.AMGR;
        let mut display = resources.DISPLAY;
        display.clear(false);
        amgr.service(&mut display).unwrap();
        display.flush();
    }

    /// Handles a full or hal full dma buffer of serial data,
    /// and writes it into the MessageManager rb
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

    #[interrupt(resources = [MIDDLE_BUTTON, TOUCH, TOUCH_THRESHOLD, INPUT_IT_COUNT, WAS_TOUCHED, STATE], priority = 2)]
    fn TSC() {
        *resources.INPUT_IT_COUNT += 1;
        let reading = resources.TOUCH.read(&mut *resources.MIDDLE_BUTTON).unwrap();
        let threshold = *resources.TOUCH_THRESHOLD;
        let current_touched = reading < threshold;

        if current_touched != *resources.WAS_TOUCHED {
            *resources.WAS_TOUCHED = current_touched;
            if current_touched == true {
                *resources.STATE += 1;
                if *resources.STATE > 4 {
                    *resources.STATE = 0;
                }
            }
        }
        resources.TOUCH.clear(TscEvent::EndOfAcquisition);
    }

    /// Handles the intermediate state where the DMA has data in it but
    /// not enough to trigger a half or full dma complete
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

    #[interrupt(resources = [TIM6, MIDDLE_BUTTON, TOUCH], priority = 2)]
    fn TIM6_DACUNDER() {
        resources.TOUCH.start(&mut *resources.MIDDLE_BUTTON);
        resources.TIM6.wait().unwrap(); // this should never panic as if we are in the IT the uif bit is set
    }

    #[interrupt(resources = [ITM, TIM7, SLEEP_TIME, CPU_USAGE, INPUT_IT_COUNT, INPUT_IT_COUNT_PER_SECOND])]
    fn TIM7() {
        // CPU_USE = ((TOTAL - SLEEP_TIME) / TOTAL) * 100.
        let total = SYS_CLK / CPU_USAGE_POLL_FREQ;
        let cpu = ((total - *resources.SLEEP_TIME) as f32 / total as f32) * 100.0;
        #[cfg(feature = "cpu-itm")]
        iprintln!(&mut resources.ITM.stim[0], "CPU_USAGE: {}%", cpu);
        *resources.SLEEP_TIME = 0;
        *resources.CPU_USAGE = cpu;
        let it_count = resources.INPUT_IT_COUNT.lock(|val| {
            let value = *val;
            *val = 0; // reset the value
            value
        });
        *resources.INPUT_IT_COUNT_PER_SECOND = it_count;
        resources.TIM7.wait().unwrap(); // this should never panic as if we are in the IT the uif bit is set
    }

    #[interrupt(resources = [IMNG, NMGR, AMGR, ITM, SYS_TICK], spawn = [APP, WM])]
    fn TIM2() {
        let mut mgr = resources.IMNG;
        let mut n_mgr = resources.NMGR;
        let mut a_mgr = resources.AMGR;
        mgr.lock(|m| {
            m.process(&mut n_mgr, &mut a_mgr);
        });

        let status = a_mgr.status();
        if status.is_running {
            spawn.APP().unwrap();
            // a_mgr.service(&mut display).unwrap();
        } else { // else run the WM
            spawn.WM().unwrap();
        }
        resources.SYS_TICK.wait().unwrap(); // this should never panic as if we are in the IT the uif bit is set
    }

    #[task(resources = [NMGR, DISPLAY, RTC, STATE, BMS, STDBY, CHRG, BT_CONN, CPU_USAGE, INPUT_IT_COUNT_PER_SECOND])]
    fn WM() {
        let mut display = resources.DISPLAY;
        let mut buffer: String<U256> = String::new();
        let state = resources.STATE.lock(|val| *val);
        let time = resources.RTC.get_time();
        let _date = resources.RTC.get_date();
        let mut n_mgr = resources.NMGR;

        display.clear(false);
        match state {
            // HOME PAGE
            0 => {
                write!(
                    buffer,
                    "{:02}:{:02}:{:02}",
                    time.hours, time.minutes, time.seconds
                )
                .unwrap();
                display.draw(
                    Font12x16::render_str(buffer.as_str())
                        .translate(Coord::new(10, 40))
                        .with_stroke(Some(0x2679_u16.into()))
                        .into_iter(),
                );
                buffer.clear(); // reset the buffer
                                // write!(buffer, "{:02}:{:02}:{:04}", date.date, date.month, date.year).unwrap();
                write!(buffer, "BT={}", resources.BT_CONN.is_high()).unwrap();
                display.draw(
                    Font6x12::render_str(buffer.as_str())
                        .translate(Coord::new(24, 60))
                        .with_stroke(Some(0x2679_u16.into()))
                        .into_iter(),
                );
                buffer.clear(); // reset the buffer
                write!(buffer, "{:02}", n_mgr.idx()).unwrap();
                display.draw(
                    Font12x16::render_str(buffer.as_str())
                        .translate(Coord::new(46, 96))
                        .with_stroke(Some(0x2679_u16.into()))
                        .into_iter(),
                );
                buffer.clear(); // reset the buffer
                let soc = resources.BMS.soc().unwrap(); /*  bodged_soc(); */
                write!(buffer, "{:02}%", soc).unwrap();
                display.draw(
                    Font6x12::render_str(buffer.as_str())
                        .translate(Coord::new(110, 12))
                        .with_stroke(Some(0x2679_u16.into()))
                        .into_iter(),
                );
                buffer.clear(); // reset the buffer
                write!(buffer, "{:03.03}v", resources.BMS.vcell().unwrap()).unwrap();
                display.draw(
                    Font6x12::render_str(buffer.as_str())
                        .translate(Coord::new(0, 12))
                        .with_stroke(Some(0x2679_u16.into()))
                        .into_iter(),
                );
                buffer.clear(); // reset the buffer
                if resources.CHRG.is_low() {
                    write!(buffer, "CHRG").unwrap();
                    display.draw(
                        Font6x12::render_str(buffer.as_str())
                            .translate(Coord::new(48, 12))
                            .with_stroke(Some(0x2679_u16.into()))
                            .into_iter(),
                    );
                    buffer.clear(); // reset the buffer
                    if let Some(soc_per_hr) = resources.BMS.charge_rate().ok() {
                        if soc_per_hr < 200.0 {
                            write!(buffer, "{:03.1}%/hr", soc_per_hr).unwrap();
                            display.draw(
                                Font6x12::render_str(buffer.as_str())
                                    .translate(Coord::new(32, 116))
                                    .with_stroke(Some(0x2679_u16.into()))
                                    .into_iter(),
                            );
                            buffer.clear(); // reset the buffer
                        }
                    }
                } else if resources.STDBY.is_high() {
                    write!(buffer, "STDBY").unwrap();
                    display.draw(
                        Font6x12::render_str(buffer.as_str())
                            .translate(Coord::new(48, 12))
                            .with_stroke(Some(0x2679_u16.into()))
                            .into_iter(),
                    );
                    buffer.clear(); // reset the buffer
                } else {
                    write!(buffer, "DONE").unwrap();
                    display.draw(
                        Font6x12::render_str(buffer.as_str())
                            .translate(Coord::new(48, 12))
                            .with_stroke(Some(0x2679_u16.into()))
                            .into_iter(),
                    );
                    buffer.clear(); // reset the buffer
                }
            }
            // MESSAGE LIST
            1 => {
                if n_mgr.idx() > 0 {
                    for i in 0..n_mgr.idx() {
                        n_mgr.peek_notification(i, |msg| {
                            write!(buffer, "[{}]: ", i + 1).unwrap();
                            for byte in msg.buffer() {
                                buffer.push(*byte as char).unwrap();
                            }
                            display.draw(
                                Font6x12::render_str(buffer.as_str())
                                    .translate(Coord::new(0, (i * 12) as i32 + 2))
                                    .with_stroke(Some(0xF818_u16.into()))
                                    .into_iter(),
                            );
                            buffer.clear();
                        });
                    }
                } else {
                    display.draw(
                        Font6x12::render_str("No messages.")
                            .translate(Coord::new(0, 12))
                            .with_stroke(Some(0xF818_u16.into()))
                            .into_iter(),
                    );
                }
            }
            // MWATCH LOGO
            2 => {
                display.draw(
                    Image16BPP::new(include_bytes!("../data/mwatch.raw"), 64, 64)
                        .translate(Coord::new(32, 32))
                        .into_iter(),
                );
            }
            // UOP LOGO
            3 => {
                display.draw(
                    Image16BPP::new(include_bytes!("../data/uop.raw"), 48, 64)
                        .translate(Coord::new(32, 32))
                        .into_iter(),
                );
            }
            //  Sys info
            4 => {
                write!(buffer, "CPU_USAGE: {:.02}%", *resources.CPU_USAGE).unwrap();
                display.draw(
                    Font6x12::render_str(buffer.as_str())
                        .translate(Coord::new(0, 12))
                        .with_stroke(Some(0xF818_u16.into()))
                        .into_iter(),
                );
                buffer.clear();
                let stack_space = get_free_stack();
                write!(buffer, "RAM: {} bytes", stack_space).unwrap();
                display.draw(
                    Font6x12::render_str(buffer.as_str())
                        .translate(Coord::new(0, 24))
                        .with_stroke(Some(0xF818_u16.into()))
                        .into_iter(),
                );
                buffer.clear();
                write!(buffer, "TSC IT: {}/s", *resources.INPUT_IT_COUNT_PER_SECOND).unwrap();
                display.draw(
                    Font6x12::render_str(buffer.as_str())
                        .translate(Coord::new(0, 36))
                        .with_stroke(Some(0xF818_u16.into()))
                        .into_iter(),
                );
                buffer.clear();
            }
            _ => panic!("Unknown state"),
        }
        display.flush();
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

fn _bodged_soc(raw: u16) -> u16 {
    let rawf = raw as f32;
    // let min = 0.0;
    let max = 80.0;
    let mut soc = ((rawf / max) * 100.0) as u16;
    if soc > 100 {
        soc = 100; // cap at 100
    }
    soc
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
