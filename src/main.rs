#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

//use defmt::*;
//use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pin, Pull};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

// Define states
#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq)] // testing equality
enum AppState {Steady, Blink, Off}

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;

static STATE: Signal<ThreadModeRawMutex, AppState> = Signal::new();
//static STATE: AtomicU32 = AtomicU32::new(AppState::Off);


// Button events
enum BtnState {Press, Release}

static BTN_1: Signal<ThreadModeRawMutex, BtnState> = Signal::new();

#[embassy_executor::task]

async fn led_task(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::Low);
    led.set_low();
    loop {
        let state = STATE.wait().await;
        match state {
            AppState::Steady => {led.set_high()},
            AppState::Blink => {loop {Timer::after(Duration::from_millis(200)).await;
                                led.toggle()}},
            AppState::Off => {led.set_low()},
        };        
    }
}


#[embassy_executor::task]

async fn btn_task(btn_pin: AnyPin) {
    let mut btn = Input::new(btn_pin, Pull::Up);
    let debounce = 100;
    loop {
        btn.wait_for_low().await;
        BTN_1.signal(BtnState::Press);
        Timer::after(Duration::from_millis(debounce)).await;
        btn.wait_for_high().await;
        BTN_1.signal(BtnState::Release);
        Timer::after(Duration::from_millis(debounce)).await;
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.spawn(led_task(p.PIN_25.degrade())).unwrap();
    spawner.spawn(btn_task(p.PIN_20.degrade())).unwrap();

    let mut state: AppState = AppState::Off;
    STATE.signal(state);
    loop {
        let btn_1 = BTN_1.wait().await;
        let next_state = match (state, btn_1) {
            (AppState::Off,    BtnState::Press) => AppState::Blink, 
            (AppState::Blink,  BtnState::Press) => AppState::Steady, 
            (AppState::Steady, BtnState::Press) => AppState::Off,
            (_, _) => state,
        };
        STATE.signal(next_state);
        state = next_state;

    }

}
