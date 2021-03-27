#![no_std]

#[cfg(debug_assertions)]
use panic_semihosting;

#[cfg(not(debug_assertions))]
use panic_halt;

pub mod gps_usart;
pub mod lora_spi;
pub mod lora_spi_rcv;
