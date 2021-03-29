## Send GPS info over LoRa - embedded Rust (no_std)

This crate compiles Rust code into binaries to run on MCUs with no OS. 
See [repo LoRaGPS](https://github.com/pdgilbert/LoRaGPS) for similar Python code that 
runs on Raspberry Pi (and like) devices with an OS. 

See [repo eg_stm_hal](https://github.com/pdgilbert/eg_stm_hal) for examples of other sensors and
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


## Building

```
cargo build  --target $TARGET  --features $HAL,$MCU
cargo build  --target $TARGET  --features $HAL,$MCU   --bin lora_spi_receive
SENDER_ID="whatever"  cargo build  --target $TARGET  --features $HAL,$MCU   --bin lora_spi_send
SENDER_ID="whatever"  cargo build  --target $TARGET  --features $HAL,$MCU   --bin lora_spi_gps

cargo test  --target $TARGET  --features $HAL,$MCU

```
where  `TARGET`, `HAL`  and `MCU` are environment variables for your processor.
SENDER_ID is optional. If supplied it will prepend sent messages. 
This is useful when there are many sending systems.
Variables `HAL`  and `MCU` overlap. It should be possible to determine  `HAL`  based on `MCU`.
The variable `HAL` is used in the code whereas some of the underlying HAL packages
actually need the specific `MCU`.

```
              cargo run  environment variables                        openocd        test board and processor
  _____________________________________________________________     _____________   ___________________________
  export HAL=stm32f0xx MCU=stm32f042   TARGET=thumbv6m-none-eabi    PROC=stm32f0x  # none-stm32f042      Cortex-M0
  export HAL=stm32f0xx MCU=stm32f030xc TARGET=thumbv6m-none-eabi    PROC=stm32f0x  # none-stm32f030      Cortex-M0
  export HAL=stm32f1xx MCU=stm32f103   TARGET=thumbv7m-none-eabi    PROC=stm32f1x  # bluepill            Cortex-M3
  export HAL=stm32f1xx MCU=stm32f100   TARGET=thumbv7m-none-eabi    PROC=stm32f1x  # none-stm32f100      Cortex-M3
  export HAL=stm32f1xx MCU=stm32f101   TARGET=thumbv7m-none-eabi    PROC=stm32f1x  # none-stm32f101      Cortex-M3
  export HAL=stm32f3xx MCU=stm32f303xc TARGET=thumbv7em-none-eabihf PROC=stm32f3x  # discovery-stm32f303 Cortex-M3
  export HAL=stm32f4xx MCU=stm32f401   TARGET=thumbv7em-none-eabihf PROC=stm32f4x  # blackpill-stm32f401 Cortex-M4
  export HAL=stm32f4xx MCU=stm32f411   TARGET=thumbv7em-none-eabihf PROC=stm32f4x  # blackpill-stm32f411 Cortex-M4
  export HAL=stm32f4xx MCU=stm32f411   TARGET=thumbv7em-none-eabihf PROC=stm32f4x  # nucleo-64           Cortex-M4
  export HAL=stm32f7xx MCU=stm32f722   TARGET=thumbv7em-none-eabihf PROC=stm32f7x  # none-stm32f722      Cortex-M7
  export HAL=stm32h7xx MCU=stm32h742   TARGET=thumbv7em-none-eabihf PROC=          # none-stm32h742      Cortex-M7
  export HAL=stm32l0xx MCU=stm32l0x2   TARGET=thumbv6m-none-eabi    PROC=stm32l0   # none-stm32l0x2      Cortex-M0
  export HAL=stm32l1xx MCU=stm32l100   TARGET=thumbv7m-none-eabi    PROC=stm32l1   # discovery-stm32l100 Cortex-M3
  export HAL=stm32l1xx MCU=stm32l151   TARGET=thumbv7m-none-eabi    PROC=stm32l1   # heltec-lora-node151 Cortex-M3
  export HAL=stm32l4xx MCU=stm32l4x2   TARGET=thumbv7em-none-eabi   PROC=stm32l4x  # none-stm32l4x1      Cortex-M4
```

Note that there can be conflicting versions of the binary files produced in `target/$TARGET` because 
the directory is only specific to the MCU triple, not to the actual MCU. 
For example, `blackpill-stm32f401` and `blackpill-stm32f411` are linked with different memory maps but are 
generated in the same directroy, so the last compile $MCU setting will determine what is in the directory.
See directories in `memoryMaps/`  to find the `memory.x` files. 
The use of these is controlled by the `build.rs` script, which is called by `cargo`.

## Running 
 
  Depending on the MCU connection to the computer, in the  openocd command use
```
    export INTERFACE=stlink-v2  
    export INTERFACE=stlink-v2-1  
```

Using openocd  to load compiled code to the MCU and for semihost or debugging:

```
openocd -f interface/$INTERFACE.cfg -f target/$PROC.cfg 
```

Build and load in a separate window with one of

```
cargo  run --target $TARGET --features $HAL,$MCU  --bin  lora_spi_receive
SENDER_ID="whatever"  cargo  run --target $TARGET --features $HAL,$MCU  --bin  lora_spi_send
SENDER_ID="whatever"  cargo  run --target $TARGET --features $HAL,$MCU  --bin  lora_spi_gps

```

See `FREQUENCY` in the code to set the channel. (Still not an feature option.)
Channels are as follows

```
   'CH_00_900': 903.08, 'CH_01_900': 905.24, 'CH_02_900': 907.40,
   'CH_03_900': 909.56, 'CH_04_900': 911.72, 'CH_05_900': 913.88,
   'CH_06_900': 916.04, 'CH_07_900': 918.20, 'CH_08_900': 920.36,
   'CH_09_900': 922.52, 'CH_10_900': 924.68, 'CH_11_900': 926.84, 'CH_12_900': 915,

   'CH_10_868': 865.20, 'CH_11_868': 865.50, 'CH_12_868': 865.80,
   'CH_13_868': 866.10, 'CH_14_868': 866.40, 'CH_15_868': 866.70,
   'CH_16_868': 867   , 'CH_17_868': 868   ,
```
See https://www.rfwireless-world.com/Tutorials/LoRa-channels-list.html for more detail.


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
