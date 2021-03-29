#[cfg(debug_assertions)]
use panic_semihosting as _;

#[cfg(not(debug_assertions))]
use panic_halt as _;

// setup() does all  hal/MCU specific setup and returns generic hal device for use in main code.

#[cfg(feature = "stm32f0xx")] //  eg stm32f030xc
use stm32f0xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    serial::{Rx, Serial, Tx},
};

#[cfg(feature = "stm32f0xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let mut p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);

    let gpioa = p.GPIOA.split(&mut rcc);

    //  stm32f030xc builds with gpiob..into_alternate_af4(cs) USART3 on tx pb10, rx pb11
    //    but stm32f042  only has 2 usarts.
    //  Both have gpioa..into_alternate_af1(cs) USART2 with tx on pa2 and rx pa3

    let (tx, rx) = cortex_m::interrupt::free(move |cs| {
        (
            gpioa.pa2.into_alternate_af1(cs), //tx pa2  for GPS
            gpioa.pa3.into_alternate_af1(cs), //rx pa3  for GPS
        )
    });

    let (tx, rx) = Serial::usart2(p.USART2, (tx, rx), 9600.bps(), &mut rcc).split();

    (tx, rx)
}

#[cfg(feature = "stm32f1xx")] //  eg blue pill stm32f103
use stm32f1xx_hal::{
    device::USART2,
    pac::Peripherals,
    prelude::*,
    serial::{Config, Rx, Serial, Tx}, //, StopBits
};

#[cfg(feature = "stm32f1xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let p = Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(64.mhz())
        .pclk1(32.mhz())
        .freeze(&mut p.FLASH.constrain().acr);

    let mut afio = p.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);

    let (tx, rx) = Serial::usart2(
        p.USART2,
        (
            gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl), //tx pa2  for GPS rx
            gpioa.pa3, //rx pa3  for GPS tx
        ),
        &mut afio.mapr,
        Config::default().baudrate(9_600.bps()),
        clocks,
        &mut rcc.apb1,
    )
    .split();

    (tx, rx)
}

#[cfg(feature = "stm32f3xx")] //  eg Discovery-stm32f303
use stm32f3xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    serial::{Rx, Serial, Tx},
};

#[cfg(feature = "stm32f3xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let p = Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(64.MHz())
        .pclk1(32.MHz())
        .freeze(&mut p.FLASH.constrain().acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);

    let (tx, rx) = Serial::usart2(
        p.USART2,
        (
            gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl), //tx pa2  for GPS rx
            gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl), //rx pa3  for GPS tx
        ),
        9600.Bd(), // 115_200.bps(),
        clocks,
        &mut rcc.apb1,
    )
    .split();

    (tx, rx)
}

#[cfg(feature = "stm32f4xx")]
// eg Nucleo-64 stm32f411, blackpill stm32f411, blackpill stm32f401
use stm32f4xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    serial::{config::Config, Rx, Serial, Tx},
};

#[cfg(feature = "stm32f4xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let p = Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze();

    let gpioa = p.GPIOA.split();

    let (tx, rx) = Serial::usart2(
        p.USART2,
        (
            gpioa.pa2.into_alternate_af7(), //tx pa2  for GPS rx
            gpioa.pa3.into_alternate_af7(), //rx pa3  for GPS tx
        ),
        Config::default().baudrate(9600.bps()),
        clocks,
    )
    .unwrap()
    .split();

    (tx, rx)
}

#[cfg(feature = "stm32f7xx")]
use stm32f7xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    serial::{Config, Oversampling, Rx, Serial, Tx},
};

#[cfg(feature = "stm32f7xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let p = Peripherals::take().unwrap();
    let rcc = p.RCC.constrain();
    let gpioa = p.GPIOA.split();

    // Relative to other hal setups, Serial::new is after spi::new because  clocks partially consumes rcc.
    let clocks = rcc.cfgr.sysclk(216.mhz()).freeze();
    //let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze();

    let (tx, rx) = Serial::new(
        p.USART2,
        (
            gpioa.pa2.into_alternate_af7(), //tx pa2  for GPS
            gpioa.pa3.into_alternate_af7(), //rx pa3  for GPS
        ),
        clocks,
        Config {
            baud_rate: 9600.bps(),
            oversampling: Oversampling::By16,
            character_match: None,
        },
    )
    .split();

    (tx, rx)
}

#[cfg(feature = "stm32h7xx")]
use stm32h7xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    serial::{Rx, Tx},
};

#[cfg(feature = "stm32h7xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let p = Peripherals::take().unwrap();
    let pwr = p.PWR.constrain();
    let vos = pwr.freeze();
    let rcc = p.RCC.constrain();
    let ccdr = rcc.sys_ck(160.mhz()).freeze(vos, &p.SYSCFG);
    let clocks = ccdr.clocks;

    let gpioa = p.GPIOA.split(ccdr.peripheral.GPIOA);

    let (tx, rx) = p
        .USART2
        .serial(
            (
                gpioa.pa2.into_alternate_af7(), //tx pa2 for GPS rx
                gpioa.pa3.into_alternate_af7(), //rx pa3 for GPS tx
            ),
            9600.bps(),
            ccdr.peripheral.USART2,
            &clocks,
        )
        .unwrap()
        .split();

    (tx, rx)
}

#[cfg(feature = "stm32l0xx")]
use stm32l0xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    rcc, // for ::Config but note name conflict with serial
    serial::{Config, Rx, Serial2Ext, Tx},
};

#[cfg(feature = "stm32l0xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.freeze(rcc::Config::hsi16());
    let gpioa = p.GPIOA.split(&mut rcc);

    let (tx, rx) = p
        .USART2
        .usart(
            gpioa.pa2, //tx pa2  for GPS
            gpioa.pa3, //rx pa3  for GPS
            Config::default().baudrate(9600.bps()),
            &mut rcc,
        )
        .unwrap()
        .split();

    (tx, rx)
}

#[cfg(feature = "stm32l1xx")] // eg  Discovery kit stm32l100 and Heltec lora_node STM32L151CCU6
use stm32l1xx_hal::{
    prelude::*,
    rcc, // for ::Config but note name conflict with serial
    serial::{Config, Rx, SerialExt, Tx},
    stm32::{Peripherals, USART1},
};

#[cfg(feature = "stm32l1xx")]
pub fn setup() -> (Tx<USART1>, Rx<USART1>) {
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.freeze(rcc::Config::hsi());

    let gpioa = p.GPIOA.split();

    let (tx, rx) = p
        .USART1
        .usart(
            (
                gpioa.pa9, //tx pa9   for GPS rx
                gpioa.pa10, //rx pa10  for GPS tx
            ),
            Config::default().baudrate(9600.bps()),
            &mut rcc,
        )
        .unwrap()
        .split();

    (tx, rx)
}

#[cfg(feature = "stm32l4xx")]
use stm32l4xx_hal::{
    pac::{Peripherals, USART2},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
};

#[cfg(feature = "stm32l4xx")]
pub fn setup() -> (Tx<USART2>, Rx<USART2>) {
    let p = Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
    let clocks = rcc
        .cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .freeze(&mut flash.acr, &mut pwr);

    let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);

    let (tx, rx) = Serial::usart2(
        p.USART2,
        (
            gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl), //tx pa2  for GPS
            gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl), //rx pa3  for GPS
        ),
        Config::default().baudrate(9600.bps()),
        clocks,
        &mut rcc.apb1r1,
    )
    .split();

    (tx, rx)
}

// End of hal/MCU specific setup. Following should be generic code.
