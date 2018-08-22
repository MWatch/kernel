#![feature(extern_prelude)]
#![feature(proc_macro_gen)]

#![deny(unsafe_code)]
#![deny(warnings)]

#![no_std]
#![no_main]

#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;
extern crate heapless;
extern crate ssd1351;
extern crate embedded_graphics;
extern crate stm32l432xx_hal as hal;

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;

use rt::ExceptionFrame;
use hal::dma::{dma1, CircBuffer, Event};
use hal::prelude::*;
use hal::serial::{Serial, Event as SerialEvent};
use hal::timer::{Timer, Event as TimerEvent};
use hal::delay::Delay;
use hal::spi::Spi;
use hal::rtc::Rtc;
use hal::tsc::{Tsc, /* Event as TscEvent */};
use hal::stm32l4::stm32l4x2;
use heapless::RingBuffer;
use heapless::String;
use heapless::consts::*;
use rtfm::{app, Threshold};

use core::fmt::Write;

use ssd1351::builder::Builder;
use ssd1351::mode::{GraphicsMode};
use ssd1351::prelude::*;

use embedded_graphics::prelude::*;
use embedded_graphics::fonts::Font12x16;
use embedded_graphics::fonts::Font6x12;

/* Our includes */
mod msgmgr;

use msgmgr::Message;
use msgmgr::MessageManager;

/// Type Alias to use in resource definitions
type Ssd1351 = ssd1351::mode::GraphicsMode<ssd1351::interface::SpiInterface<hal::spi::Spi<hal::stm32l4::stm32l4x2::SPI1, (hal::gpio::gpioa::PA5<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>, hal::gpio::gpioa::PA6<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>, hal::gpio::gpioa::PA7<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>)>, hal::gpio::gpiob::PB1<hal::gpio::Output<hal::gpio::PushPull>>>>;
entry!(main);

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

// TODO this catches systick, RTFM needs to strong link against it
exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

