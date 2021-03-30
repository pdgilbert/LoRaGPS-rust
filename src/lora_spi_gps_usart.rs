#[cfg(debug_assertions)]
use panic_semihosting as _;

#[cfg(not(debug_assertions))]
use panic_halt as _;

use core::convert::Infallible;

use embedded_hal::blocking::delay::DelayMs;

// The embedded_hal_compat crate is to smooth the transition for hal crates that are
// not yet based on embedded_hal 1.0.0-alpha while rust-radio-sx127x is.
// When passing the older hal crate objects to the newer rust-radio-sx127x methods
// the objects are appended with .compat().

use embedded_hal_compat::IntoCompat;

// MODE needs the old version as it is passed to the device hal crates
use old_e_h::spi::{Mode, Phase, Polarity};

use radio_sx127x::Error as sx127xError; // Error name conflict with hals
use radio_sx127x::{
    device::lora::{
        Bandwidth, CodingRate, FrequencyHopping, LoRaChannel, LoRaConfig, PayloadCrc,
        PayloadLength, SpreadingFactor,
    },
    device::{Channel, Modem, PaConfig, PaSelect},
    prelude::*, // prelude has Sx127x,
};

// trait needs to be in scope to find  methods start_transmit and check_transmit.
use radio::{Receive, Transmit};

// lora and radio parameters

pub const MODE: Mode = Mode {
    //  SPI mode for radio
    phase: Phase::CaptureOnSecondTransition,
    polarity: Polarity::IdleHigh,
};

pub const FREQUENCY: u32 = 907_400_000; // frequency in hertz ch_12: 915_000_000, ch_2: 907_400_000

pub const CONFIG_CH: LoRaChannel = LoRaChannel {
    freq: FREQUENCY as u32, // frequency in hertz
    bw: Bandwidth::Bw125kHz,
    sf: SpreadingFactor::Sf7,
    cr: CodingRate::Cr4_8,
};

pub const CONFIG_LORA: LoRaConfig = LoRaConfig {
    preamble_len: 0x8,
    symbol_timeout: 0x64,
    payload_len: PayloadLength::Variable,
    payload_crc: PayloadCrc::Enabled,
    frequency_hop: FrequencyHopping::Disabled,
    invert_iq: false,
};

//   compare other settings in python version
//    lora.set_mode(sx127x_lora::RadioMode::Stdby).unwrap();
//    set_tx_power(level, output_pin) level >17 => PA_BOOST.
//    lora.set_tx_power(17,1).unwrap();
//    lora.set_tx_power(15,1).unwrap();

//baud = 1000000 is this needed for spi or just USART ?

pub const CONFIG_PA: PaConfig = PaConfig {
    output: PaSelect::Boost,
    power: 10,
};

//let CONFIG_RADIO = Config::default() ;

pub const CONFIG_RADIO: radio_sx127x::device::Config = radio_sx127x::device::Config {
    modem: Modem::LoRa(CONFIG_LORA),
    channel: Channel::LoRa(CONFIG_CH),
    pa_config: CONFIG_PA,
    xtal_freq: 32000000, // CHECK
    timeout_ms: 100,
};

// setup() does all  hal/MCU specific setup and returns generic object for use in main code.

#[cfg(feature = "stm32f0xx")] //  eg stm32f030xc
use stm32f0xx_hal::{
    delay::Delay,
    pac::{CorePeripherals, Peripherals, USART2},
    prelude::*,
    serial::{Rx, Serial, Tx},
    spi::{Error, Spi},
};

