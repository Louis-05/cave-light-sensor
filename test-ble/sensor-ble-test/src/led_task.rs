//! One async task per LED. While disabled, the task blocks on a `Signal` — no
//! timer wakeups, so the executor can stay in `wfe` until the BLE writer
//! signals a state change.

use crate::state::LedState;
use embassy_futures::select::{select, Either};
use embassy_nrf::gpio::Output;
use embassy_time::{Duration, Timer};

#[embassy_executor::task(pool_size = 3)]
pub async fn run(mut led: Output<'static>, state: &'static LedState) -> ! {
    loop {
        if !state.enabled() {
            led.set_high(); // active-low: high = OFF
            state.wait_change().await;
            continue;
        }

        let half = Duration::from_millis(state.period_ms() as u64);

        led.set_low(); // ON
        if let Either::Second(_) = select(Timer::after(half), state.wait_change()).await {
            continue; // state changed mid-cycle — restart from top
        }

        led.set_high(); // OFF
        if let Either::Second(_) = select(Timer::after(half), state.wait_change()).await {
            continue;
        }
    }
}
