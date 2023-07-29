#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

pub mod yui {
    pub mod led {
        // LED control
        use embassy_time::{Duration, Timer};
        use embassy_rp::gpio::{AnyPin, Output, Level};
        use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
        use embassy_sync::signal::Signal;
        pub enum State {Vibrate, Blink, Steady, Off}
        pub static LED: Signal<ThreadModeRawMutex, State> = Signal::new();
        
        #[embassy_executor::task]
        pub async fn task(led_pin: AnyPin) {
            let mut led = 
                Output::new(led_pin, 
            Level::Low);
            loop {
                    match LED.wait().await {
                        State::Vibrate      => {
                            for _ in 1..10 {
                                led.set_high();
                                Timer::after(Duration::from_millis(25))
                                .await;
                                led.set_low();
                                Timer::after(Duration::from_millis(25))
                                .await;
                            };
                        },
                        State::Blink  => {
                            led.set_low();
                            Timer::after(Duration::from_millis(25))
                            .await;
                            led.set_high();
                            Timer::after(Duration::from_millis(50))
                            .await;
                            led.set_low()},
                        State::Steady => {
                            led.set_high()},
                        State::Off    => {
                            led.set_low()
                        },
                    }   
                };
            }
    }

    pub mod btn {
        use embassy_time::{Duration, Timer, Instant};
        use embassy_rp::gpio::{AnyPin, Input, Pull};
        use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
        use embassy_sync::signal::Signal;
        pub enum Event {Press, Short, Long}
        pub static BTN: Signal<ThreadModeRawMutex, Event> = Signal::new();

        #[embassy_executor::task]
        pub async fn task(btn_pin: AnyPin) {
            let mut btn = Input::new(btn_pin, Pull::Up);
            let longpress = 1000;
            let debounce = 10;

            loop {
                btn.wait_for_low().await;
                BTN.signal(Event::Press);
                let when_pressed = Instant::now().as_millis();
                Timer::after(Duration::from_millis(debounce)).await;
                btn.wait_for_high().await;
                if Instant::now().as_millis() - when_pressed >= longpress {
                    BTN.signal(Event::Long);    
                } else {
                    BTN.signal(Event::Short);    
                };
                Timer::after(Duration::from_millis(longpress)).await;
            };
        }
    }
}



pub mod ysense {
    pub mod adc {
        use embassy_rp::adc::Adc;
        // use embassy_rp::adc::{Adc, InterruptHandler};
        // use embassy_rp::bind_interrupts;
        /*bind_interrupts!(struct Irqs {
            ADC_IRQ_FIFO => InterruptHandler;
        });*/
        use embassy_time::{Duration, Ticker};
        use embassy_rp::peripherals::{PIN_26, PIN_27, PIN_28, PIN_29};
        use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
        use embassy_sync::signal::Signal;
        pub static RESULT: Signal<ThreadModeRawMutex, u16> = Signal::new();
        //type AdcPin: embedded_hal::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;

        #[embassy_executor::task]
        pub async fn task(mut adc: Adc<'static>, 
                        mut adc_0: PIN_26,
                        mut adc_1: PIN_27,
                        mut adc_2: PIN_28,
                        mut adc_3: PIN_29,
                        hz: u64) {
            let mut ticker = Ticker::every(Duration::from_hz(hz));
            loop {
                // let mut adc_pin = p.PIN_27;
                let res = adc.read(&mut adc_0)
                .await;
                RESULT.signal(res);
                let res = adc.read(&mut adc_1)
                .await;
                RESULT.signal(res);
                let res = adc.read(&mut adc_2)
                .await;
                RESULT.signal(res);
                let res = adc.read(&mut adc_3)
                .await;
                RESULT.signal(res);
                ticker.next()
                .await;
                }
        }
    }
}