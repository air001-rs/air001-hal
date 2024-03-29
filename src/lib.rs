//! HAL for Air001 microcontroller

#![no_std]
#![deny(rustdoc::broken_intra_doc_links)]

pub use air001_pac as pac;

pub mod delay;
pub mod gpio;
pub mod prelude;
pub mod pwm;
pub mod rcc;
pub mod serial;
pub mod spi;
pub mod time;
pub mod timers;
pub mod watchdog;
