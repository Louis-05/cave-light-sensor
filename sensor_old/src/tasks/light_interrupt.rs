use common_data::event::Event;
use rtic::Mutex;
use rtt_target::debug_rprintln;
use stm32l0xx_hal::{
    exti::{Exti, ExtiLine, GpioLine},
    gpio::{gpiob::PB0, Floating, Input},
};

use crate::{
    app::exti0_1,
    cfg::MIN_OFF_TIME,
    light_sensor::{self, *},
    GPIO_LINE,
};

pub struct LocalLightInterruptState {
    pub interrupt_pin: PB0<Input<Floating>>,
    pub sensor: light_sensor::MyOpt3001,
}

pub fn light_interrupt(mut ctx: exti0_1::Context) {
    if Exti::is_pending(GpioLine::from_raw_line(GPIO_LINE).unwrap()) {
        Exti::unpend(GpioLine::from_raw_line(GPIO_LINE).unwrap());
        debug_rprintln!("Interrupt");

        let state = ctx.local.light_state;
        let now_timestamp = ctx.shared.rtc.lock(|r| r.now().timestamp());

        // let mut light_detected: bool = false;
        let status = state.sensor.read_status().unwrap();
        if status.was_too_high {
            wait_for_dark(&mut state.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
            //light_detected = true;
            debug_rprintln!("High event {}", now_timestamp);

            let event = Event::High(now_timestamp);
            ctx.shared.storage.lock(|storage| {
                storage.add_or_overwrite_event(event, MIN_OFF_TIME).unwrap();
            });
        } else if status.was_too_low {
            wait_for_light(&mut state.sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
            //light_detected = false;
            debug_rprintln!("Low event {}", now_timestamp);

            let event = Event::Low(now_timestamp);
            ctx.shared.storage.lock(|storage| {
                // We never overwrite with a low event so it's not necessary to call overwrite to do
                // the check
                storage.add_new_event(event).unwrap();
            });
        }
    }
}