app! {
    device: stm32l4x2,

    resources: {
        // static STDOUT: sh::hio::HStdout;
        static BUFFER: [[u8; 64]; 2] = [[0; 64]; 2];
        static CB: CircBuffer<[u8; 64], dma1::C5>;
        static MSG_PAYLOADS: [[u8; 256]; 8] = [[0; 256]; 8];
        static MMGR: MessageManager;
        static RB: heapless::RingBuffer<u8, heapless::consts::U128> = heapless::RingBuffer::new();
        static USART1_RX: hal::serial::Rx<hal::stm32l4::stm32l4x2::USART1>;
        static DISPLAY: Ssd1351;
        static RTC: hal::rtc::Rtc;
        static TOUCH: hal::tsc::Tsc<hal::gpio::gpiob::PB4<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::OpenDrain>>>>;
        static OK_BUTTON: hal::gpio::gpiob::PB5<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::PushPull>>>;
        static STATUS_LED: hal::gpio::gpiob::PB3<hal::gpio::Output<hal::gpio::PushPull>>;
        static TOUCH_THRESHOLD: u16;
    },

    init: {
        resources: [BUFFER, MSG_PAYLOADS, RB],
    },

    tasks: {
        DMA1_CH5: { /* DMA channel for Usart1 RX */
            path: rx,
            resources: [CB, MMGR],
        },
        USART1: { /* Global usart1 it, uses for idle line detection */
            path: rx_idle,
            resources: [CB, MMGR, USART1_RX],
        },
        TIM2: {
            path: sys_tick,
            resources: [MMGR, DISPLAY, RTC, OK_BUTTON, TOUCH, TOUCH_THRESHOLD, STATUS_LED],
        },
        TSC: {
            path: touch,
            resources: [TOUCH]
        }
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {

    // let hstdout = hio::hstdout().unwrap();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();
    // let clocks = rcc.cfgr.sysclk(80.mhz()).pclk1(80.mhz()).pclk2(80.mhz()).freeze(&mut flash.acr);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    
    let mut gpioa = p.device.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = p.device.GPIOB.split(&mut rcc.ahb2);
    let mut channels = p.device.DMA1.split(&mut rcc.ahb1);
    

    let mut pwr = p.device.PWR.constrain(&mut rcc.apb1r1);
    let rtc = Rtc::rtc(p.device.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1);

    /* Ssd1351 Display */
    let mut delay = Delay::new(p.core.SYST, clocks);
    let mut rst = gpioa
        .pa8
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let dc = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let spi = Spi::spi1(
        p.device.SPI1,
        (sck, miso, mosi),
        SSD1351_SPI_MODE,
        2.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();
    display.reset(&mut rst, &mut delay);
    display.init().unwrap();

    /* Serial with DMA */
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    
    let mut serial = Serial::usart1(p.device.USART1, (tx, rx), 9_600.bps(), clocks, &mut rcc.apb2);
    serial.listen(SerialEvent::Idle); // Listen to Idle Line detection
    let (_, rx) = serial.split();

    channels.5.listen(Event::HalfTransfer);
    channels.5.listen(Event::TransferComplete);

    // Touch sense controller
    let sample_pin = gpiob.pb4.into_touch_sample(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut ok_button = gpiob.pb5.into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let tsc = Tsc::tsc(p.device.TSC, sample_pin, &mut rcc.ahb1);
    let baseline = tsc.acquire(&mut ok_button).unwrap();
    let threshold = (baseline / 100) * 60;
    // tsc.listen(TscEvent::EndOfAcquisition); // enable interrupts

    //status LED
    let led = gpiob.pb3.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    
    let rb: &'static mut RingBuffer<u8, U128> = r.RB; /* Static RB for Msg recieving */
    
    /* Define out block of message - surely there must be a nice way to to this? */
    let msgs: [msgmgr::Message; 8] = [
        Message::new(r.MSG_PAYLOADS[0]),
        Message::new(r.MSG_PAYLOADS[1]),
        Message::new(r.MSG_PAYLOADS[2]),
        Message::new(r.MSG_PAYLOADS[3]),
        Message::new(r.MSG_PAYLOADS[4]),
        Message::new(r.MSG_PAYLOADS[5]),
        Message::new(r.MSG_PAYLOADS[6]),
        Message::new(r.MSG_PAYLOADS[7]),
    ];


    /* Pass messages to the Message Manager */
    let mmgr = MessageManager::new(msgs, rb);

    let mut systick = Timer::tim2(p.device.TIM2, 2.hz(), clocks, &mut rcc.apb1r1);
    systick.listen(TimerEvent::TimeOut);

    // writeln!(hstdout, "Init Complete!");

    init::LateResources {
        CB: rx.circ_read(channels.5, r.BUFFER),
        MMGR: mmgr,
        USART1_RX: rx,
        DISPLAY: display,
        RTC: rtc,
        TOUCH: tsc,
        OK_BUTTON: ok_button,
        STATUS_LED: led,
        TOUCH_THRESHOLD: threshold
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi(); /* Wait for interrupts - sleep mode */
    }
}
/// Example Incoming payload
/// echo -ne '\x02N\x1FBodyHere!\x03' > /dev/ttyUSB0
fn rx(_t: &mut Threshold, mut r: DMA1_CH5::Resources) {
    let mut mgr = r.MMGR;
    r.CB
        .peek(|buf, _half| {
            mgr.write(buf);
        })
        .unwrap();
}

fn rx_idle(_t: &mut Threshold, mut r: USART1::Resources) {
    if r.USART1_RX.is_idle(true) {

        let mut mgr = r.MMGR;
        r.CB
            .partial_peek(|buf, _half| {
                let len = buf.len();
                if len > 0 {
                    mgr.write(buf);
                }
                
                Ok( (len, ()) )
            })
            .unwrap();

    }
}

fn sys_tick(_t: &mut Threshold, mut r: TIM2::Resources) {
    let mut mgr = r.MMGR;
    mgr.process();
    let msg_count = mgr.msg_count();
    // writeln!(out, "MSGS[{}] ", msg_count);
    // for i in 0..msg_count {
    //     mgr.peek_message(i, |msg| {
    //         let payload: &[u8] = &msg.payload;
    //         let len = msg.payload_idx;
    //         if len > 0 {
    //             writeln!(out, "MSG[{}] ", i);
    //             // for byte in payload {
    //             //     iprint!(out, "{}", *byte as char);
    //             // }
    //             // iprintln!(out, "");
    //         }
    //         // Payload is in the variable payload
    //     });
    // }

    let mut buffer: String<U16> = String::new();
    let time = r.RTC.get_time();
    let date = r.RTC.get_date();
    {
        write!(buffer, "{:02}:{:02}:{:02}", time.hours, time.minutes, time.seconds).unwrap();
        r.DISPLAY.draw(Font12x16::render_str(buffer.as_str(), 0xF818_u16.into()).translate(Coord::new(10, 40)).into_iter());
        buffer.clear(); // reset the buffer
        write!(buffer, "{:02}:{:02}:{:04}", date.date, date.month, date.year).unwrap();
        r.DISPLAY.draw(Font6x12::render_str(buffer.as_str(), 0x880B_u16.into()).translate(Coord::new(24, 60)).into_iter());
        buffer.clear(); // reset the buffer
        write!(buffer, "{:02}", msg_count).unwrap();
        r.DISPLAY.draw(Font12x16::render_str(buffer.as_str(), 0xF818_u16.into()).translate(Coord::new(46, 96)).into_iter());
        buffer.clear(); // reset the buffer
    }


    let reading = r.TOUCH.acquire(&mut *r.OK_BUTTON).unwrap();
    let threshold: u16 = *r.TOUCH_THRESHOLD;
    if reading < threshold {
        r.STATUS_LED.set_high();
    } else {
        r.STATUS_LED.set_low();
    }
}

fn touch(_t: &mut Threshold, mut _r: TSC::Resources) {
    // r.TOUCH;
}
