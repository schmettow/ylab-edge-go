pub mod btn {
    use embassy_time::{Duration, Timer, Instant};
    use embassy_rp::gpio::{AnyPin, Input, Pull};
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;
    pub enum Event {Press, Short, Long}
    pub static BTN: Signal<CriticalSectionRawMutex, Event> = Signal::new();


    #[embassy_executor::task]
    pub async fn task(btn_pin: AnyPin) {
        let mut btn = Input::new(btn_pin, Pull::Up);
        let longpress = 1000;
        let debounce = 50;

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