#![feature(type_alias_impl_trait)]
#![no_std]
#![no_main]

use teensy4_panic as _;

#[rtic::app(device = teensy4_bsp)]
mod app {
    use bsp::board;
    use teensy4_bsp as bsp;

    const FPS: u32 = 30;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: board::Led,
        pit: bsp::hal::pit::Pit<2>,
        poller: bsp::logging::Poller,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        cx.core.SCB.set_sleepdeep();

        let board::Resources {
            pins,
            mut gpio2,
            pit: (_, _, mut pit, _),
            usb,
            ..
        } = board::t40(cx.device);

        let led = board::led(&mut gpio2, pins.p13);
        pit.set_interrupt_enable(true);
        pit.set_load_timer_value(board::PERCLK_FREQUENCY / FPS);
        pit.enable();

        let poller = bsp::logging::log::usbd(usb, bsp::logging::Interrupts::Enabled).unwrap();

        (Shared {}, Local { led, pit, poller })
    }
    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            rtic::export::wfi()
        }
    }

    #[task(binds = PIT, local = [led, pit, ctr: u32 = 0])]
    fn blink_and_log(cx: blink_and_log::Context) {
        let pit = cx.local.pit;
        let led = cx.local.led;
        *cx.local.ctr += 1;

        led.toggle();
        while pit.is_elapsed() {
            pit.clear_elapsed();
        }

        log::info!("This would be frame #{}", cx.local.ctr);
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn poll_logger(cx: poll_logger::Context) {
        cx.local.poller.poll();
    }
}
