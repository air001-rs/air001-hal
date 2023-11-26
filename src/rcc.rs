//! # Reset & Clock Control
use crate::pac::RCC;
use fugit::HertzU32 as Hertz;

use self::inner::SystClkSource;

pub trait RccExt {
    // Configure the clocks of the RCC peripheral
    fn configure(self) -> CFGR;
}

impl RccExt for RCC {
    fn configure(self) -> CFGR {
        CFGR {
            hclk: None,
            pclk: None,
            sysclk: None,
            clock_src: SystClkSource::HSI,
            rcc: self,
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    pub clocks: Clocks,
    pub regs: RCC, // TODO: should be pub(crate)
}

/// RCC for Air001.
mod inner {
    use crate::pac::RCC;

    pub(super) const HSI: u32 = 8_000_000; // 8 MHz

    pub(super) enum SystClkSource {
        HSI,
    }

    pub(super) fn get_freq(c_src: &SystClkSource) -> u32 {
        match c_src {
            SystClkSource::HSI => HSI,
        }
    }

    pub(super) fn enable_clock(rcc: &mut RCC, c_src: &SystClkSource) {
        match c_src {
            SystClkSource::HSI => {
                // enable HSI
                rcc.cr.write(|w| w.hsion().set_bit());
                // wait until HSI is ready
                while rcc.cr.read().hsirdy().bit_is_clear() {}
            }
        }
    }

    pub(super) fn enable_pll(rcc: &mut RCC, c_src: &SystClkSource, ppre_bits: u8, hpre_bits: u8) {
        // Set PLL source
        match (c_src) {
            SystClkSource::HSI => rcc.pllcfgr.modify(|_, w| w.pllsrc().clear_bit()),
        }

        // Enable PLL and wait until PLL is ready
        rcc.cr.modify(|_, w| w.pllon().set_bit());
        while rcc.cr.read().pllrdy().bit_is_clear() {}

        // Use PLL CLK as SYSCLK, set APB and AHB prescaler
        rcc.cfgr.modify(|_, w| unsafe {
            w.ppre()
                .bits(ppre_bits)
                .hpre()
                .bits(hpre_bits)
                .sw()
                .bits(0b010) // PLL CLK
        });
    }
}

pub struct CFGR {
    hclk: Option<u32>,
    pclk: Option<u32>,
    sysclk: Option<u32>,
    clock_src: SystClkSource,
    rcc: RCC,
}

impl CFGR {
    pub fn hclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hclk = Some(freq.into().raw());
        self
    }

    pub fn pclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk = Some(freq.into().raw());
        self
    }

    pub fn sysclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sysclk = Some(freq.into().raw());
        self
    }

    pub fn freeze(mut self, flash: &mut crate::pac::FLASH) -> Rcc {
        // Default to HSI
        let sysclk = self.sysclk.unwrap_or(self::inner::HSI);

        let r_sysclk; // The "real" sysclock value, calculated below
        let enable_pll;

        // Select clock source based on user input and capability
        // Highest selected frequency source available takes precedent.
        let src_clk_freq = self::inner::get_freq(&self.clock_src);

        // PLL check
        if sysclk == src_clk_freq {
            r_sysclk = src_clk_freq;
            enable_pll = false;
        } else {
            r_sysclk = src_clk_freq * 2;
            enable_pll = true;
        }

        let hpre_bits = self
            .hclk
            .map(|hclk| match r_sysclk / hclk {
                0 => unreachable!(),
                1 => 0b0111,
                2 => 0b1000,
                3..=5 => 0b1001,
                6..=11 => 0b1010,
                12..=39 => 0b1011,
                40..=95 => 0b1100,
                96..=191 => 0b1101,
                192..=383 => 0b1110,
                _ => 0b1111,
            })
            .unwrap_or(0b0111);

        let hclk = r_sysclk / (1 << (hpre_bits - 0b0111));

        let ppre_bits = self
            .pclk
            .map(|pclk| match hclk / pclk {
                0 => unreachable!(),
                1 => 0b011,
                2 => 0b100,
                3..=5 => 0b101,
                6..=11 => 0b110,
                _ => 0b111,
            })
            .unwrap_or(0b011);

        let ppre: u8 = 1 << (ppre_bits - 0b011);
        let pclk = hclk / (ppre as u32);

        // Adjust flash wait state.
        unsafe { flash.acr.write(|w| w.latency().bit(r_sysclk <= 24_000_000)) }

        // Enable the requested clock
        self::inner::enable_clock(&mut self.rcc, &self.clock_src);

        // Enable PLL
        if enable_pll {
            self::inner::enable_pll(&mut self.rcc, &mut self.clock_src, ppre_bits, hpre_bits)
        } else {
            // Use HSI as source
            self.rcc.cfgr.modify(|_, w| unsafe {
                w.ppre()
                    .bits(ppre_bits)
                    .hpre()
                    .bits(hpre_bits)
                    .sw()
                    .bits(0b010) // PLL CLK
            });
        }

        Rcc {
            clocks: Clocks {
                hclk: Hertz::Hz(hclk),
                pclk: Hertz::Hz(pclk),
                sysclk: Hertz::Hz(sysclk),
            },
            regs: self.rcc,
        }
    }
}

/// Frozen clock frequency
///
/// The existence of this value indicates that the clock configuration can no longer be changed.
pub struct Clocks {
    hclk: Hertz,
    pclk: Hertz,
    sysclk: Hertz,
}

/// Frozen clock frequencies
impl Clocks {
    // Returns the frequency of th AHB
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    // Returns the frequency of the APB
    pub fn pclk(&self) -> Hertz {
        self.pclk
    }

    // Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
