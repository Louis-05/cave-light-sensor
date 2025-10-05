#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_abort as _;

mod cfg;
mod light_sensor;
mod rtc;
mod storage;
mod tasks;

use light_sensor::*;
use opt300x::Opt300x;
use rtic::app;
use rtic_monotonics::systick::*;
use stm32l0xx_hal::{
    exti::{DirectLine, Exti, ExtiLine, GpioLine, TriggerEdge},
    gpio::{gpiob::*, OpenDrain, Output, PushPull, *},
    i2c::I2c,
    pac::I2C1,
    prelude::*,
    pwr::PWR,
    rcc::Config,
    rtc::Rtc,
    syscfg::SYSCFG,
};

const GPIO_LINE: u8 = 0;

#[app(device = stm32l0xx_hal::pac, peripherals = true)]
mod app {
    use link_lib::Link;
    use stm32l0xx_hal::{pac::LPUART1, serial::Serial};

    use self::storage::event_storage::EventStorage;
    use super::*;
    use crate::tasks::{light_interrupt::LocalLightInterruptState, uart_interrupt::LocalUartState};

    #[shared]
    struct Shared {
        speedy: bool,
        storage: EventStorage<
            shared_bus::I2cProxy<
                'static,
                cortex_m::interrupt::Mutex<
                    core::cell::RefCell<I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>>>,
                >,
            >,
        >,
        rtc: Rtc,
    }

    #[local]
    struct Local {
        led: PB3<Output<PushPull>>,
        state: bool,
        light_state: LocalLightInterruptState,
        uart_state: LocalUartState,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Setup clocks
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());

        let pwr = PWR::new(cx.device.PWR, &mut rcc);
        let lse = rcc.enable_lse(&pwr);

        // Initialize the systick interrupt & obtain the token to prove that we did
        let systick_mono_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 16_000_000, systick_mono_token); // default STM32F303 clock-rate is 36MHz
                                                                      //rtt_init_print!();
                                                                      // Setup LED
        let gpiob = cx.device.GPIOB.split(&mut rcc);
        let gpioa = cx.device.GPIOA.split(&mut rcc);

        let uart_tx_pin = gpioa.pa1;
        let uart_rx_pin = gpioa.pa3;

        let mut serial = cx
            .device
            .LPUART1
            .usart(
                uart_tx_pin,
                uart_rx_pin,
                stm32l0xx_hal::serial::Config::default(),
                &mut rcc,
            )
            .unwrap();
        serial.use_lse(&mut rcc, &lse);

        //let link: Link<Serial<LPUART1>, (), (), 32>::new(serial);

        let sda = gpiob.pb7.into_open_drain_output();
        let scl = gpiob.pb6.into_open_drain_output();

        let i2c = cx.device.I2C1.i2c(
            sda,
            scl,
            embedded_time::rate::units::Hertz(100_000),
            &mut rcc,
        );

        let mut shared_i2c  = shared_bus::new_cortexm!(I2c<I2C1, PB7<Output<OpenDrain>>, PB6<Output<OpenDrain>>> = i2c).unwrap();
        let storage = EventStorage::new(shared_i2c.acquire_i2c());

        let sensor = Opt300x::new_opt3001(
            shared_i2c.acquire_i2c(),
            opt300x::SlaveAddr::Alternative(false, false),
        );
        let mut sensor = sensor.into_continuous().ok().unwrap();

        setup_sensor(&mut sensor);

        let rtc = Rtc::new(cx.device.RTC, &mut rcc, &pwr, None).unwrap();

        let interrupt_pin = gpiob.pb0.into_floating_input();
        let mut syscfg = SYSCFG::new(cx.device.SYSCFG, &mut rcc);
        let mut exti = Exti::new(cx.device.EXTI);

        exti.listen_gpio(
            &mut syscfg,
            interrupt_pin.port(),
            GpioLine::from_raw_line(interrupt_pin.pin_number()).unwrap(),
            TriggerEdge::Falling,
        );

        exti.listen_direct(DirectLine::Lpuart1);

        let mut led = gpiob.pb3.into_push_pull_output();
        led.set_high().unwrap();

        // Schedule the blinking task
        blink::spawn().ok();

        (
            Shared {
                speedy: false,
                storage,
                rtc,
            },
            Local {
                light_state: LocalLightInterruptState {
                    interrupt_pin: interrupt_pin,
                    sensor: sensor,
                },

                led,
                state: false,
            },
        )
    }

    #[task(local = [led, state], shared = [speedy])]
    async fn blink(mut ctx: blink::Context) {
        loop {
            if *ctx.local.state {
                ctx.local.led.set_high().unwrap();
                *ctx.local.state = false;
            } else {
                ctx.local.led.set_low().unwrap();
                *ctx.local.state = true;
            }
            let delay: u32 = ctx
                .shared
                .speedy
                .lock(|speedy| if *speedy { 100 } else { 1000 });

            Systick::delay(delay.millis()).await;
        }
    }

    #[task(binds = EXTI0_1, local = [light_state], shared = [speedy,storage,rtc])]
    fn exti0_1(ctx: exti0_1::Context) {}

    #[task(binds = AES_RNG_LPUART1, local = [uart_state], shared = [rtc])]
    fn uart0(ctx: uart0::Context) {}
}