#[cfg(feature = "stm32f0xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Infallible, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Infallible, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    //  Infallible, Infallible   reflect the error type on the spi and gpio traits.

    let cp = CorePeripherals::take().unwrap();
    let mut p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiob = p.GPIOB.split(&mut rcc);

    let (sck, miso, mosi, _rst, pa1, pb8, pb9, pa0, tx, rx) =
        cortex_m::interrupt::free(move |cs| {
            (
                gpioa.pa5.into_alternate_af0(cs), //    sck     on PA5
                gpioa.pa6.into_alternate_af0(cs), //   miso     on PA6
                gpioa.pa7.into_alternate_af0(cs), //   mosi     on PA7
                //gpioa.pa1.into_push_pull_output(cs),  //  cs     on PA1
                gpiob.pb1.into_push_pull_output(cs), //   reset    on PB1
                gpioa.pa1.into_push_pull_output(cs), //   CsPin    on PA1
                gpiob.pb8.into_floating_input(cs),   //   BusyPin  on PB8 DIO0
                gpiob.pb9.into_floating_input(cs),   //   ReadyPin on PB9 DIO1
                gpioa.pa0.into_push_pull_output(cs), //   ResetPin on PA0
                gpioa.pa2.into_alternate_af1(cs),    //tx pa2  for GPS
                gpioa.pa3.into_alternate_af1(cs),    //rx pa3  for GPS
            )
        });

    let spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE, 8.mhz(), &mut rcc);

    let delay = Delay::new(cp.SYST, &rcc);

    // Create lora radio instance

    let lora = Sx127x::spi(
        spi.compat(),
        pa1.compat(),
        pb8.compat(),
        pb9.compat(),
        pa0.compat(),
        delay.compat(),
        &CONFIG_RADIO,
    )
    .unwrap(); // should handle error

    //  stm32f030xc builds with gpiob..into_alternate_af4(cs) USART3 on tx pb10, rx pb11
    //    but stm32f042  only has 2 usarts.
    //  Both have gpioa..into_alternate_af1(cs) USART2 with tx on pa2 and rx pa3

    // This is done for tx, rx above because move |cs| consumes gpioa
    // let (tx, rx) = cortex_m::interrupt::free(move |cs| {...});

    let (tx, rx) = Serial::usart2(p.USART2, (tx, rx), 9600.bps(), &mut rcc).split();

    (lora, tx, rx)
}

#[cfg(feature = "stm32f1xx")] //  eg blue pill stm32f103
use stm32f1xx_hal::{
    delay::Delay,
    device::USART2,
    pac::{CorePeripherals, Peripherals},
    prelude::*,
    serial::{Config, Rx, Serial, Tx}, //, StopBits
    spi::{Error, Spi},
};

#[cfg(feature = "stm32f1xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Infallible, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Infallible, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(64.mhz())
        .pclk1(32.mhz())
        .freeze(&mut p.FLASH.constrain().acr);

    let mut afio = p.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

    let spi = Spi::spi1(
        p.SPI1,
        (
            gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl), //   sck   on PA5
            gpioa.pa6.into_floating_input(&mut gpioa.crl),      //   miso  on PA6
            gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl), //   mosi  on PA7
        ),
        &mut afio.mapr,
        MODE,
        8.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let delay = Delay::new(cp.SYST, clocks);

    // Create lora radio instance

    let lora = Sx127x::spi(
        spi.compat(),                                             //Spi
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl).compat(), //CsPin         on PA1
        gpiob.pb8.into_floating_input(&mut gpiob.crh).compat(),   //BusyPin  DIO0 on PB8
        gpiob.pb9.into_floating_input(&mut gpiob.crh).compat(),   //ReadyPin DIO1 on PB9
        gpioa.pa0.into_push_pull_output(&mut gpioa.crl).compat(), //ResetPin      on PA0
        delay.compat(),                                           //Delay
        &CONFIG_RADIO,                                            //&Config
    )
    .unwrap(); // should handle error

    let (tx, rx) = Serial::usart2(
        p.USART2,
        (
            gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl), //tx pa2  for GPS rx
            gpioa.pa3,                                          //rx pa3  for GPS tx
        ),
        &mut afio.mapr,
        Config::default().baudrate(9_600.bps()),
        clocks,
        &mut rcc.apb1,
    )
    .split();

    (lora, tx, rx)
}

#[cfg(feature = "stm32f3xx")] //  eg Discovery-stm32f303
use stm32f3xx_hal::{
    delay::Delay,
    pac::{CorePeripherals, Peripherals, USART2},
    prelude::*,
    serial::{Rx, Serial, Tx},
    spi::{Error, Spi},
};

