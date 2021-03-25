# Send GPS info over LoRa - embedded Rust (nostd)

## Preamble

pre-alpha.  JUST SETTING UP
This crate compiles Rust code into binaries to run on MCUs with no OS. 
See [repo LoRaGPS](https://pdgilbert.github.io/LoRaGPS/) for similar Python code that 
runs on Raspberry Pi (and like) devices with an OS. 

See [repo eg_stm_hal(https://pdgilbert.github.io/eg_stm_hal/) for examples of other sensors and
[a summary of their status.](https://pdgilbert.github.io/eg_stm_hal/).


##  Contents
- [Building](#building)
- [Running](#running)
- [License](#License)
- [Contribution](#Contribution)


## Summary

| xxx              |   Description                                              |
| ---------------- |:---------------------------------------------------------- |
| lora_spi_send    | transmit a character string over LoRa,  + semihost output  |
| lora_spi_receive | receive  a character string over LoRa,  + semihost output  |
| lora_spi_gps     | read gps and transmit over LoRa,  + semihost output        |

The current status of these examples is [here.](https://pdgilbert.github.io/eg_stm_hal/#additional-examples)

## Building

```
cargo build  --target $TARGET  --features $HAL,$MCU   --bin xxx
```
where `xxx` is  from the table above, and `TARGET`, `HAL`  and `MCU` are
environment variables for your processor. 
Variables `HAL`  and `MCU` overlap. It should be possible to determine  `HAL`  based on `MCU`.
The variable `HAL` is used in the code whereas some of the underlying HAL packages
actually need the specific `MCU`.

```
  export HAL=stm32f0xx MCU=stm32f030xc TARGET=thumbv6m-none-eabi     # none-stm32f030      Cortex-M0
  export HAL=stm32f1xx MCU=stm32f103   TARGET=thumbv7m-none-eabi     # bluepill            Cortex-M3
  export HAL=stm32f1xx MCU=stm32f100   TARGET=thumbv7m-none-eabi     # none-stm32f100      Cortex-M3
  export HAL=stm32f1xx MCU=stm32f101   TARGET=thumbv7m-none-eabi     # none-stm32f101      Cortex-M3
  export HAL=stm32f3xx MCU=stm32f303xc TARGET=thumbv7em-none-eabihf  # discovery-stm32f303 Cortex-M3
  export HAL=stm32f4xx MCU=stm32f401   TARGET=thumbv7em-none-eabihf  # blackpill-stm32f401 Cortex-M4
  export HAL=stm32f4xx MCU=stm32f411   TARGET=thumbv7em-none-eabihf  # blackpill-stm32f411 Cortex-M4
  export HAL=stm32f4xx MCU=stm32f411   TARGET=thumbv7em-none-eabihf  # nucleo-64           Cortex-M4
  export HAL=stm32f7xx MCU=stm32f722   TARGET=thumbv7em-none-eabihf  # none-stm32f722      Cortex-M7
  export HAL=stm32h7xx MCU=stm32h742   TARGET=thumbv7em-none-eabihf  # none-stm32h742      Cortex-M7
  export HAL=stm32l0xx MCU=stm32l0x2   TARGET=thumbv6m-none-eabi     # none-stm32l0x2      Cortex-M0
  export HAL=stm32l1xx MCU=stm32l100   TARGET=thumbv7m-none-eabi     # discovery-stm32l100 Cortex-M3
  export HAL=stm32l1xx MCU=stm32l151   TARGET=thumbv7m-none-eabi     # heltec-lora-node151 Cortex-M3
  export HAL=stm32l4xx MCU=stm32l4x2   TARGET=thumbv7em-none-eabi    # none-stm32l4x1      Cortex-M4
```

## Running 

```
  export INTERFACE=stlink-v2    PROC=stm32f1x  #cheap  dongle and blue pill
  export INTERFACE=stlink-v2-1  PROC=stm32f1x  #better dongle and blue pill
```

```
openocd -f interface/$INTERFACE.cfg -f target/$PROC.cfg 
```

cargo  run --target $TARGET --features $HAL,$MCU  xxx

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
