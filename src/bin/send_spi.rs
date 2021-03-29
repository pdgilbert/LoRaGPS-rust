//! Transmit a simple message with LoRa using crate radio_sx127x (on SPI).
//!  Using  sck, miso, mosi, cs, reset and D00, D01. Not yet using  D02, D03
//!  For pin connections see the setup() sections in src/lora_spi.rs.
//! Tested using an RFM95 style radio.

#![no_std]
#![no_main]

#[cfg(debug_assertions)]
use panic_semihosting as _;

#[cfg(not(debug_assertions))]
use panic_halt as _;
//use panic_reset;

use cortex_m_rt::entry;
use cortex_m_semihosting::*;
use embedded_hal::blocking::delay::DelayMs;

use radio::Transmit;

use lora_gps::lora_spi;

#[entry]
fn main() -> ! {
    // set this with
    // SENDER_ID="whatever" cargo build ...
    // or  cargo:rustc-env=SENDER_ID="whatever"
    let id = option_env!("SENDER_ID").expect("Hello, LoRa!").as_bytes();

    let mut lora = lora_spi::setup(); //delay is available in lora

    // print out configuration (for debugging)

    //    let v = lora.lora_get_config();
    //    hprintln!("configuration {}", v).unwrap();

    //    hprintln!("chammel          {}", lora.get_chammel()).unwrap();

    //hprintln!("mode                  {}", lora.get_mode()).unwrap();
    //hprintln!("mode                  {}", lora.read_register(Register::RegOpMode.addr())).unwrap();
    //hprintln!("bandwidth          {:?}", lora.get_signal_bandwidth()).unwrap();
    //hprintln!("coding_rate          {:?}",  lora.get_coding_rate_4()).unwrap();
    //hprintln!("spreading_factor {:?}",  lora.get_spreading_factor()).unwrap();
    //hprintln!("spreading_factor {:?}",
    //hprintln!("invert_iq          {:?}",  lora.get_invert_iq()).unwrap();
    //hprintln!("tx_power          {:?}",  lora.get_tx_power()).unwrap();

    // transmit something

    //let buffer = &[0xaa, 0xbb, 0xcc];

    let message = id;
    //let message = b"Hello, LoRa!";

    //let mut buffer = [0;100];      //Nov 2020 limit data.len() < 255 in radio_sx127x  .start_transmit
    //for (i,c) in message.chars().enumerate() {
    //        buffer[i] = c as u8;
    //        }

    loop {
        lora.start_transmit(message).unwrap(); // should handle error

        match lora.check_transmit() {
            Ok(b) => {
                if b {
                    hprintln!("TX complete").unwrap()
                } else {
                    hprintln!("TX not complete").unwrap()
                }
            }

            Err(_err) => {
                hprintln!("Error in lora.check_transmit(). Should return True or False.").unwrap()
            }
        };

        match lora.try_delay_ms(5000u32) {
            Ok(b) => b, // b is ()
            Err(_err) => {
                hprintln!("Error returned from lora.try_delay_ms().").unwrap();
                panic!("should reset in release mode.");
            }
        };
    }
}
