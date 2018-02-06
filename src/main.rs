//! Prints "Hello, world!" on the OpenOCD console using semihosting
//!
//! ---

#![feature(used)]
#![no_std]

#[macro_use(singleton)]
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate stm32f103xx_hal as hal;
#[macro_use(block)]
extern crate nb;

use core::fmt::Write;
use cortex_m::{asm};
use cortex_m_semihosting::hio;
use hal::prelude::*;
use hal::serial::Serial;
use hal::stm32f103xx;
use hal::dma::Half;



fn main() {
    
    let p = stm32f103xx::Peripherals::take().unwrap();

    // writeln!(stdout, "Hello, world!").unwrap();

    /* Borrow peripherals, flash and rcc(clocks) */
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    /* Lock clocks */
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    /* Alternate function IO (SERIAL/SPI/I2C etc) */
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let mut channels = p.DMA1.split(&mut rcc.ahb);

    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    // let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

    // USART2
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;

    /* Initialze UART2 */
    let serial = Serial::usart2(
        p.USART2,   /* Peripheral */
        (tx, rx),   /* Pin Tuple */
        &mut afio.mapr,
        9_600.bps(),    /* Baud rate */
        clocks,         /*  Periph clock speed */
        &mut rcc.apb1,
    );

    let (mut tx, mut rx) = serial.split();

    let buf = singleton!(: [[u8; 32]; 2] = [[0; 32]; 2]).unwrap();

    let mut circ_buffer = rx.circ_read(channels.6, buf);
    /* Gets the prt of the dma chan in the buffer */
    // let dtr_in = channels.6.cndtr(); // private?
    loop {
        read_dma(&mut circ_buffer);
    }
}

fn read_dma(circ_buffer: &mut hal::dma::CircBuffer<[u8; 32], hal::dma::dma1::C6>){
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Reading First Half").unwrap();

    /* Sits spinning till dma half complete is set */
    while circ_buffer.readable_half().unwrap() != Half::First {}

    /* When we have half cplt read the buffer */
    let _first_half = circ_buffer.peek(|half, _| *half).unwrap();
    // print_buff(&_first_half);

    /* Then do the same for the second half of the buff */
    writeln!(stdout, "Reading Second Half").unwrap();
    while circ_buffer.readable_half().unwrap() != Half::Second {}

    let _second_half = circ_buffer.peek(|half, _| *half).unwrap();
    print_buff(&_second_half);
}

fn print_buff(array: &[u8; 32]){
    let mut stdout = hio::hstdout().unwrap();
    for x in array {
        writeln!(stdout, "{}", *x as char).unwrap();
    }
}

// As we are not using interrupts, we just register a dummy catch all handler
#[link_section = ".vector_table.interrupts"]
#[used]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}
