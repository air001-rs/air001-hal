# air001-hal

air001-hal contains a hardware abstraction on top of the peripheral access API for the Air001 microcontroller.

See [air001-quickstart](https://github.com/air001-rs/air001-quickstart) for quickstart guide. 

Some of the implementation was shamelessly adapted from the [stm32f0xx-hal](https://github.com/stm32-rs/stm32f0xx-hal) crate.

##  Peripheral status

Not fully tested yet.

- [ ] RCC: Reset and Clock Control
  - [x] HSI: High Speed Internal
  - [ ] HSE: High Speed External
  - [ ] CSS: Clock Security System
- [x] GPIO: General Purpose Input/Output
- [x] USART: Universal Synchronous Asynchronous Receiver Transmitter
- [ ] I2C: Inter-intergrated Circuit interface
- [ ] SPI: Serial Peripheral Interface
- [ ] DMA: Direct Memory Access control
- [ ] ADC: Analog to Digital Converter
- [x] ADTM: Advanced control Timer (TIM1)
- [x] GPTM: General Purpose Timer (TIM3, TIM14, TIM16, TIM17)
- [ ] LPTIM: Low Power Timer
- [ ] IRTIM: Infrared Timer
- [x] IWDG: Independent Watchdog
- [ ] WWDG: Window Watchdog
- [ ] COMP: Comparator
- [ ] FLASH: Flash memory and user option bytes
- [ ] PWR: Power control
- [ ] DBG: Debug support

