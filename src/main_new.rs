//! Blinks an LED

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate cortex_m_semihosting as sh;
extern crate stm32l432xx_hal as hal;
#[macro_use(block)]
extern crate nb;

use hal::prelude::*;
use hal::stm32l4::stm32l4x2;

use hal::delay::Delay;
use rt::ExceptionFrame;

use core::fmt::Write;
use sh::hio;

entry!(main);

fn main() -> ! {

    let mut hstdout = hio::hstdout().unwrap();

    writeln!(hstdout, "Hello, world!").unwrap();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32l4x2::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain(); // .constrain();
    let mut rcc = dp.RCC.constrain();

    // Try a different clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let clocks = rcc.cfgr
    //     .sysclk(64.mhz())
    //     .pclk1(32.mhz())
    //     .freeze(&mut flash.acr);

    // let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    // let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.afrh);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
    let mut led = gpiob.pb3.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    
    let mut timer = Delay::new(cp.SYST, clocks);
    loop {
        // block!(timer.wait()).unwrap();
        timer.delay_ms(1000 as u32);
        led.set_high();
        // block!(timer.wait()).unwrap();
        timer.delay_ms(1000 as u32);
        led.set_low();
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
