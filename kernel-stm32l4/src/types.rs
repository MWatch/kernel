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
pub type BatteryManagementInterface = hal::i2c::I2c<
    hal::stm32::I2C1,
    (
        hal::gpio::gpioa::PA9<
            hal::gpio::Alternate<hal::gpio::AF4, hal::gpio::Output<hal::gpio::OpenDrain>>,
        >,
        hal::gpio::gpioa::PA10<
            hal::gpio::Alternate<hal::gpio::AF4, hal::gpio::Output<hal::gpio::OpenDrain>>,
        >,
    ),
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

cfg_if::cfg_if! {
    if #[cfg(feature = "itm")] {
        pub type LoggerType = cortex_m_log::log::Logger<cortex_m_log::printer::itm::ItmSync<cortex_m_log::modes::InterruptFree>>;
    } else if #[cfg(feature = "rtt")] {
        pub type LoggerType = crate::rtt::RttLog;
    }
}

pub type ChargeStatusPin = hal::gpio::gpioa::PA12<hal::gpio::Input<hal::gpio::PullUp>>;
pub type StandbyStatusPin = hal::gpio::gpioa::PA11<hal::gpio::Input<hal::gpio::PullUp>>;
pub type TouchSenseController = hal::tsc::Tsc<
    hal::gpio::gpiob::PB4<
        hal::gpio::Alternate<hal::gpio::AF9, hal::gpio::Output<hal::gpio::OpenDrain>>,
    >,
>;
pub type BluetoothConnectedPin = hal::gpio::gpioa::PA8<hal::gpio::Input<hal::gpio::Floating>>;
