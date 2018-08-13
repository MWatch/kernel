#![feature(use_extern_macros)]
#![feature(proc_macro_gen)]
#![feature(proc_macro_mod)]
#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_raw_ident)]
#![feature(extern_prelude)]
// #![deny(unsafe_code)]
// #![deny(warnings)]
// #![feature(lang_items)]
#![no_std]
#![no_main]

// extern crate panic_abort;
extern crate panic_semihosting;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting as sh;
extern crate heapless;
extern crate stm32l432xx_hal as hal;

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;

use rt::ExceptionFrame;
use hal::dma::{dma1, CircBuffer, Event};
use hal::prelude::*;
use hal::serial::{Serial, Event as SerialEvent};
use hal::timer::{Timer, Event as TimerEvent};
use hal::stm32l4::stm32l4x2;
use heapless::RingBuffer;
use rtfm::{app, Threshold};
use core::fmt::Write;
use sh::hio;

/* Our includes */
mod msgmgr;

use msgmgr::Message;
use msgmgr::MessageManager;

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
        static STDOUT: sh::hio::HStdout;
        static BUFFER: [[u8; 64]; 2] = [[0; 64]; 2];
        static CB: CircBuffer<[u8; 64], dma1::C5>;
        static MSG_PAYLOADS: [[u8; 256]; 8] = [[0; 256]; 8];
        static MMGR: MessageManager;
        static RB: heapless::RingBuffer<u8, [u8; 128]> = heapless::RingBuffer::new();
        static USART1_RX: hal::serial::Rx<hal::stm32l4::stm32l4x2::USART1>;
    },

    init: {
        resources: [BUFFER, MSG_PAYLOADS, RB],
    },

    tasks: {
        DMA1_CH5: { /* DMA channel for Usart1 RX */
            path: rx,
            resources: [CB, STDOUT, MMGR],
        },
        USART1: { /* Global usart1 it, uses for idle line detection */
            path: rx_idle,
            resources: [STDOUT, CB, MMGR, USART1_RX],
        },
        TIM2: {
            path: sys_tick,
            resources: [STDOUT, MMGR],
        },
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {

    let mut hstdout = hio::hstdout().unwrap();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.ahb2);
    let mut channels = p.device.DMA1.split(&mut rcc.ahb1);
    
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    
    let mut serial = Serial::usart1(p.device.USART1, (tx, rx), 9_600.bps(), clocks, &mut rcc.apb2);
    serial.listen(SerialEvent::Idle); // Listen to Idle Line detection
    let (_, rx) = serial.split();

    channels.5.listen(Event::HalfTransfer);
    channels.5.listen(Event::TransferComplete);
    
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

    let rb: &'static mut RingBuffer<u8, [u8; 128]> = r.RB; /* Static RB for Msg recieving */

    /* Pass messages to the Message Manager */
    let mmgr = MessageManager::new(msgs, rb);

    let mut systick = Timer::tim2(p.device.TIM2, 1.khz(), clocks, &mut rcc.apb1r1);
    systick.listen(TimerEvent::TimeOut);

    writeln!(hstdout, "Init Complete!");

    init::LateResources {
        CB: rx.circ_read(channels.5, r.BUFFER),
        STDOUT: hstdout,
        MMGR: mmgr,
        USART1_RX: rx
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi(); /* Wait for interrupts */
    }
}
/// Example Incoming payload
/// echo -ne '\x02N\x1FBodyHere!\x03' > /dev/ttyUSB0
fn rx(_t: &mut Threshold, mut r: DMA1_CH5::Resources) {
    let mut mgr = r.MMGR;
    r.CB
        .peek(|buf, _half| {
            match mgr.write(buf) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Failed to write to RingBuffer: {:?}", e);
                }
            }
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
                    match mgr.write(buf) {
                        Ok(_) => {}
                        Err(e) => {
                            panic!("Failed to write to RingBuffer: {:?}", e);
                        }
                    }
                }
                
                Ok( (len, ()) )
            })
            .unwrap();

    }
}

fn sys_tick(_t: &mut Threshold, mut r: TIM2::Resources) {
    let out = &mut r.STDOUT; // .stim[0]
    let mut mgr = r.MMGR;
    mgr.process();
    let msg_count = mgr.msg_count();
    writeln!(out, "MSGS[{}] ", msg_count);
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
}
