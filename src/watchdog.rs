use embedded_hal::watchdog;

use crate::pac::IWDG;
use crate::time::Hertz;

const FEED: u16 = 0xAAAA; // Reset the watchdog value
const START: u16 = 0xCCCC; // Start the watchdog
const ENABLE: u16 = 0x5555; // Enable access to PR, RLR and WINR registers

/// Watchdog instance
pub struct Watchdog {
    iwdg: IWDG,
}

impl watchdog::Watchdog for Watchdog {
    /// Feed the watchdog, so that at least one `period` goes by before the next
    /// reset
    fn feed(&mut self) {
        self.iwdg.kr.write(|w| unsafe { w.key().bits(FEED) });
    }
}

/// Timeout configuration for the IWDG
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct IwdgTimeout {
    psc: u8,
    reload: u16,
}

impl From<Hertz> for IwdgTimeout {
    /// This converts the value so it's usable by the IWDG
    /// Due to conversion losses, the specified frequency is a maximum
    ///
    /// It can also only represent values < 10000 Hertz
    fn from(hz: Hertz) -> Self {
        let mut time = 40_000 / 4 / hz.0;
        let mut psc = 0;
        let mut reload = 0;
        while psc < 7 {
            reload = time;
            if reload < 0x1000 {
                break;
            }
            psc += 1;
            time /= 2;
        }
        // As we get an integer value, reload is always below 0xFFF
        let reload = reload as u16;
        IwdgTimeout { psc, reload }
    }
}

impl Watchdog {
    pub fn new(iwdg: IWDG) -> Self {
        Self { iwdg }
    }
}

impl watchdog::WatchdogEnable for Watchdog {
    type Time = IwdgTimeout;
    fn start<T>(&mut self, period: T)
    where
        T: Into<IwdgTimeout>,
    {
        let time: IwdgTimeout = period.into();
        // Feed the watchdog in case it's already running
        // (Waiting for the registers to update takes sometime)
        self.iwdg.kr.write(|w| unsafe { w.key().bits(FEED) });
        // Enable the watchdog
        self.iwdg.kr.write(|w| unsafe { w.key().bits(START) });
        self.iwdg.kr.write(|w| unsafe { w.key().bits(ENABLE) });
        // Wait until it's safe to write to the registers
        while self.iwdg.sr.read().pvu().bit() {}
        self.iwdg.pr.write(|w| unsafe { w.pr().bits(time.psc) });
        while self.iwdg.sr.read().rvu().bit() {}
        self.iwdg.rlr.write(|w| unsafe { w.rl().bits(time.reload) });
        // Wait until the registers are updated before issuing a reset with
        // (potentially false) values
        while self.iwdg.sr.read().bits() != 0 {}
        self.iwdg.kr.write(|w| unsafe { w.key().bits(FEED) });
    }
}
