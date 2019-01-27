//! Kernel types
//!
//! Exposes all the types the sdk may want to use, which the kernel provides

#![no_std]

pub use stm32l4xx_hal as hal;

/// Type Alias to use in resource definitions
pub type Ssd1351 = ssd1351::mode::GraphicsMode<
    ssd1351::interface::SpiInterface<
        hal::spi::Spi<
            hal::stm32l4::stm32l4x2::SPI1,
            (
                hal::gpio::gpioa::PA5<
                    hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>,
                >,
                hal::gpio::gpioa::PA6<
                    hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>,
                >,
                hal::gpio::gpioa::PA7<
                    hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>,
                >,
            ),
        >,
        hal::gpio::gpiob::PB1<hal::gpio::Output<hal::gpio::PushPull>>,
    >,
>;
pub type BatteryManagementIC = max17048::Max17048<
    hal::i2c::I2c<
        hal::stm32::I2C1,
        (
            hal::gpio::gpioa::PA9<
                hal::gpio::Alternate<hal::gpio::AF4, hal::gpio::Output<hal::gpio::OpenDrain>>,
            >,
            hal::gpio::gpioa::PA10<
                hal::gpio::Alternate<hal::gpio::AF4, hal::gpio::Output<hal::gpio::OpenDrain>>,
            >,
        ),
    >,
>;
pub type RightButton = hal::gpio::gpiob::PB5<
    hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::PushPull>>,
>;
pub type MiddleButton = hal::gpio::gpiob::PB6<
    hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::PushPull>>,
>;
pub type LeftButton = hal::gpio::gpiob::PB7<
    hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::PushPull>>,
>;

pub type InputHandlerFn = extern "C" fn(*mut Context, bool) -> i32;

pub type SetupFn = extern "C" fn() -> i32;
pub type ServiceFn = extern "C" fn(*mut Context) -> i32;

pub enum InputType {
    Left,
    Middle,
    Right,
}

pub static mut CONTEXT_POINTER: Option<&'static mut Context> = None;

pub struct Context<'a> {
    pub display: &'a mut Ssd1351,
}

// is this safe?
unsafe impl<'a> Send for Context<'a> {}

#[repr(C)]
/// The callbacks supplied by the OS.
pub struct Table {
    /// Draw a colour on the display - x, y, colour
    pub draw_pixel: extern "C" fn(*mut Context, u8, u8, u16) -> i32,
    /// Register an input event handler for an input
    pub register_input: extern "C" fn(*mut Context, InputType, InputHandlerFn) -> i32,
}

pub static CALLBACK_TABLE: Table = Table {
    draw_pixel: draw_pixel,
    register_input: register_input
};

impl<'a> Context<'a> {
    pub fn get() -> &'static mut Context<'a> {
        unsafe {
            if let Some(tbl) = &mut CONTEXT_POINTER {
                tbl
            } else {
                panic!("Bad context, context is only valid within update()!");
            }
        }
    }
}

//TODO is this safe? It's only safe if when we launch an application we never draw anything else
// i.e we give control of the display to the application
pub extern "C" fn draw_pixel(context: *mut Context, x: u8, y: u8, colour: u16) -> i32 {
    let ctx = unsafe { &mut *context };
    ctx.display.set_pixel(x as u32, y as u32, colour);
    0
}

pub extern "C" fn register_input(_context: *mut Context, _input_type: InputType, _cb: InputHandlerFn) -> i32 {
    unimplemented!()
}
