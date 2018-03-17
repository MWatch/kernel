
#![feature(proc_macro)]
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx_hal as hal;

use hal::dma::{CircBuffer, Event, dma1};
use hal::prelude::*;
use hal::serial::Serial;
use hal::stm32f103xx;
use rtfm::{app, Threshold};

/* Our includes */
mod msgmgr;

use msgmgr::MessageManager;
use msgmgr::Message;


app! {
    device: stm32f103xx,

    resources: {
        static BUFFER: [[u8; 8]; 2] = [[0; 8]; 2];
        static CB: CircBuffer<[u8; 8], dma1::C6>;
        static STDOUT: cortex_m::peripheral::ITM;
        static MSG_PAYLOADS: [[u8; 256]; 8] = [[0; 256]; 8];
        static MMGR: MessageManager;
    },

    init: {
        resources: [BUFFER, MSG_PAYLOADS],
    },

    tasks: {
        DMA1_CHANNEL6: {
            path: rx,
            resources: [CB, STDOUT, MMGR],
        },
        SYS_TICK: {
            path: sys_tick,
            resources: [STDOUT, MMGR],
        },
    }
}

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {

    let mut itm = p.core.ITM;
    iprintln!(&mut itm.stim[0], "Hello, world!");

    /* Enable SYS_TICK IT */
    let mut syst = p.core.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(1);
    syst.enable_interrupt();
    syst.enable_counter();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = p.device.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = p.device.GPIOA.split(&mut rcc.apb2);

    /* USART2 */
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

    /* Define out block of message - surely there must be a nice way to to this? */
    let msgs: [msgmgr::Message; 8] = [ 
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[0] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[1] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[2] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[3] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[4] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[5] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[6] },
        Message { msg_type: msgmgr::MessageType::Unknown, payload: r.MSG_PAYLOADS[7] },
    ];

    /* Pass messages to the Message Manager */
    let mmgr = MessageManager::new(msgs);

    init::LateResources {
        CB: rx.circ_read(channels.6, r.BUFFER),
        STDOUT : itm,
        MMGR: mmgr,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn rx(_t: &mut Threshold, mut r: DMA1_CHANNEL6::Resources) {
    let out = &mut r.STDOUT.stim[0];
    
    r.CB
        .peek(|_buf, _half| {
            for x in _buf {
                iprint!(out, "{}", *x as char);
            }
            iprintln!(out, "");
        })
        .unwrap();
}

fn sys_tick(_t: &mut Threshold, mut r: SYS_TICK::Resources){
    let out = &mut r.STDOUT.stim[0];
    // iprintln!(out, "DMA6: Message[0].msg_type = {:?}", r.MMGR.msg_pool[0].msg_type);
}
