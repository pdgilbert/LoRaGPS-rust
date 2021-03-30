//! Serial interface read GPS on usart and transmit with LoRa using crate radio_sx127x (on SPI).
//!  Using  sck, miso, mosi, cs, reset and D00, D01. Not yet using  D02, D03
//!  For pin connections see the setup() sections in src/lora_spi.rs and src/gps_usart.rs.
//! Tested using an RFM95 style radio.

#![no_std]
#![no_main]

#[cfg(debug_assertions)]
use panic_semihosting as _;

#[cfg(not(debug_assertions))]
use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::*;
use embedded_hal::blocking::delay::DelayMs;
use radio::Transmit;

use heapless::{consts, Vec};
use nb::block;

//use embedded_hal::serial::Read;
use old_e_h::serial::Read;

use lora_gps::lora_spi;

#[entry]
fn main() -> ! {
    // set this with
    // SENDER_ID="whatever" cargo build ...
    // or  cargo:rustc-env=SENDER_ID="whatever"
    let id = option_env!("SENDER_ID").expect("").as_bytes();

    //hprintln!("id  {:?} length {:?}", id, id.len()).unwrap(); 

    let (mut lora, _tx_gps, mut rx_gps) = lora_spi::setup(); //  lora (delay is available in lora)

    // byte buffer   Nov 2020 limit data.len() < 255 in radio_sx127x  .start_transmit
    let mut buffer: Vec<u8, consts::U80> = Vec::new();
    let mut buf2: Vec<u8, consts::U80> = Vec::new();

    //hprintln!("buffer at {} of {}", buffer.len(), buffer.capacity()).unwrap();  //0 of 80
    //hprintln!("buf2   at {} of {}",   buf2.len(),   buf2.capacity()).unwrap();  //0 of 80
    buffer.clear();
    buf2.clear();

    //hprintln!("going into write/read loop ^C to exit ...").unwrap();

    let e: u8 = b'x'; // replace char errors with "x"
    let mut good = false; // true while capturing a line

    //let mut size: usize;   // buffer size should not be needed
    //size = buffer.len();   //packet size
    //hprintln!("read buffer {} of {}", size, buffer.capacity()).unwrap();
    hprintln!("entering transmit loop").unwrap();

    loop {
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
            //push byte into buffer and transmit if error/buffer full or end of line. \r is 13, \n is 10            
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
                if ( buffer.len() >45 ) && ( &buffer[0..6] == [36, 71, 80, 82, 77, 67] ) {                    
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
                } else {
                     for v in buffer[..].iter() {
                        buf2.push(*v).unwrap();
                    }
                };

                match lora.start_transmit(&buf2) {
                    Ok(b) => b, // b is ()
                    Err(_err) => {
                        hprintln!("Error returned from lora.start_transmit().").unwrap();
                        panic!("should reset in release mode.");
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
                        panic!("should reset in release mode.");
                    }
                };

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
