use core::{
    convert::Infallible,
    fmt::{Result, Write},
    ops::Deref,
};

use embedded_hal::prelude::*;

use crate::{gpio::*, rcc::Rcc, time::Bps};

use core::marker::PhantomData;

/// Serial error
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
}

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
    /// Idle line state detected
    Idle,
}

pub trait TxPin<USART> {}
pub trait RxPin<USART> {}

macro_rules! usart_pins {
    ($($USART:ident => {
        tx => [$($tx:ty),+ $(,)*],
        rx => [$($rx:ty),+ $(,)*],
    })+) => {
        $(
            $(
                impl TxPin<crate::pac::$USART> for $tx {}
            )+
            $(
                impl RxPin<crate::pac::$USART> for $rx {}
            )+
        )+
    }
}

usart_pins! {
    USART1 => {
        tx => [
            gpioa::PA2<Alternate<AF1>>,
            gpioa::PA7<Alternate<AF8>>,
            gpioa::PA14<Alternate<AF1>>,
            gpiob::PB6<Alternate<AF0>>,
            gpiof::PF1<Alternate<AF8>>
        ],
        rx => [
            gpioa::PA3<Alternate<AF1>>,
            gpioa::PA13<Alternate<AF8>>,
            gpiob::PB2<Alternate<AF0>>,
            gpiof::PF0<Alternate<AF8>>
        ],
    }
}

usart_pins! {
    USART2 => {
        tx => [
            gpioa::PA0<Alternate<AF9>>,
            gpioa::PA2<Alternate<AF4>>,
            gpioa::PA4<Alternate<AF9>>,
            gpioa::PA7<Alternate<AF9>>,
            gpioa::PA14<Alternate<AF4>>,
            gpiob::PB6<Alternate<AF4>>,
            gpiof::PF0<Alternate<AF9>>,
            gpiof::PF1<Alternate<AF4>>,
        ],
        rx => [
            gpioa::PA1<Alternate<AF9>>,
            gpioa::PA3<Alternate<AF4>>,
            gpioa::PA5<Alternate<AF9>>,
            gpiob::PB2<Alternate<AF3>>,
            gpiof::PF0<Alternate<AF4>>,
            gpiof::PF1<Alternate<AF9>>,
        ],
    }
}


/// Serial abstraction
pub struct Serial<USART, TXPIN, RXPIN> {
    usart: USART,
    pins: (TXPIN, RXPIN),
}

// Common register
type SerialRegisterBlock = crate::pac::usart1::RegisterBlock;

/// Serial receiver
pub struct Rx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Rx<USART> {}

/// Serial transmitter
pub struct Tx<USART> {
    usart: *const SerialRegisterBlock,
    _instance: PhantomData<USART>,
}

// NOTE(unsafe) Required to allow protected shared access in handlers
unsafe impl<USART> Send for Tx<USART> {}

macro_rules! usart {
    ($($USART:ident: ($usart:ident, $usarttx:ident, $usartrx:ident, $usartXen:ident, $apbenr:ident),)+) => {
        $(
            use crate::pac::$USART;
            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN>
            where
                TXPIN: TxPin<$USART>,
                RXPIN: RxPin<$USART>,
            {
                /// Creates a new serial instance
                pub fn $usart(usart: $USART, pins: (TXPIN, RXPIN), baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let mut serial = Serial { usart, pins };
                    serial.configure(baud_rate, rcc);
                    // Enable transmission and receiving
                    serial.usart.cr1.modify(|_, w| w.te().set_bit().re().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<TXPIN> Serial<$USART, TXPIN, ()>
            where
                TXPIN: TxPin<$USART>,
            {
                /// Creates a new tx-only serial instance
                pub fn $usarttx(usart: $USART, txpin: TXPIN, baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let rxpin = ();
                    let mut serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(baud_rate, rcc);
                    // Enable transmission
                    serial.usart.cr1.modify(|_, w| w.te().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<RXPIN> Serial<$USART, (), RXPIN>
            where
                RXPIN: RxPin<$USART>,
            {
                /// Creates a new rx-only serial instance
                pub fn $usartrx(usart: $USART, rxpin: RXPIN, baud_rate: Bps, rcc: &mut Rcc) -> Self
                {
                    let txpin = ();
                    let mut serial = Serial { usart, pins: (txpin, rxpin) };
                    serial.configure(baud_rate, rcc);
                    // Enable receiving
                    serial.usart.cr1.modify(|_, w| w.re().set_bit().ue().set_bit());
                    serial
                }
            }

            impl<TXPIN, RXPIN> Serial<$USART, TXPIN, RXPIN> {
                fn configure(&mut self, baud_rate: Bps, rcc: &mut Rcc) {
                    // Enable clock for USART
                    rcc.regs.$apbenr.modify(|_, w| w.$usartXen().set_bit());

                    // Calculate correct baudrate divisor on the fly
                    // FIXME: correct rcc setup
                    // let brr = rcc.clocks.pclk().0 / baud_rate.0;
                    let brr = 8000000 / baud_rate.0;
                    self.usart.brr.write(|w| unsafe { w.bits(brr) });

                    // Reset other registers to disable advanced USART features
                    self.usart.cr2.reset();
                    self.usart.cr3.reset();
                }

                /// Starts listening for an interrupt event
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().set_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().set_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().set_bit())
                        },
                    }
                }

                /// Stop listening for an interrupt event
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().clear_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().clear_bit())
                        },
                        Event::Idle => {
                            self.usart.cr1.modify(|_, w| w.idleie().clear_bit())
                        },
                    }
                }

                /// Returns true if the line idle status is set
                pub fn is_idle(&self) -> bool {
                    self.usart.sr.read().idle().bit_is_set()
                }

                /// Returns true if the tx register is empty
                pub fn is_txe(&self) -> bool {
                    self.usart.sr.read().txe().bit_is_set()
                }

                /// Returns true if the rx register is not empty (and can be read)
                pub fn is_rx_not_empty(&self) -> bool {
                    self.usart.sr.read().rxne().bit_is_set()
                }

                /// Returns true if transmission is complete
                pub fn is_tx_complete(&self) -> bool {
                    self.usart.sr.read().tc().bit_is_set()
                }
            }
        )+
    }
}