#[cfg(feature = "stm32f3xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Infallible, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Infallible, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(64.MHz())
        .pclk1(32.MHz())
        .freeze(&mut p.FLASH.constrain().acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb);

    let spi = Spi::spi1(
        p.SPI1,
        (
            gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl), // sck   on PA5
            gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl), // miso  on PA6
            gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl), // mosi  on PA7
        ),
        MODE,
        8_000_000.Hz(),
        clocks,
        &mut rcc.apb2,
    );

    let delay = Delay::new(cp.SYST, clocks);

    // Create lora radio instance

    let lora = Sx127x::spi(
        spi.compat(), //Spi
        gpioa
            .pa1
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .compat(), //CsPin   on PA1
        gpiob
            .pb8
            .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
            .compat(), //BusyPin  DIO0 on PB8
        gpiob
            .pb9
            .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
            .compat(), //ReadyPin DIO1 on PB9
        gpioa
            .pa0
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .compat(), //ResetPin      on PA0
        delay.compat(), //Delay
        &CONFIG_RADIO, //&Config
    )
    .unwrap(); // should handle error

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

    (lora, tx, rx)
}

#[cfg(feature = "stm32f4xx")]
// eg Nucleo-64 stm32f411, blackpill stm32f411, blackpill stm32f401
use stm32f4xx_hal::{
    delay::Delay,
    pac::{CorePeripherals, Peripherals, USART2},
    prelude::*,
    serial::{config::Config, Rx, Serial, Tx},
    spi::{Error, Spi},
    time::MegaHertz,
};

// If the type for the lora object is needed somewhere other than just in the setup() return type then it
// may be better to explicitly define it as follows.
//
//    use embedded_spi::wrapper::Wrapper;
//
//    type LoraType = Sx127x<Wrapper<Spi<SPI1,
//                           (PA5<Alternate<AF5>>,    PA6<Alternate<AF5>>,   PA7<Alternate<AF5>>)>,  Error,
//                   PA1<Output<PushPull>>,  PB8<Input<Floating>>,  PB9<Input<Floating>>,  PA0<Output<PushPull>>,
//                   Infallible,  Delay>,  Error, Infallible, Infallible>;
// then
//    pub fn setup() ->  LoraType {

#[cfg(feature = "stm32f4xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Infallible, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Infallible, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();

    let spi = Spi::spi1(
        p.SPI1,
        (
            gpioa.pa5.into_alternate_af5(), // sck   on PA5
            gpioa.pa6.into_alternate_af5(), // miso  on PA6
            gpioa.pa7.into_alternate_af5(), // mosi  on PA7
        ),
        MODE,
        MegaHertz(8).into(),
        clocks,
    );

    let delay = Delay::new(cp.SYST, clocks);

    // Create lora radio instance

    // open_drain_output is really input and output. BusyPin is just input, but I think this should work
    //            gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh),
    // however, gives trait bound  ... InputPin` is not satisfied

    let lora = Sx127x::spi(
        spi.compat(),                               //Spi
        gpioa.pa1.into_push_pull_output().compat(), //CsPin         on PA1
        gpiob.pb8.into_floating_input().compat(),   //BusyPin  DI00 on PB8
        gpiob.pb9.into_floating_input().compat(),   //ReadyPin DI01 on PB9
        gpioa.pa0.into_push_pull_output().compat(), //ResetPin      on PA0
        delay.compat(),                             //Delay
        &CONFIG_RADIO,                              //&Config
    )
    .unwrap(); // should handle error

    //DIO0  triggers RxDone/TxDone status.
    //DIO1  triggers RxTimeout and other errors status.
    //D02, D03 ?

    //lora.lora_configure( config_lora, &config_ch ).unwrap(); # not yet pub, to change something

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

    (lora, tx, rx)
}

#[cfg(feature = "stm32f7xx")]
use stm32f7xx_hal::{
    delay::Delay,
    pac::{CorePeripherals, Peripherals, USART2},
    prelude::*,
    serial::{Config, Oversampling, Rx, Serial, Tx},
    spi::{ClockDivider, Error, Spi},
};

