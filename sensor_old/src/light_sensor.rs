use opt300x::{ic::Opt3001, FaultCount, Opt300x};
use stm32l0xx_hal::{
    gpio::{gpiob::*, OpenDrain, Output},
    i2c::I2c,
    pac::I2C1,
};

pub type MyOpt3001 = Opt300x<
    shared_bus::I2cProxy<
        'static,
        cortex_m::interrupt::Mutex<
            core::cell::RefCell<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>>,
        >,
    >,
    Opt3001,
    opt300x::mode::Continuous,
>;

const MANTISSA_MAX: u16 = 0xFFF;
const MANTISSA_MIN: u16 = 0;
const EXPONENT_MAX: u8 = 0b1011;
const EXPONENT_MIN: u8 = 0;
pub const EXPONENT_THRESHOLD: u8 = 0b1001u8;
pub const MANTISSA_THRESHOLD: u16 = 488u16;

pub fn setup_sensor(sensor: &mut MyOpt3001) {
    // Set the comparison mode to Lacthed window instead of Hysterersis
    sensor
        .set_comparison_mode(opt300x::ComparisonMode::LatchedWindow)
        .unwrap();

    // We use a pullup resistor so LOW is active
    sensor
        .set_interrupt_pin_polarity(opt300x::InterruptPinPolarity::Low)
        .unwrap();

    // Number of "positive" read that are necessary to activate an interrupt
    sensor.set_fault_count(FaultCount::One).unwrap();

    sensor
        .set_integration_time(opt300x::IntegrationTime::Ms800)
        .unwrap();

    //Reset flag and interrupt in case the MCU had a reset when while the sensor
    // had an interrupt
    let _ = sensor.read_status();

    wait_for_light(sensor, MANTISSA_THRESHOLD, EXPONENT_THRESHOLD);
}

pub fn wait_for_dark(sensor: &mut MyOpt3001, mantissa: u16, exponent: u8) {
    sensor
        .set_high_limit_raw(EXPONENT_MAX, MANTISSA_MAX)
        .unwrap();
    sensor.set_low_limit_raw(exponent, mantissa).unwrap();
}

pub fn wait_for_light(sensor: &mut MyOpt3001, mantissa: u16, exponent: u8) {
    sensor.set_high_limit_raw(exponent, mantissa).unwrap();
    sensor
        .set_low_limit_raw(EXPONENT_MIN, MANTISSA_MIN)
        .unwrap();
}
