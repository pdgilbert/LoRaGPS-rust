//! Similar to send_gps.rs with i2c interface to oled using crate and to ads using crate ads1x1x.
//! The ads is set up to monitors battery and load current.
//! Serial interface read GPS on usart and transmit with LoRa using crate radio_sx127x (on SPI).
//!  Using  sck, miso, mosi, cs, reset and D00, D01. Not yet using  D02, D03
//!  For pin connections see the setup() sections in src/lora_spigps_usart.rs.
//! Tested using an RFM95 style radio.

#![no_std]
#![no_main]

#[cfg(debug_assertions)]
use panic_semihosting as _;

#[cfg(not(debug_assertions))]
use panic_halt as _;

use cortex_m::prelude::_embedded_hal_adc_OneShot;
use cortex_m_rt::entry;
use cortex_m_semihosting::*;

use embedded_hal::blocking::delay::DelayMs;
use radio::Transmit;

use heapless::{String, Vec};
use nb::block;

//use embedded_hal::serial::Read;
use old_e_h::serial::Read;

use ads1x1x::{channel as AdcChannel, Ads1x1x, FullScaleRange, SlaveAddr};

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyleBuilder, MonoTextStyle},   //FONT_6X10
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

//use ssd1306::{mode::GraphicsMode, prelude::*, Builder, I2CDIBuilder};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use lora_gps::lora_spi_gps_usart::{setup, LED};

fn display(
    bat_mv: i16,
    bat_ma: i16,
    load_ma: i16,
    temp_c: i16,
    values_b: [i16; 3],
    disp: &mut impl DrawTarget<Color = BinaryColor>,
    //disp : impl DrawTarget<BinaryColor> + WriteOnlyDataCommand,
    //disp : impl DrawTarget<BinaryColor> + cortex_m::prelude::_embedded_hal_serial_Write,
    text_style: MonoTextStyle<BinaryColor>,
) -> () {
    let mut lines: [String<32>; 4] = [
        heapless::String::new(),
        heapless::String::new(),
        heapless::String::new(),
        heapless::String::new(),
    ];

    write!(lines[0], "bat:{:4}mV{:4}mA", bat_mv, bat_ma).unwrap();
    write!(lines[1], "load:    {:5}mA", load_ma).unwrap();
    write!(
        lines[2],
        "B:{:4} {:4} {:4}",
        values_b[0], values_b[1], values_b[2]
    )
    .unwrap();
    write!(lines[3], "temperature{:3} C", temp_c).unwrap();

    let _z = disp.clear(BinaryColor::Off);
    // check for err variant
    for i in 0..lines.len() {
        let _z = Text::new(&lines[i], Point::new(0, i as i32 * 16), text_style)
            //.into_styled(text_style)
            .draw(&mut *disp);
        // check for err variant
    }
    //disp.flush().unwrap();
    ()
}

#[entry]
fn main() -> ! {
    // set this with
    // SENDER_ID="whatever" cargo build ...
    // or  cargo:rustc-env=SENDER_ID="whatever"
    let id = option_env!("SENDER_ID").expect("").as_bytes();

    let (mut lora, _tx_gps, mut rx_gps, i2c, mut led) = setup(); //  lora (delay is available in lora)
    led.off();

    // i2c oled and ads setup

    let manager = shared_bus::BusManager::<cortex_m::interrupt::Mutex<_>, _>::new(i2c);
    let interface = I2CDisplayInterface::new(manager.acquire());

    // set display size 128x32 or 128x64 and Font6x8 or Font8x16
    let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    disp.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13)   //.font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Display initialized ...", Point::zero(), text_style, Baseline::Top)
        .draw(&mut disp)
        .unwrap();

    //let interface = I2CDIBuilder::new().init(manager.acquire());

    // set display size 128x32 or 128x64 and Font6x8 or Font8x16
    //let mut disp: GraphicsMode<_, _> = Builder::new()
    //    .size(DisplaySize128x64)
    //    .connect(interface)
    //    .into();
   // disp.init().unwrap();