#[cfg(feature = "stm32f7xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Infallible, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Infallible, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();

    let sck = gpioa.pa5.into_alternate_af5(); // sck   on PA5
    let miso = gpioa.pa6.into_alternate_af5(); // miso  on PA6
    let mosi = gpioa.pa7.into_alternate_af5(); // mosi  on PA7

    //   somewhere 8.mhz needs to be set in spi

    let spi =
        Spi::new(p.SPI1, (sck, miso, mosi)).enable::<u8>(&mut rcc.apb2, ClockDivider::DIV32, MODE);

    //let clocks = rcc.cfgr.sysclk(216.mhz()).freeze();
    let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze();

    let delay = Delay::new(cp.SYST, clocks);

    // Create lora radio instance

    // spi::new  partially consumes rcc which causes problem for second use of clocks
    let lora = Sx127x::spi(
        spi.compat(),                               //Spi
        gpioa.pa1.into_push_pull_output().compat(), //CsPin         on PA1
        gpiob.pb8.into_floating_input().compat(),   //BusyPin  DIO0 on PB8
        gpiob.pb9.into_floating_input().compat(),   //ReadyPin DIO1 on PB9
        gpioa.pa0.into_push_pull_output().compat(), //ResetPin      on PA0
        delay.compat(),                             //Delay
        &CONFIG_RADIO,                              //&Config
    )
    .unwrap(); // should handle error

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

    (lora, tx, rx)
}

#[cfg(feature = "stm32h7xx")]
use stm32h7xx_hal::{
    delay::Delay,
    pac::{CorePeripherals, Peripherals, USART2},
    prelude::*,
    serial::{Rx, Tx},
    spi::Error,
    Never,
};

#[cfg(feature = "stm32h7xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Never, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Never, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let pwr = p.PWR.constrain();
    let vos = pwr.freeze();
    let rcc = p.RCC.constrain();
    let ccdr = rcc.sys_ck(160.mhz()).freeze(vos, &p.SYSCFG);
    let clocks = ccdr.clocks;

    let gpioa = p.GPIOA.split(ccdr.peripheral.GPIOA);
    let gpiob = p.GPIOB.split(ccdr.peripheral.GPIOB);

    // following github.com/stm32-rs/stm32h7xx-hal/blob/master/examples/spi.rs
    let spi = p.SPI1.spi(
        (
            gpioa.pa5.into_alternate_af5(), // sck   on PA5
            gpioa.pa6.into_alternate_af5(), // miso  on PA6
            gpioa.pa7.into_alternate_af5(), // mosi  on PA7
        ),
        MODE,
        8.mhz(),
        ccdr.peripheral.SPI1,
        &clocks,
    );

    let delay = Delay::new(cp.SYST, clocks);

    // Create lora radio instance

    let lora = Sx127x::spi(
        spi.compat(),                               //Spi
        gpioa.pa1.into_push_pull_output().compat(), //CsPin         on PA1
        gpiob.pb8.into_floating_input().compat(),   //BusyPin  DIO0 on PB8
        gpiob.pb9.into_floating_input().compat(),   //ReadyPin DIO1 on PB9
        gpioa.pa0.into_push_pull_output().compat(), //ResetPin      on PA0
        delay.compat(),                             //Delay
        &CONFIG_RADIO,                              //&Config
    )
    .unwrap(); // should handle error

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

    (lora, tx, rx)
}

#[cfg(feature = "stm32l0xx")]
use stm32l0xx_hal::{
    pac::{CorePeripherals, Peripherals, USART2},
    prelude::*,
    rcc, // for ::Config but note name conflict with serial
    serial::{Config, Rx, Serial2Ext, Tx},
    spi::Error,
};

#[cfg(feature = "stm32l0xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, void::Void, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, void::Void, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.freeze(rcc::Config::hsi16());
    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiob = p.GPIOB.split(&mut rcc);

    // following  github.com/stm32-rs/stm32l0xx-hal/blob/master/examples/spi.rs
    let spi = p.SPI1.spi(
        (
            gpioa.pa5, // sck   on PA5
            gpioa.pa6, // miso  on PA6
            gpioa.pa7, // mosi  on PA7
        ),
        MODE,
        8.mhz(),
        &mut rcc,
    );

    let delay = cp.SYST.delay(rcc.clocks);

    // Create lora radio instance

    let lora = Sx127x::spi(
        spi.compat(),                               //Spi
        gpioa.pa1.into_push_pull_output().compat(), //CsPin         on PA1
        gpiob.pb8.into_floating_input().compat(),   //BusyPin  DIO0 on PB8
        gpiob.pb9.into_floating_input().compat(),   //ReadyPin DIO1 on PB9
        gpioa.pa0.into_push_pull_output().compat(), //ResetPin      on PA0
        delay.compat(),                             //Delay
        &CONFIG_RADIO,                              //&Config
    )
    .unwrap(); // should handle error

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

    (lora, tx, rx)
}