usart! {
    USART1: (usart1, usart1tx, usart1rx, usart1en, apbenr2),
}

usart! {
    USART2: (usart2, usart2tx, usart2rx,usart2en, apbenr1),
}

impl<USART> embedded_hal::serial::Read<u8> for Rx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(self.usart)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Read<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    RXPIN: RxPin<USART>,
{
    type Error = Error;

    /// Tries to read a byte from the uart
    fn read(&mut self) -> nb::Result<u8, Error> {
        read(&*self.usart)
    }
}

impl<USART> embedded_hal::serial::Write<u8> for Tx<USART>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    type Error = Infallible;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> embedded_hal::serial::Write<u8> for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    TXPIN: TxPin<USART>,
{
    type Error = Infallible;

    /// Ensures that none of the previously written words are still buffered
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        flush(&*self.usart)
    }

    /// Tries to write a byte to the uart
    /// Fails if the transmit buffer is full
    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        write(&*self.usart, byte)
    }
}

impl<USART, TXPIN, RXPIN> Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
{
    /// Splits the UART Peripheral in a Tx and an Rx part
    /// This is required for sending/receiving
    pub fn split(self) -> (Tx<USART>, Rx<USART>)
    where
        TXPIN: TxPin<USART>,
        RXPIN: RxPin<USART>,
    {
        (
            Tx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
            Rx {
                usart: &*self.usart,
                _instance: PhantomData,
            },
        )
    }

    pub fn release(self) -> (USART, (TXPIN, RXPIN)) {
        (self.usart, self.pins)
    }
}

impl<USART> Write for Tx<USART>
where
    Tx<USART>: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

impl<USART, TXPIN, RXPIN> Write for Serial<USART, TXPIN, RXPIN>
where
    USART: Deref<Target = SerialRegisterBlock>,
    TXPIN: TxPin<USART>,
{
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

/// Ensures that none of the previously written words are still buffered
fn flush(usart: *const SerialRegisterBlock) -> nb::Result<(), Infallible> {
    // NOTE(unsafe) atomic read with no side effects
    let isr = unsafe { (*usart).sr.read() };

    if isr.tc().bit_is_set() {
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to write a byte to the UART
/// Returns `Err(WouldBlock)` if the transmit buffer is full
fn write(usart: *const SerialRegisterBlock, byte: u8) -> nb::Result<(), Infallible> {
    // NOTE(unsafe) atomic read with no side effects
    let isr = unsafe { (*usart).sr.read() };

    if isr.txe().bit_is_set() {
        // NOTE(unsafe) atomic write to stateless register
        unsafe { (*usart).dr.write(|w| w.dr().bits(byte as u16)) }
        Ok(())
    } else {
        Err(nb::Error::WouldBlock)
    }
}

/// Tries to read a byte from the UART
fn read(usart: *const SerialRegisterBlock) -> nb::Result<u8, Error> {
    // NOTE(unsafe) atomic read with no side effects
    let isr = unsafe { (*usart).sr.read() };

    // NOTE(unsafe) read dr after sr clears pe,fe,ne,ore
    let data = unsafe {(*usart).dr.read().dr().bits() as u8};

    if isr.pe().bit_is_set() {
        Err(nb::Error::Other(Error::Parity))
    } else if isr.fe().bit_is_set() {
        Err(nb::Error::Other(Error::Framing))
    } else if isr.ne().bit_is_set() {
        Err(nb::Error::Other(Error::Noise))
    } else if isr.ore().bit_is_set() {
        Err(nb::Error::Other(Error::Overrun))
    } else if isr.rxne().bit_is_set() {
        Ok(data)
    } else {
        Err(nb::Error::WouldBlock)
    }
}
