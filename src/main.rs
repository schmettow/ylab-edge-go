#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

//use defmt::*;
//use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pin, Pull};
//use embassy_time::queue::TimerQueue;
use embassy_time::{Duration, Timer, Instant, Ticker};
use {defmt_rtt as _, panic_probe as _};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;


// Define states
#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq)] // testing equality
enum AppState {New, Ready, Record}

static STATE: Signal<ThreadModeRawMutex, AppState> = Signal::new();

// LED control
#[embassy_executor::task]

async fn led_task(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::Low);
    led.set_low();
    let state: AppState = STATE.wait().await; // we need at least one signal to kick-start
    let mut ticker = Ticker::every(Duration::from_millis(200));
    loop {
        if STATE.signaled() {
            let state = STATE.wait().await;
            match state {
                AppState::Ready => {led.set_high()},
                AppState::Record => {led.toggle()},
                AppState::New => {led.set_low()},
            };
        };

        if state == AppState::Record {
            ticker.next().await;
            led.toggle()
        }    
    }
}


// Button events
enum BtnEvent {Short, Long}
static BTN_1: Signal<ThreadModeRawMutex, BtnEvent> = Signal::new();

#[embassy_executor::task]
async fn btn_task(btn_pin: AnyPin) {
    let mut btn = Input::new(btn_pin, Pull::Up);
    let debounce = 100;
    let longpress = 1000;


    loop {
        btn.wait_for_low().await;
        let press_time = Instant::now().as_millis();
        Timer::after(Duration::from_millis(debounce)).await;
        
        btn.wait_for_high().await;
        if Instant::now().as_millis() - press_time >= longpress {
            BTN_1.signal(BtnEvent::Long);    
        } else {
            BTN_1.signal(BtnEvent::Short);    
        };
        Timer::after(Duration::from_millis(debounce)).await;
    };
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.spawn(led_task(p.PIN_25.degrade())).unwrap();
    spawner.spawn(btn_task(p.PIN_20.degrade())).unwrap();

    let mut state: AppState = AppState::New;
    STATE.signal(state);
    loop {
        let btn_1 = BTN_1.wait().await;
        let next_state = match (state, btn_1) {
            (AppState::New,    BtnEvent::Short)  => AppState::Ready, 
            (AppState::Ready,  BtnEvent::Short)  => AppState::Record, 
            (AppState::Record,  BtnEvent::Short) => AppState::Ready, 
            (_, BtnEvent::Long)                  => AppState::New,
        };
        STATE.signal(next_state);
        state = next_state;

    }

}