#[cfg(feature = "stm32l1xx")] // eg  Discovery kit stm32l100 and Heltec lora_node STM32L151CCU6
use stm32l1xx_hal::{
    prelude::*,
    rcc, // for ::Config but note name conflict with serial
    serial::{Config, Rx, SerialExt, Tx},
    spi::Error,
    stm32::{CorePeripherals, Peripherals, USART1},
};

#[cfg(feature = "stm32l1xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Infallible, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Infallible, Infallible>>,
    Tx<USART1>,
    Rx<USART1>,
) {
    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut rcc = p.RCC.freeze(rcc::Config::hsi());

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();

    let spi = p.SPI1.spi(
        (
            gpioa.pa5, // sck   on PA5  in board on Heltec
            gpioa.pa6, // miso  on PA6  in board on Heltec
            gpioa.pa7, // mosi  on PA7  in board on Heltec
        ),
        MODE,
        8.mhz(),
        &mut rcc,
    );

    let delay = cp.SYST.delay(rcc.clocks);

    // Create lora radio instance

    //  Heltec lora_node STM32L151CCU6
    let lora = Sx127x::spi(
        spi.compat(),                               //Spi
        gpioa.pa4.into_push_pull_output().compat(), //CsPin         on PA4  in board on Heltec
        gpiob.pb11.into_floating_input().compat(),  //BusyPin  DIO0 on PB11 in board on Heltec
        gpiob.pb10.into_floating_input().compat(),  //ReadyPin DIO1 on PB10 in board on Heltec
        gpioa.pa3.into_push_pull_output().compat(), //ResetPin      on PA3  in board on Heltec
        delay.compat(),                             //Delay
        &CONFIG_RADIO,                              //&Config
    )
    .unwrap(); // should handle error

    let (tx, rx) = p
        .USART1
        .usart(
            (
                gpioa.pa9,  //tx pa9   for GPS rx
                gpioa.pa10, //rx pa10  for GPS tx
            ),
            Config::default().baudrate(9600.bps()),
            &mut rcc,
        )
        .unwrap()
        .split();

    (lora, tx, rx)
}

#[cfg(feature = "stm32l4xx")]
use stm32l4xx_hal::{
    delay::Delay,
    pac::{CorePeripherals, Peripherals, USART2},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
    spi::{Error, Spi},
};

#[cfg(feature = "stm32l4xx")]
pub fn setup() -> (
    impl DelayMs<u32>
        + Transmit<Error = sx127xError<Error, Infallible, Infallible>>
        + Receive<Info = PacketInfo, Error = sx127xError<Error, Infallible, Infallible>>,
    Tx<USART2>,
    Rx<USART2>,
) {
    let cp = CorePeripherals::take().unwrap();
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
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);

    let spi = Spi::spi1(
        p.SPI1,
        (
            gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl), // sck   on PA5
            gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl), // miso  on PA6
            gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl), // mosi  on PA7
        ),
        MODE,
        8.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let delay = Delay::new(cp.SYST, clocks);

    // Create lora radio instance

    let lora = Sx127x::spi(
        spi.compat(), //Spi
        gpioa
            .pa1
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .compat(), //CsPin   on PA1
        gpiob
            .pb8
            .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
            .compat(), //BusyPin  DIO0 on PB8
        gpiob
            .pb9
            .into_floating_input(&mut gpiob.moder, &mut gpiob.pupdr)
            .compat(), //ReadyPin DIO1 on PB9
        gpioa
            .pa0
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
            .compat(), //ResetPin      on PA0
        delay.compat(), //Delay
        &CONFIG_RADIO, //&Config
    )
    .unwrap(); // should handle error

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

    (lora, tx, rx)
}

// End of hal/MCU specific setup. Following should be generic code.
