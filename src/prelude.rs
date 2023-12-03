pub use embedded_hal::prelude::*;

pub use embedded_hal::digital::v2::InputPin as _embedded_hal_gpio_InputPin;
pub use embedded_hal::digital::v2::OutputPin as _embedded_hal_gpio_OutputPin;
pub use embedded_hal::digital::v2::StatefulOutputPin as _embedded_hal_gpio_StatefulOutputPin;
pub use embedded_hal::digital::v2::ToggleableOutputPin as _embedded_hal_gpio_ToggleableOutputPin;

pub use crate::gpio::GpioExt as _air001_hal_gpio_GpioExt;
pub use crate::rcc::RccExt as _air001_hal_rcc_RccExt;
pub use crate::time::U32Ext as _stm32f0xx_hal_time_U32Ext;
