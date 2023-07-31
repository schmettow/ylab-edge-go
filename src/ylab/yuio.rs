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