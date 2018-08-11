#![feature(use_extern_macros)]
#![feature(proc_macro_gen)]
#![feature(proc_macro_mod)]
#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_raw_ident)]
#![deny(unsafe_code)]
#![feature(extern_prelude)]
// #![deny(warnings)]
// #![feature(lang_items)]
#![no_std]
#![no_main]

extern crate panic_abort;
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate heapless;
extern crate stm32l432xx_hal as hal;

#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;

use rt::ExceptionFrame;
use hal::dma::{dma1, CircBuffer, Event};
use hal::prelude::*;
use hal::serial::Serial;
// use hal::i2c::{I2c, Mode};
use cortex_m::asm;
use hal::stm32l4::stm32l4x2;
use heapless::RingBuffer;
use rtfm::atomic;
use rtfm::{app, Threshold};

/* Our includes */
mod msgmgr;

use msgmgr::Message;
use msgmgr::MessageManager;

// const CB_HALF_LEN: usize = 64; /* Buffer size of DMA Half */
// const MSG_PAYLOAD_SIZE: usize = 256; /* Body Of payload */
// const MSG_COUNT: usize = 8; /* Number of message to store */

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
        static STDOUT: cortex_m::peripheral::ITM;
        static BUFFER: [[u8; 64]; 2] = [[0; 64]; 2];
        static CB: CircBuffer<[u8; 64], dma1::C5>;
        static MSG_PAYLOADS: [[u8; 256]; 8] = [[0; 256]; 8];
        static MMGR: MessageManager;
        static RB: heapless::RingBuffer<u8, [u8; 128]> = heapless::RingBuffer::new();
    },

    init: {
        resources: [BUFFER, MSG_PAYLOADS, RB],
    },

    tasks: {
        DMA1_CH5: {
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
    syst.set_reload(1_000_000); // 1_000_000 / 80_000_000, where 80_000_000 is HCLK
    syst.enable_interrupt();
    syst.enable_counter();

    // let p = stm32l4x2::Peripherals::take().unwrap();

    let mut flash = p.device.FLASH.constrain();
    let mut rcc = p.device.RCC.constrain();
    let mut gpioa = p.device.GPIOA.split(&mut rcc.ahb2);
    let mut channels = p.device.DMA1.split(&mut rcc.ahb1);
    
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    
    let serial = Serial::usart1(p.device.USART1, (tx, rx), 9_600.bps(), clocks, &mut rcc.apb2);
    let (mut tx, mut rx) = serial.split();

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
    // let mode = Mode::Standard { frequency: 100.khz().0 };
    // let sclk = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    // let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    // let mut i2c = I2c::i2c1(p.device.I2C1, (sclk, sda), &mut afio.mapr, mode, clocks, &mut rcc.apb1);

    // let byte = [0xFF];
    // i2c.write(0x3C, &byte).unwrap();

    /* Pass messages to the Message Manager */
    let mmgr = MessageManager::new(msgs, rb);

    init::LateResources {
        CB: rx.circ_read(channels.5, r.BUFFER),
        STDOUT: itm,
        MMGR: mmgr,
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
    let out = &mut r.STDOUT.stim[0];
    let mut mgr = r.MMGR;
    r.CB
        .peek(|buf, _half| {
            match mgr.write(buf) {
                Ok(_) => {}
                Err(e) => {
                    iprintln!(out, "Failed to write to RingBuffer: {:?}", e);
                }
            }
        })
        .unwrap();
}

fn sys_tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    let out = &mut r.STDOUT.stim[0];
    let mut mgr = r.MMGR;
    mgr.process(); // TODO IMPLEMENT - probably can be interrupted
                   // atomic(_t, |_cs| { // dont interrrupt the printint process, so we run it atomically
                   //     mgr.print_rb(out);
                   // });
    let msg_count = mgr.msg_count();

    for i in 0..msg_count {
        mgr.peek_message(i, |msg| {
            let payload: &[u8] = &msg.payload;
            let len = msg.payload_idx;
            if len > 0 {
                iprintln!(out, "MSG[{}] ", i);
                // for byte in payload {
                //     iprint!(out, "{}", *byte as char);
                // }
                // iprintln!(out, "");
            }
            // Payload is in the variable payload
        });
    }
}
