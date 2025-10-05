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

    pub enum Mode {
        Normal,
        LpRun,
        LpRunSlow,
        Stop
    }

    #[shared]
    struct Shared {
        mode : Mode,
    }

    #[local]
    struct Local {
        dp : stm32l0x0::Peripherals,
        mper : cortex_m::Peripherals
    }

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
            // Read PC13 Input Value
            if !dp.GPIOC.idr.read().id13().is_high() {
                dp.GPIOA.odr.write(|w| w.od5().high());
            } else {
                dp.GPIOA.odr.write(|w| w.od5().low());
            }
        }

        let _test = "test";
        (Shared {
            mode : Mode::Normal
        }, Local { dp , mper : cx.core})
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

    #[task(binds = EXTI4_15)]
    fn exti4_15(cx: exti4_15::Context) {
        // Safe access to local `static mut` variable
       
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

    fn enter_stop_mode(dp: &stm32l0x0::Peripherals,mper : &mut cortex_m::Peripherals) {
        // Set SLEEPDEEP bit of Cortex System Control Register
        mper.SCB.set_sleepdeep();

        dp.PWR.cr.write(|w|{w.pdds().stop_mode().cwuf().clear().ulp().enabled().lpsdsr().low_power_mode().lpds().low_power_mode()});

        dp.RCC.cfgr.write(|w|w.stopwuck().msi());

        while dp.PWR.csr.read().wuf().bit_is_set() {
        }

        // Enter stop mode
        cortex_m::asm::dsb();
    }

    #[idle(shared = [mode],local = [dp,mper])]
    fn idle(mut cx: idle::Context) -> ! {
        info!("idle enter");
        cx.shared.mode.lock(|mode| {
            match mode {
                Mode::Normal => config_clock_normal(cx.local.dp),
                Mode::LpRun => config_clock_lp_run(cx.local.dp),
                Mode::LpRunSlow => config_clock_lp_run_veryslow(cx.local.dp),
                Mode::Stop => enter_stop_mode(cx.local.dp, cx.local.mper),
            }
        });
        
        loop {
            cortex_m::asm::wfi();
            info!("idle loop");
        }
    }
}
