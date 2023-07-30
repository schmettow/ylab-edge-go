#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use embassy_rp::gpio::{Pin};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;

// YUI
use ylab::yui::led as yled;
use ylab::yui::btn as ybtn;
// Ysense
use ylab::ysense::adc as yadc;

#[embassy_executor::task]
async fn fake_task(hz: u64) {
    static RESULT: Signal<ThreadModeRawMutex, u16> = Signal::new();
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
    /* multi-tasking */ 
    spawner.spawn(yled::task(p.PIN_25.degrade())).unwrap();
    spawner.spawn(ybtn::task(p.PIN_20.degrade())).unwrap();
    spawner.spawn(yadc::task(p.ADC, p.PIN_26, 
                                p.PIN_27, 
                                p.PIN_28, 
                                p.PIN_29, 5000)).unwrap();
    /* state machine */
    let mut state = AppState::New;
    yled::LED.signal(yled::State::Off);
    STATE.signal(state);


    loop {
        let btn_1 = ybtn::BTN.wait().await;
        let next_state = 
            match (state, btn_1) {
                (AppState::New,     ybtn::Event::Short) => AppState::Ready,
                (AppState::Ready,   ybtn::Event::Short) => AppState::Record, 
                (AppState::Record,  ybtn::Event::Short) => AppState::Ready,
                (_,                 ybtn::Event::Long)  => AppState::New,
                (_, _) => state,
            };

        if state != next_state {
            match next_state {
                AppState::New       => {yled::LED.signal(yled::State::Vibrate)},
                AppState::Ready     => {yled::LED.signal(yled::State::Blink)},
                AppState::Record    => {yled::LED.signal(yled::State::Steady);
                                        let _result = yadc::RESULT.wait()
                                            .await;}
            }
            STATE.signal(next_state);
            state = next_state;
        }
    }

}
