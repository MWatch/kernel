//! Prints "Hello, world!" on the OpenOCD console using semihosting
//!
//! ---

#![feature(used)]
#![no_std]

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


fn main() {
    let mut stdout = hio::hstdout().unwrap();
    let p = stm32f103xx::Peripherals::take().unwrap();

    writeln!(stdout, "Hello, world!").unwrap();

    /* Borrow peripherals, flash and rcc(clocks) */
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    /* Lock clocks */
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    /* Alternate function IO (SERIAL/SPI/I2C etc) */
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

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

    let sent = b'Y';
    loop {
        writeln!(stdout, "Transmitting : {}", sent).unwrap();
        block!(tx.write(sent)).ok();
        writeln!(stdout, "Waiting on resp").unwrap();
        let received = block!(rx.read());
        match received {
            Ok(byte) => writeln!(stdout, "We recieved: {:b}", byte),
            Err(why)      => {
                panic!("Failed to read a byte {:?}", why) 
            }
        };
    }
}

// As we are not using interrupts, we just register a dummy catch all handler
#[link_section = ".vector_table.interrupts"]
#[used]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}
