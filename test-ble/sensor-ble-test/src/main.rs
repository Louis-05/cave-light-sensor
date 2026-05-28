#![no_std]
#![no_main]

mod ble;
mod led_task;
mod state;

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::interrupt::Priority;
use embassy_time::{Duration, Instant, Timer};
use nrf_softdevice::Softdevice;
use panic_halt as _;

const CHASE_STEP:  Duration = Duration::from_millis(150);
const CHASE_TOTAL: Duration = Duration::from_secs(15);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // The SoftDevice S140 reserves interrupt priorities P0, P1, and P4 for
    // its own use. Anything embassy-nrf claims at startup must run at P2 or
    // lower (numerically larger) — leaving these at the embassy default (P0)
    // makes `sd_softdevice_enable()` return SDM_INCORRECT_INTERRUPT_CONFIG
    // and `Softdevice::enable()` panics; with `panic-halt` the device hangs
    // silently right after the boot chase, with no BLE advertising visible.
    let mut nrf_cfg = embassy_nrf::config::Config::default();
    nrf_cfg.gpiote_interrupt_priority = Priority::P2;
    nrf_cfg.time_interrupt_priority   = Priority::P2;
    let p = embassy_nrf::init(nrf_cfg);

    // User LEDs on the Xiao nRF52840 — all active-low (High = OFF).
    let mut red   = Output::new(p.P0_26, Level::High, OutputDrive::Standard);
    let mut green = Output::new(p.P0_30, Level::High, OutputDrive::Standard);
    let mut blue  = Output::new(p.P0_06, Level::High, OutputDrive::Standard);

    // 15-second startup chase — runs before the SoftDevice is enabled so we
    // have direct, exclusive ownership of the pins. embassy_time uses RTC1
    // and the SD uses RTC0, so there is no conflict when we enable it later.
    let start = Instant::now();
    let mut phase: u8 = 0;
    while start.elapsed() < CHASE_TOTAL {
        red.set_high();
        green.set_high();
        blue.set_high();
        match phase {
            0 => red.set_low(),
            1 => green.set_low(),
            _ => blue.set_low(),
        }
        phase = (phase + 1) % 3;
        Timer::after(CHASE_STEP).await;
    }
    red.set_high();
    green.set_high();
    blue.set_high();

    // Bring up the SoftDevice and the BLE GATT server.
    let sd = Softdevice::enable(&ble::sd_config());
    let server = ble::Server::new(sd).unwrap();
    spawner.spawn(ble::softdevice_task(sd).unwrap());

    // Hand off LED ownership to per-LED tasks. From this point each LED
    // is driven entirely from its corresponding `LedState`, which the BLE
    // event handler mutates on every characteristic write.
    spawner.spawn(led_task::run(red,   &state::RED).unwrap());
    spawner.spawn(led_task::run(green, &state::GREEN).unwrap());
    spawner.spawn(led_task::run(blue,  &state::BLUE).unwrap());

    // Advertising / connection / disconnect loop never returns.
    ble::run(sd, server).await
}
