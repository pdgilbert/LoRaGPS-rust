
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

#[cfg(debug_assertions)]
use panic_semihosting;

#[cfg(not(debug_assertions))]
use panic_halt;
//use panic_reset;

//use core::convert::Infallible;

//use cortex_m_rt::entry;
//use cortex_m_semihosting::*;
//use nb::block;

//use heapless::{consts, Vec};


pub mod lora_spi;
pub mod lora_spi_rcv;
pub mod gps_usart;
