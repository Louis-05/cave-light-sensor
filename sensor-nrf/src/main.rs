#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    // Red user LED on Xiao nRF52840 — active-low, so High = off.
    let mut led = Output::new(p.P0_26, Level::High, OutputDrive::Standard);

    loop {
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
    }
}
