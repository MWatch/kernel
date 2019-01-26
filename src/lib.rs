//! Kernel types
//! 
//! Exposes all the types the sdk may want to use, which the kernel provides

#![no_std]

extern crate stm32l4xx_hal as hal;

/// Type Alias to use in resource definitions
pub type Ssd1351 = ssd1351::mode::GraphicsMode<ssd1351::interface::SpiInterface<hal::spi::Spi<hal::stm32l4::stm32l4x2::SPI1, (hal::gpio::gpioa::PA5<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>, hal::gpio::gpioa::PA6<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>, hal::gpio::gpioa::PA7<hal::gpio::Alternate<hal::gpio::AF5, hal::gpio::Input<hal::gpio::Floating>>>)>, hal::gpio::gpiob::PB1<hal::gpio::Output<hal::gpio::PushPull>>>>;
pub type BatteryManagementIC = max17048::Max17048<hal::i2c::I2c<hal::stm32::I2C1, (hal::gpio::gpioa::PA9<hal::gpio::Alternate<hal::gpio::AF4, hal::gpio::Output<hal::gpio::OpenDrain>>>, hal::gpio::gpioa::PA10<hal::gpio::Alternate<hal::gpio::AF4, hal::gpio::Output<hal::gpio::OpenDrain>>>)>>;
pub type RightButton = hal::gpio::gpiob::PB5<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::PushPull>>>;
pub type MiddleButton = hal::gpio::gpiob::PB6<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::PushPull>>>;
pub type LeftButton = hal::gpio::gpiob::PB7<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::PushPull>>>;

pub type InputHandlerFn = extern "C" fn(*mut Context, bool) -> i32;

pub enum InputType {
    Left,
    Middle,
    Right,
}

pub struct Context<'a> {
    pub display: &'a mut Ssd1351
}

/// Pointer to the structure we're given by the host.
pub static mut TABLE_POINTER: Option<&'static Table> = None;

#[repr(C)]
/// The callbacks supplied by the OS.
pub struct Table<'a> {
    pub context: *mut Context<'a>,
    /// Draw a colour on the display - x, y, colour
    pub draw_pixel: extern "C" fn(*mut Context, u8, u8, u16) -> i32,
    /// Register an input event handler for an input
    pub register_input: extern "C" fn(*mut Context, InputType, InputHandlerFn) -> i32,
}

impl<'a> Table<'a> {
    pub fn get() -> &'static Table<'a> {
        unsafe {
            if let Some(tbl) = &TABLE_POINTER {
                tbl
            } else {
                panic!("Bad context");
            }
        }
    }
}

//TODO is this safe? It's only safe if when we launch an application we never draw anything else
// i.e we give control of the display to the application
pub extern "C" fn draw_pixel(context: *mut Context, x: u8, y: u8, colour: u16) -> i32 {
    let ctx = unsafe {
        &mut *context
    };
    ctx.display.set_pixel(x as u32, y as u32, colour);
    0
}