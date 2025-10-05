#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

#[rtic::app(device = stm32l0::stm32l0x0, peripherals = false,dispatchers = [SPI1])]
mod app {
    use cortex_m::asm::nop;
    use defmt::info;
    use stm32l0::stm32l0x0;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        let dp = stm32l0x0::Peripherals::take().unwrap();
       

        //Enable clock for Port A and C
        dp.RCC
            .iopenr
            .write(|w| w.iopaen().set_bit().iopcen().set_bit());

        //PORT A PIN 5 set as output + push pull
        dp.GPIOA.moder.write(|w| w.mode5().output());
        dp.GPIOA.otyper.write(|w| w.ot5().push_pull());

        //PORT C PIN 13 set as input + floating, not 100% necessary since this is the default
        // config
        dp.GPIOC.moder.write(|w| w.mode13().input());
        dp.GPIOC.pupdr.write(|w| w.pupd13().floating());

        loop {
            for _i in 0..5 {
                dp.GPIOA.odr.write(|w| w.od5().high());
                sleep_ms(&mut cx.core, 500, 2_100_000);
                dp.GPIOA.odr.write(|w| w.od5().low());
                sleep_ms(&mut cx.core, 500, 2_100_000);
            }
            config_clock_lp_run(&dp);
            for _i in 0..5 {
                dp.GPIOA.odr.write(|w| w.od5().high());
                sleep_ms(&mut cx.core, 500, 131_072);
                dp.GPIOA.odr.write(|w| w.od5().low());
                sleep_ms(&mut cx.core, 500, 131_072);
            }
            config_clock_lp_run_veryslow(&dp);
            for _i in 0..5 {
                dp.GPIOA.odr.write(|w| w.od5().high());
                sleep_ms(&mut cx.core, 500, 65_536);
                dp.GPIOA.odr.write(|w| w.od5().low());
                sleep_ms(&mut cx.core, 500, 65_536);
            }
            exit_lp_run(&dp);
        }

        loop {
            // Read PC13 Input Value
            if !dp.GPIOC.idr.read().id13().is_high() {
                dp.GPIOA.odr.write(|w| w.od5().high());
            } else {
                dp.GPIOA.odr.write(|w| w.od5().low());
            }
        }

        let _test = "test";
        (Shared {}, Local {})
    }

    fn config_clock_lp_run(dp: &stm32l0x0::Peripherals) {
        // enable MSI
        dp.RCC.cr.write(|w| w.msion().set_bit());

        while dp.RCC.cr.read().msirdy().is_not_ready() {}

        // set MSI clock frequency , Max for LP RUN is range 1 ( 131_072 Hz )
        dp.RCC.icscr.write(|w| w.msirange().range1());

        // select MSI as system clock
        dp.RCC.cfgr.write(|w| w.sw().msi());

        // set bits for low power run
        dp.PWR.cr.write(|w| w.lprun().set_bit());
        dp.PWR.cr.write(|w| w.lpsdsr().set_bit());
    }

    fn config_clock_lp_run_veryslow(dp: &stm32l0x0::Peripherals) {
        // enable MSI
        dp.RCC.cr.write(|w| w.msion().set_bit());

        while dp.RCC.cr.read().msirdy().is_not_ready() {}

        // set MSI clock frequency , Max for LP RUN is range 1 ( 65_536 Hz )
        dp.RCC.icscr.write(|w| w.msirange().range0());

        // select MSI as system clock
        dp.RCC.cfgr.write(|w| w.sw().msi());

        // set bits for low power run
        dp.PWR.cr.write(|w| w.lprun().set_bit());
        dp.PWR.cr.write(|w| w.lpsdsr().set_bit());
    }

    fn exit_lp_run(dp: &stm32l0x0::Peripherals) {
        // enable MSI
        dp.PWR.cr.write(|w| w.lprun().clear_bit());
        dp.PWR.cr.write(|w| w.lpsdsr().clear_bit());

        config_clock_normal(dp);
    }

    fn config_clock_normal(dp: &stm32l0x0::Peripherals) {
        // enable MSI
        dp.RCC.cr.write(|w| w.msion().set_bit());

        while dp.RCC.cr.read().msirdy().is_not_ready() {}

        // set MSI clock frequency , Max for LP RUN is range 1 ( 131 kHz )
        dp.RCC.icscr.write(|w| w.msirange().range5());

        // select MSI as system clock
        dp.RCC.cfgr.write(|w| w.sw().msi());

    }



    fn sleep_ms(mper : &mut cortex_m::Peripherals,ms : u64,clock_hz : u64) {
        
        let total_wait_ticks = (ms * clock_hz)/1000 ;
        //let wraps = total_wait_ticks / 

        mper.SYST.set_reload(total_wait_ticks as u32);
        mper.SYST.clear_current();

        mper.SYST.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
        mper.SYST.enable_counter();
        while !mper.SYST.has_wrapped() {
            cortex_m::asm::nop();
        }
    }

    /*fn enter_stop_mode() {
        // Set SLEEPDEEP bit of Cortex System Control Register
        let mut scb = unsafe { cortex_m::Peripherals::steal().SCB };
        //cortex_m::Peripherals::take().unwrap().SCB;
        scb.set_sleepdeep();

        // Enable PWR peripheral clock
        let rcc = embassy_stm32::pac::RCC;

        //rcc.apb1enr().modify(|w| w.set_pwren(true));

        rcc.cfgr().modify(|w| {
            w.set_stopwuck(embassy_stm32::pac::rcc::vals::Stopwuck::MSI);
        });

        // Configure stop mode
        let pwr = embassy_stm32::pac::PWR;
        pwr.cr().modify(|w| {
            w.set_ulp(true);
            w.set_cwuf(true);
            w.set_pdds(embassy_stm32::pac::pwr::vals::Pdds::STOP_MODE); // Power-down deepsleep
            w.set_lpds(embassy_stm32::pac::pwr::vals::Mode::LOW_POWER_MODE); // Low-power deepsleep
            w.set_lpsdsr(embassy_stm32::pac::pwr::vals::Mode::LOW_POWER_MODE);
        });

        //pwr.csr().read().wuf()
        while pwr.csr().read().wuf() {}

        // Enter stop mode
        cortex_m::asm::dsb();
        cortex_m::asm::wfi();
    }*/

    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            nop();
            //enter_stop_mode();
        }
    }
}
