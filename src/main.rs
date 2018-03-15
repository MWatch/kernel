
#![feature(proc_macro)]
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx_hal as hal;

use cortex_m::asm;
use hal::dma::{CircBuffer, Event, dma1};
use hal::prelude::*;
use hal::serial::Serial;
use hal::stm32f103xx;
use rtfm::{app, Threshold};
use core::fmt::Write;


app! {
    device: stm32f103xx,

    resources: {
        static BUFFER: [[u8; 8]; 2] = [[0; 8]; 2];
        static CB: CircBuffer<[u8; 8], dma1::C6>;
        static STDOUT: cortex_m::peripheral::ITM;
    },

    init: {
        resources: [BUFFER],
    },

    tasks: {
        DMA1_CHANNEL6: {
            path: rx,
            resources: [CB, STDOUT],
        },
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {

    let mut itm = p.core.ITM;
    iprintln!(&mut itm.stim[0], "Hello, world!");

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);

    // USART2
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;


    let serial = Serial::usart2(
        p.device.USART2,
        (tx, rx),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb1,
    );

    let (_tx, rx) = serial.split();

    let mut channels = p.device.DMA1.split(&mut rcc.ahb);
    
    channels.6.listen(Event::HalfTransfer);
    channels.6.listen(Event::TransferComplete);

    init::LateResources {
        CB: rx.circ_read(channels.6, r.BUFFER),
        STDOUT : itm,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn rx(_t: &mut Threshold, mut r: DMA1_CHANNEL6::Resources) {
    let prnt = &mut r.STDOUT.stim[0];
    r.CB
        .peek(|_buf, _half| {
            for x in _buf {
                iprint!(prnt, "{}", *x as char);
            }
            iprintln!(prnt, "");
        })
        .unwrap();
}
