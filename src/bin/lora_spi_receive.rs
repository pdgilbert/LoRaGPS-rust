//! Receive message with LoRa using crate radio_sx127x (on SPI).
//! See also examples gps_rw,  lora_spi_gps_rw,  lora_spi_send.

//   Using  sck, miso, mosi, cs, reset and D00, D01. Not yet using  D02, D03
//   See setup() sections below for pins.

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

// trait needs to be in scope to find  methods start_transmit and check_transmit.
use radio::Receive;

use radio_sx127x::device::{PacketInfo};

use lora_gps::lora_spi_rcv;

fn to_str(x: &[u8]) -> &str {
    match core::str::from_utf8(x) {
        Ok(str) => &str,
        Err(_error) => "problem converting u8 to str ",
    }
}

#[entry]
fn main() -> ! {
    let mut lora = lora_spi_rcv::setup(); //delay is available in lora.delay_ms()

    lora.start_receive().unwrap(); // should handle error

    let mut buff = [0u8; 1024];
    let mut n: usize;
    let mut info = PacketInfo::default();

    loop {
        let poll = lora.check_receive(false);
        // false (the restart option) specifies whether transient timeout or CRC errors should be
        // internally handled (returning Ok(false) or passed back to the caller as errors.

        match poll {
            Ok(v) if v => {
                n = lora.get_received(&mut info, &mut buff).unwrap();
                //hprintln!("RX complete ({:?}, length: {})", info, n).unwrap();
                //hprintln!("{:?}", &buff[..n]).unwrap();
                // for some reason the next prints twice?
                hprintln!("{}", to_str(&buff[..n])).unwrap()
            }

            Ok(_v) => (), // hprint!(".").unwrap(),   // print "." if nothing received

            Err(err) => hprintln!("poll error {:?} ", err).unwrap(),
        };

        match lora.try_delay_ms(100u32) {
            Ok(b) => b, // b is ()
            Err(_err) => {
                hprintln!("Error returned from lora.try_delay_ms().").unwrap();
                panic!("should reset in release mode.");
            }
        };
    }
}
