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
use hal::tsc::{Tsc, Event as TscEvent};
use hal::stm32l4::stm32l4x2;
use heapless::RingBuffer;
use heapless::String;
use heapless::consts::*;
use rtfm::{app, Threshold, Resource};

use core::fmt::Write;

use ssd1351::builder::Builder;
use ssd1351::mode::{GraphicsMode};
use ssd1351::prelude::*;

use embedded_graphics::Drawing;
use embedded_graphics::prelude::*;
use embedded_graphics::fonts::Font6x12;
use embedded_graphics::fonts::Font12x16;

/* Our includes */
mod msgmgr;
mod view;

use msgmgr::Message;
use msgmgr::MessageManager;

/// Type Alias to use in resource definitions
pub type Ssd1351 = ssd1351::mode::GraphicsMode<ssd1351::interface::SpiInterface<hal::spi::Spi<hal::stm32l4::stm32l4x2::SPI1, (hal::gpio::gpioa::PA5<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>, hal::gpio::gpioa::PA6<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>, hal::gpio::gpioa::PA7<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>)>, hal::gpio::gpiob::PB1<hal::gpio::Output<hal::gpio::PushPull>>>>;

#[entry]
fn main() -> ! {
    rtfm_main()
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

app! {
    device: stm32l4x2,

    resources: {
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
        static TOUCHED: bool = false;
        static WAS_TOUCHED: bool = false;
    },

    init: {
        resources: [BUFFER, MSG_PAYLOADS, RB],
    },

    tasks: {
        TIM2: {
            path: sys_tick,
            resources: [MMGR, DISPLAY, RTC, TOUCHED, WAS_TOUCHED],
        },
        DMA1_CH5: { /* DMA channel for Usart1 RX */
            priority: 2,
            path: rx_full,
            resources: [CB, MMGR],
        },
        USART1: { /* Global usart1 it, uses for idle line detection */
            priority: 2,
            path: rx_idle,
            resources: [CB, MMGR, USART1_RX],
        },
        TSC: {
            priority: 2, /* Input should always preempt other tasks */
            path: touch,
            resources: [OK_BUTTON, TOUCH, TOUCH_THRESHOLD, STATUS_LED, TOUCHED]
        }
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {

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
        4.mhz(), // TODO increase this when off the breadboard!
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

    /* Touch sense controller */
    let sample_pin = gpiob.pb4.into_touch_sample(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut ok_button = gpiob.pb5.into_touch_channel(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut tsc = Tsc::tsc(p.device.TSC, sample_pin, &mut rcc.ahb1);
    
    // Acquire for rough estimate of capacitance
    const NUM_SAMPLES: u16 = 10;
    let mut baseline = 0;
    for _ in 0..NUM_SAMPLES {
        baseline += tsc.acquire(&mut ok_button).unwrap();
    }
    let threshold = ((baseline / NUM_SAMPLES) / 100) * 75;

    /* status LED */
    let led = gpiob.pb3.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    
    /* Static RB for Msg recieving */
    let rb: &'static mut RingBuffer<u8, U128> = r.RB;
    
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

    let mut systick = Timer::tim2(p.device.TIM2, 4.hz(), clocks, &mut rcc.apb1r1);
    systick.listen(TimerEvent::TimeOut);

    // input 'thread' poll the touch buttons - could we impl a proper hardare solution with the TSC?
    // let mut input = Timer::tim7(p.device.TIM7, 20.hz(), clocks, &mut rcc.apb1r1);
    // input.listen(TimerEvent::TimeOut);

    tsc.listen(TscEvent::EndOfAcquisition);
    // tsc.listen(TscEvent::MaxCountError); // TODO
    // we do this to kick off the tsc loop - the interrupt starts a reading everytime one completes
    rtfm::set_pending(stm32l4x2::Interrupt::TSC);

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

/// Handles a full or hal full dma buffer of serial data,
/// and writes it into the MessageManager rb
fn rx_full(_t: &mut Threshold, mut r: DMA1_CH5::Resources) {
    let mut mgr = r.MMGR;
    r.CB
        .peek(|buf, _half| {
            mgr.write(buf);
        })
        .unwrap();
}

/// Handles the intermediate state where the DMA has data in it but
/// not enough to trigger a half or full dma complete
fn rx_idle(_t: &mut Threshold, mut r: USART1::Resources) {
    let mut mgr = r.MMGR;
    if r.USART1_RX.is_idle(true) {
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

fn sys_tick(t: &mut Threshold, mut r: TIM2::Resources) {
    let mut mgr = r.MMGR;
    let mut display = r.DISPLAY;
    let current_touched = r.TOUCHED.claim(t, | val, _| *val);
    let mut buffer: String<U256> = String::new();

    let msg_count = mgr.claim_mut(t, | m, _t| {
        m.process();
        m.msg_count()
    });
    
    let time = r.RTC.get_time();
    let date = r.RTC.get_date();

    
    // clears the screen 
    if current_touched != *r.WAS_TOUCHED {
        display.clear();
        *r.WAS_TOUCHED = current_touched;
    }

    if !current_touched {
        write!(buffer, "{:02}:{:02}:{:02}", time.hours, time.minutes, time.seconds).unwrap();
        display.draw(Font12x16::render_str(buffer.as_str())
            .translate(Coord::new(10, 40))
            .with_stroke(Some(0xF818_u16.into()))
            .into_iter());
        buffer.clear(); // reset the buffer
        write!(buffer, "{:02}:{:02}:{:04}", date.date, date.month, date.year).unwrap();
        display.draw(Font6x12::render_str(buffer.as_str())
            .translate(Coord::new(24, 60))
            .with_stroke(Some(0x880B_u16.into()))
            .into_iter());
        buffer.clear(); // reset the buffer
        write!(buffer, "{:02}", msg_count).unwrap();
        display.draw(Font12x16::render_str(buffer.as_str())
            .translate(Coord::new(46, 96))
            .with_stroke(Some(0xF818_u16.into()))
            .into_iter());
        buffer.clear(); // reset the buffer
    } else {
        mgr.claim_mut(t, |m, _t| {
            // for i in 0..msg_count {
                let i = 0;
                m.peek_message(i, |msg| {
                    write!(buffer, "[{}]: ", i + 1);
                    for c in 0..msg.payload_idx {
                        buffer.push(msg.payload[c] as char).unwrap();
                    }
                    display.draw(Font6x12::render_str(buffer.as_str())
                        .translate(Coord::new(0, (i * 12) as i32 + 2))
                        .with_stroke(Some(0xF818_u16.into()))
                        .into_iter());
                    buffer.clear();
                });
            // }
        });
    }
}


fn touch(_t: &mut Threshold, mut r: TSC::Resources) {
    // let reading = r.TOUCH.read_unchecked();
    let reading = r.TOUCH.read(&mut *r.OK_BUTTON).unwrap();
    if reading < *r.TOUCH_THRESHOLD {
        r.STATUS_LED.set_high();
        *r.TOUCHED = true;
    } else {
        r.STATUS_LED.set_low();
        *r.TOUCHED = false;
    }
    r.TOUCH.start(&mut *r.OK_BUTTON);
}
