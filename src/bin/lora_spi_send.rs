//! Transmit a simple message with LoRa using crate radio_sx127x (on SPI).
//! This example is similar to gps_rw,  lora_spi_gps_rw,  lora_gps_rw and  lora_spi_send.

//https://www.rfwireless-world.com/Tutorials/LoRa-channels-list.html
// channels are as follows
//   'CH_00_900': 903.08, 'CH_01_900': 905.24, 'CH_02_900': 907.40,
//   'CH_03_900': 909.56, 'CH_04_900': 911.72, 'CH_05_900': 913.88,
//   'CH_06_900': 916.04, 'CH_07_900': 918.20, 'CH_08_900': 920.36,
//   'CH_09_900': 922.52, 'CH_10_900': 924.68, 'CH_11_900': 926.84, 'CH_12_900': 915,
//
//   'CH_10_868': 865.20, 'CH_11_868': 865.50, 'CH_12_868': 865.80,
//   'CH_13_868': 866.10, 'CH_14_868': 866.40, 'CH_15_868': 866.70,
//   'CH_16_868': 867   , 'CH_17_868': 868   ,

// See FREQUENCY below to set the channel.

#![no_std]
#![no_main]

#[cfg(debug_assertions)]
use panic_semihosting;

#[cfg(not(debug_assertions))]
use panic_halt;
//use panic_reset;

use cortex_m_rt::entry;
use cortex_m_semihosting::*;
use embedded_hal::blocking::delay::DelayMs;

use radio::Transmit;

use lora_gps::lora_spi;

#[entry]
fn main() -> ! {
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

    let message = b"Hello, LoRa!";

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
