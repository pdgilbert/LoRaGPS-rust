//! Receive message with LoRa using crate radio_sx127x (on SPI) and print on semihost.
//!  Using  sck, miso, mosi, cs, reset and D00, D01. Not yet using  D02, D03
//!  For pin connections see the setup() sections in src/lora_spi_gps_usart.rs.
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

// trait needs to be in scope to find  methods start_transmit and check_transmit.
use radio::Receive;

use radio_sx127x::device::PacketInfo;

use lora_gps::lora_spi_gps_usart::setup;

fn to_str(x: &[u8]) -> &str {
    match core::str::from_utf8(x) {
        Ok(str) => &str,
        Err(_error) => "problem converting u8 to str ",
    }
}

#[entry]
fn main() -> ! {
    let (mut lora, _rx, _tx) = setup(); //delay is available in lora.delay_ms()

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
