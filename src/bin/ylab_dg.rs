#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use {defmt_rtt as _, panic_probe as _};
//use cortex_m::prelude::_embedded_hal_digital_InputPin;
use embassy_executor::Spawner;
//use embassy_executor::{Spawner, Executor};
use embassy_time::{Duration, Ticker, Instant, Timer};
use embassy_rp::gpio::{AnyPin, Input, Pull, Pin};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;


// LED control
mod ylab_led {
    use embassy_time::{Duration, Timer};
    use embassy_rp::gpio::{AnyPin, Output, Level};
    use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
    use embassy_sync::signal::Signal;
    pub enum State {Vibrate, Blink, Steady, Off}
    pub static LED: Signal<ThreadModeRawMutex, State> = Signal::new();
    
    #[embassy_executor::task]
    pub async fn led_task(led_pin: AnyPin) {
        
        let mut led = 
            Output::new(led_pin, 
        Level::Low);
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

// BUTTON

enum BtnEvent {Press, Short, Long}
static BTN_1: Signal<ThreadModeRawMutex, BtnEvent> = Signal::new();

#[embassy_executor::task]
async fn btn_task(btn_pin: AnyPin) {
    let mut btn = Input::new(btn_pin, Pull::Up);
    let longpress = 1000;
    let debounce = 10;


    loop {
        btn.wait_for_low().await;
        BTN_1.signal(BtnEvent::Press);
        let when_pressed = Instant::now().as_millis();
        Timer::after(Duration::from_millis(debounce)).await;
        btn.wait_for_high().await;
        if Instant::now().as_millis() - when_pressed >= longpress {
            BTN_1.signal(BtnEvent::Long);    
        } else {
            BTN_1.signal(BtnEvent::Short);    
        };
        Timer::after(Duration::from_millis(longpress)).await;
    };
}


/* ADC  */

//use embedded_hal::adc::{Channel, OneShot};
use embassy_rp::adc::{Adc, Config, InterruptHandler};
use embassy_rp::bind_interrupts;
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

use embassy_rp::peripherals::{PIN_26, PIN_27, PIN_28, PIN_29};
static RESULT: Signal<ThreadModeRawMutex, u16> = Signal::new();
//type AdcPin: embedded_hal::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;

#[embassy_executor::task]
async fn adc_task(mut adc: Adc<'static>, 
                  mut adc_0: PIN_26,
                  mut adc_1: PIN_27,
                  mut adc_2: PIN_28,
                  mut adc_3: PIN_29,
                  hz: u64) {
    let mut ticker = Ticker::every(Duration::from_hz(hz));
    loop {
        // let mut adc_pin = p.PIN_27;
        let res = adc.read(&mut adc_0).await;
        RESULT.signal(res);
        let res = adc.read(&mut adc_1).await;
        RESULT.signal(res);
        let res = adc.read(&mut adc_2).await;
        RESULT.signal(res);
        let res = adc.read(&mut adc_3).await;
        RESULT.signal(res);
        ticker.next().await;
         }
                           }

#[embassy_executor::task]
async fn fake_task(hz: u64) {
    let mut ticker = Ticker::every(Duration::from_hz(hz));
    loop {
        let result: u16 = 42;
        RESULT.signal(result);
        ticker.next().await;
    };
}

/* MAIN */

/// 
// Define states
#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq, )] // testing equality
enum AppState {New, Ready, Record}
static STATE: Signal<ThreadModeRawMutex, AppState> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    /* Peripherals */
    let p = embassy_rp::init(Default::default());
    /* ADC */
    let adc: Adc<'_> = Adc::new(p.ADC, Irqs, Config::default());
    /* multi-tasking */ 
    spawner.spawn(ylab_led::led_task(p.PIN_25.degrade())).unwrap();
    spawner.spawn(btn_task(p.PIN_20.degrade())).unwrap();
    spawner.spawn(fake_task(1)).unwrap();
    spawner.spawn(adc_task(adc, p.PIN_26, 
                                p.PIN_27, 
                                p.PIN_28, 
                                p.PIN_29, 5000)).unwrap();
    /* state machine */
    let mut state = AppState::New;
    ylab_led::LED.signal(ylab_led::State::Off);
    STATE.signal(state);


    loop {
        let btn_1 = BTN_1.wait().await;
        let next_state = 
            match (state, btn_1) {
                (AppState::New,     BtnEvent::Short) => AppState::Ready,
                (AppState::Ready,   BtnEvent::Short) => AppState::Record, 
                (AppState::Record,  BtnEvent::Short) => AppState::Ready,
                (_,                 BtnEvent::Long)  => AppState::New,
                (_, _) => state,
            };

        if state != next_state {
            match next_state {
                AppState::New       => {ylab_led::LED.signal(ylab_led::State::Vibrate)},
                AppState::Ready     => {ylab_led::LED.signal(ylab_led::State::Blink)},
                AppState::Record    => {ylab_led::LED.signal(ylab_led::State::Steady)}
            }
            STATE.signal(next_state);
            state = next_state;
        }
    }

}
