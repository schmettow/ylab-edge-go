//! This example test the RP Pico on RGB led.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod ylab_rgb {
    use embassy_time::{Duration, Timer};
    use embassy_rp::gpio::{AnyPin, Output, Level};
    use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
    use embassy_sync::signal::Signal;
    pub enum State {Vibrate, Blink, Steady, Off}
    pub static LED: Signal<ThreadModeRawMutex, State> = Signal::new();
    
    #[embassy_executor::task]
    pub async fn rgb_task() {
        use smart_leds::{brightness, SmartLedsWrite, RGB8};
        use ws2812_pio::{Ws2812Direct};
        type RgbStatusLed = Ws2812Direct<pac::PIO0, 
                        bsp::hal::pio::SM0, 
                        bsp::hal::gpio::pin::bank0::Gpio28>;
        loop {
                match LED.wait().await {
                    State::Vibrate      => {
                        for _ in 1..10 {
                            led.set_high();
                            Timer::after(Duration::from_millis(25)).await;
                            led.set_low();
                            Timer::after(Duration::from_millis(25)).await;
                        };
                    },
                    State::Blink        => {
                        led.set_low();
                        Timer::after(Duration::from_millis(25)).await;
                        led.set_high();
                        Timer::after(Duration::from_millis(50)).await;
                        led.set_low()},
                    State::Steady       => {
                        led.set_high()},
                    State::Off  => {
                        led.set_low()
                    },
                    }   
                };
            }
}



use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after(Duration::from_secs(1)).await;

        info!("led off!");
        led.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}
