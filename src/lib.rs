//! HAL for Air001 microcontroller

#![no_std]
#![deny(rustdoc::broken_intra_doc_links)]

pub use air001_pac as pac;

pub mod gpio;
pub mod prelude;
pub mod rcc;