//
    //let text_style = TextStyleBuilder::new(Font8x16)
    //    .text_color(BinaryColor::On)
    //    .build();
//
    //Text::new("Display initialized ...", Point::zero())
    //    .into_styled(text_style)
    //    .draw(&mut disp)
    //    .unwrap();

    disp.flush().unwrap();

    //let mut adc = Ads1x1x::new_ads1015(manager.acquire(), SlaveAddr::default()); // = addr = GND
    let mut adc_a = Ads1x1x::new_ads1015(manager.acquire(), SlaveAddr::Alternative(false, false)); //addr = GND
    let mut adc_b = Ads1x1x::new_ads1015(manager.acquire(), SlaveAddr::Alternative(false, true)); //addr =  V

    // set FullScaleRange to measure expected max voltage.
    // This is very small for diff across low value shunt resistors
    //   but up to 5v for single pin with usb power.
    // +- 6.144v , 4.096v, 2.048v, 1.024v, 0.512v, 0.256v
    adc_a
        .set_full_scale_range(FullScaleRange::Within0_256V)
        .unwrap();
    adc_b
        .set_full_scale_range(FullScaleRange::Within4_096V)
        .unwrap();

    // LoRa setup

    // byte buffer   Nov 2020 limit data.len() < 255 in radio_sx127x  .start_transmit
    let mut buffer: Vec<u8, 80> = Vec::new(); // up to 80  u8 elements on stack
    let mut buf2: Vec<u8, 80> = Vec::new(); // up to 80  u8 elements on stack

    buffer.clear();
    buf2.clear();

    let e: u8 = b'x'; // replace char errors with "x"
    let mut good = false; // true while capturing a line

    loop {
        // gps and lora

        let byte = match block!(rx_gps.read()) {
            Ok(byt) => byt,
            Err(_error) => e,
        };

        if byte == 36 {
            //  $ is 36. start of a line
            buffer.clear();
            good = true; //start capturing line
        };

        if good {
            //push byte into buffer then transmit if error/buffer full or end of line. \r is 13, \n is 10
            if buffer.push(byte).is_err() || byte == 13 {
                //hprintln!("{:?}", &buffer).unwrap();

                buf2.clear();
                // put id in first
                for v in id.iter() {
                    buf2.push(*v).unwrap();
                }
                buf2.push(b' ').unwrap();

                // for message $GPRMC == [36, 71, 80, 82, 77, 67]
                //    transmit GPS N and E in hundredths of degrees
                // otherwise transmits the GPS message line

                // if message id is $GPRMC but if buffer is too short then not valid data (V) orsomething was lost, so skip
                //B411-2 $GPRMC,030052.00,V,,,,,,,300321,,,N*7A
                //$GPRMC,031737.00,A,4523.74241,N,07540.61255,W,0.551,,300321,,,A*66
                if (buffer.len() > 45) && (&buffer[0..6] == [36, 71, 80, 82, 77, 67]) {
                    // [19..31] is north/south.
                    for v in buffer[19..31].iter() {
                        buf2.push(*v).unwrap();
                    }
                    buf2.push(b' ').unwrap();
                    // [32..45] is east/west
                    for v in buffer[32..45].iter() {
                        buf2.push(*v).unwrap();
                    }

                    //hprintln!("{:?}", &buf2).unwrap();
                    hprint!(".").unwrap(); // print "."  on transmit of $GPRMC message (but not others)
                    led.on(); // double blink on transmit of decoded message, one here and one below.
                    let _ = lora.try_delay_ms(2u32);
                    led.off();
                    let _ = lora.try_delay_ms(300u32);
                } else {
                    for v in buffer[..].iter() {
                        buf2.push(*v).unwrap();
                    }
                };

                // CONSIDER A FUNCTION
                //  lora_send(&buf2, &lora, &led);
                // TO REPLACE NEXT SECTION, BUT IT GETS MESSY WITH TYPES FOR lora and led

                match lora.start_transmit(&buf2) {
                    Ok(_b) => {
                        led.on();
                        let _ = lora.try_delay_ms(2u32);
                        led.off();
                    }
                    Err(_err) => {
                        hprintln!("Error returned from lora.start_transmit().").unwrap();
                        //panic!("should reset in release mode.");
                    }
                };

                // Note hprintln! requires semihosting. If hprintln! (thus also match section below) are
                // removed then this example works on battery power with no computer attached.
                // (tested only on blackpill with stm32f411 )

                // The first transmission often return false and prints "x", but works after that.
                // If this continually returns "TX not complete" then the radio should probably be reset,
                //  but should avoid panic_reset after first transmission.

                match lora.check_transmit() {
                    Ok(b) => {
                        if !b {
                            hprint!("x").unwrap();
                            // if multible times then panic!("should reset in release mode.");
                        }
                    }
                    Err(_err) => {
                        hprintln!("Error returned from lora.check_transmit().").unwrap();
                        //panic!("should reset in release mode.");
                    }
                };

                // NEXT SECTION TO   buffer.clear(); WILL BE NICER WHEN read_adc IS A FUNCTION

                // HAVE TO FIGURE OUT Ads1x1x TRAIT FOR THIS
                //fn read_adc(adc_a : Ads1x1x, adc_b : Ads1x1x)  -> (i16, i16, i16, i16, [i16; 3]) {
                // Note scale_cur divides, scale_a and scale_b multiply
                let scale_cur = 10; // calibrated to get mA/mV depends on FullScaleRange above and values of shunt resistors
                let scale_a = 2; // calibrated to get mV    depends on FullScaleRange
                let scale_b = 2; // calibrated to get mV    depends on FullScaleRange

                //TMP35 scale is 100 deg C per 1.0v (slope 10mV/deg C) and goes through
                //     <50C, 1.0v>,  so 0.0v is  -50C.

                let scale_temp = 5; //divides
                let offset_temp = 50;

                //first adc  Note that readings are zero on USB power (programming) rather than battery.

                let bat_ma = block!(adc_a.read(&mut AdcChannel::DifferentialA1A3)).unwrap_or(8091)
                    / scale_cur;
                let load_ma = block!(adc_a.read(&mut AdcChannel::DifferentialA2A3)).unwrap_or(8091)
                    / scale_cur;

                // toggle FullScaleRange to measure battery voltage, not just diff across shunt resistor
                adc_a
                    .set_full_scale_range(FullScaleRange::Within4_096V)
                    .unwrap();
                let bat_mv =
                    block!(adc_a.read(&mut AdcChannel::SingleA0)).unwrap_or(8091) * scale_a;
                adc_a
                    .set_full_scale_range(FullScaleRange::Within0_256V)
                    .unwrap();

                // second adc
                let values_b = [
                    block!(adc_b.read(&mut AdcChannel::SingleA0)).unwrap_or(8091) * scale_b,
                    block!(adc_b.read(&mut AdcChannel::SingleA1)).unwrap_or(8091) * scale_b,
                    block!(adc_b.read(&mut AdcChannel::SingleA2)).unwrap_or(8091) * scale_b,
                ];

                let temp_c = block!(adc_b.read(&mut AdcChannel::SingleA3)).unwrap_or(8091)
                    / scale_temp
                    - offset_temp;

                //    (bat_mv, bat_ma, load_ma, temp_c, values_b)
                //};

                //    let (bat_mv, bat_ma, load_ma, temp_c, values_b) = read_adc(adc_a, adc_b);

                display(
                    bat_mv, bat_ma, load_ma, temp_c, values_b, &mut disp, text_style,
                );
                disp.flush().unwrap();

                buffer.clear();
                good = false;
                match lora.try_delay_ms(5000u32) {
                    Ok(b) => b, // b is ()
                    Err(_err) => {
                        hprintln!("Error returned from lora.try_delay_ms().").unwrap();
                        panic!("should reset in release mode.");
                    }
                };
            };
        };
    }
}
