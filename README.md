# air001-hal

air001-hal contains a hardware abstraction on top of the peripheral access API for the Air001$microcontroller.

Some of the implementation was shamelessly adapted from the [stm32f0xx-hal](https://github.com/stm32-rs/stm32f0xx-hal) crate.

##  Peripheral status

- [ ] RCC: Reset and Clock Control
  - [x] HSI: High Speed Internal
  - [ ] HSE: High Speed External
  - [ ] CSS: Clock Security System
- [ ] GPIO: General Purpose Input/Output
  - [x] GPIOA, GPIOB
  - [ ] GPIOF
- [ ] USART: Universal Synchronous Asynchronous Receiver Transmitter
- [ ] I2C: Inter-intergrated Circuit interface
- [ ] SPI: Serial Peripheral Interface
- [ ] DMA: Direct Memory Access control
- [ ] ADC: Analog to Digital Converter
- [ ] ADTM: Advanced control Timer (TIM1)
- [ ] GPTM: General Purpose Timer (TIM3, TIM14, TIM16, TIM17)
- [ ] LPTIM: Low Power Timer
- [ ] IRTIM: Infrared Timer
- [ ] IWDG: Independent Watchdog
- [ ] WWDG: Window Watchdog
- [ ] COMP: Comparator
- [ ] FLASH: Flash memory and user option bytes
- [ ] PWR: Power control
- [ ] DBG: Debug support
