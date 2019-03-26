//! API for SDK and type exports


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

pub type LoggerType = cortex_m_log::log::Logger<cortex_m_log::printer::itm::ItmSync<cortex_m_log::modes::InterruptFree>>;
pub type ChargeStatusPin = hal::gpio::gpioa::PA12<hal::gpio::Input<hal::gpio::PullUp>>;
pub type StandbyStatusPin = hal::gpio::gpioa::PA11<hal::gpio::Input<hal::gpio::PullUp>>;
pub type TouchSenseController = hal::tsc::Tsc<hal::gpio::gpiob::PB4<hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::OpenDrain>>>>;
pub type BluetoothConnectedPin = hal::gpio::gpioa::PA8<hal::gpio::Input<hal::gpio::Floating>>;

pub type InputHandlerFn = extern "C" fn(*mut Context, bool) -> i32;

pub type SetupFn = extern "C" fn() -> i32;
pub type ServiceFn = extern "C" fn(*mut Context) -> i32;
pub type InputFn = extern "C" fn(*mut Context, InputEvent) -> i32;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Left,
    Middle,
    Right,
    Dual,
    Multi,
    LeftMiddle,
    RightMiddle,
}

pub static mut CONTEXT_POINTER: Option<&'static mut Context> = None;

pub struct Context<'a> {
    pub display: Option<&'a mut Ssd1351>,
    pub log: extern "C" fn(&str) -> i32,
}

/// WARNING only safe if we guarentee the safety ourselves, i.e context doesn't live longer than the &mut references that it contains
unsafe impl<'a> Send for Context<'a> {}

#[repr(C)]
/// The callbacks supplied by the OS.
pub struct Table {
    /// Draw a colour on the display - x, y, colour
    pub draw_pixel: unsafe extern "C" fn(*mut Context, u8, u8, u16) -> i32,
    /// Print a string using th info! macro
    pub print: unsafe extern "C" fn(*mut Context, &str) -> i32,
}

pub static CALLBACK_TABLE: Table = Table {
    draw_pixel,
    print
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


/// Assumes control over the display, it is up to use to make sure the display is not borrowed by anything else
pub unsafe extern "C" fn draw_pixel(context: *mut Context, x: u8, y: u8, colour: u16) -> i32 {
    let ctx =&mut *context;
    // let display = ctx.display.expect("Display invoked in an invalid application state");
    if let Some(display) = &mut ctx.display {
        display.set_pixel(u32::from(x), u32::from(y), colour);
    } else {
        panic!("Display invoked in an invalid state. Applications can only use the display within update.")
    }
    0
}

pub unsafe extern "C" fn print(context: *mut Context, string: &str) -> i32 {
    let ctx = &mut *context;
    (ctx.log)(string);
    0
}
