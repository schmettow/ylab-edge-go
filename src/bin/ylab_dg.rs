#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

/// __YLab Edge__ is a sensor recording firmware for the Cytron Maker Pi Pico
/// board.
/// # Dependencies
/// 
/// YLab Edge makes use of the Embassy framework, in particular:
/// 
/// +  multi-threading with async
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
/// + timing using Embassy time 
// use embassy_time::{Duration, Ticker};
/// + peripherals
use embassy_rp::gpio::Pin;
/// + thread-safe types
//use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
//use embassy_sync::signal::Signal;

/// Furthermore, YLab Edge brings its own high-level modules
/// for rapidly developing interactive data collection devices.
/// 
/// + YUI input (LED) and output (button)
use ylab::yuio::led as yled;
use ylab::yuio::disp as ydsp;
use ylab::yuii::btn as ybtn;
/// + sensors (built-in ADC)
/// + data transport/storage
use ylab::ysns::adc as yadc;
use ylab::ytfk::bsu as ybsu;

/// The storage task currently connects one sensor two the output channel. 
/// It lives as its own task, because signals have no means of polling. 
/// When data is received the LED briefly interrupts.
#[embassy_executor::task]
async fn storage_task() { 
    loop {
        let result = yadc::RESULT.wait().await;
        yled::LED.signal(yled::State::Interrupt);
        log::info!("{},{}", result.time, result.reading.0);
    };
}





///# Main Program

/// The main program orchestrates the input/output, sensors and storage 
/// using a state machine. 
/// 
/// The allowed states are defines by the enum AppState
/// Down the line we have to make copies of the AppState. This is why 
/// a couple of traits need to be derived for AppState.

#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq, )] // testing equality
enum AppState {New, Ready, Record}

/// A possible variation of this setup is to isolate interaction model
/// into its own task. If Embassy promise to be zero-cost even on massive multi-threading,
/// this would make programming very easy. 

/// The main task starts by collecting the peripherals, 
/// before they are moved to the individual tasks which are spanwed here. We start with the storage task, 
/// which just passes on sensor data, when they occur.
/// 
/// Then, the initial state is set and a signal is send to the LED.
/// The event loop waits for button events (long or short press) 
/// and changes the states, accordingly.
/// If an actual state change has occured, the state is signaled to the UI 
/// and initialized if that is necessary. In this case, entering Record 
/// starts the sensor sampling.

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    
    let p = embassy_rp::init(Default::default());
    spawner.spawn(yled::task(p.PIN_25.degrade())).unwrap();
    spawner.spawn(ydsp::task(p.I2C0, p.PIN_4, p.PIN_5)).unwrap();
    spawner.spawn(ybtn::task(p.PIN_20.degrade())).unwrap();
    spawner.spawn(yadc::task(p.ADC, p.PIN_26, 
                                p.PIN_27, 
                                p.PIN_28, 
                                p.PIN_29, 
                                10)).unwrap();
    spawner.spawn(ybsu::task(p.USB)).unwrap();
    
    let mut state = AppState::New;
    yled::LED.signal(yled::State::Off);

    loop {
        let btn_1 = ybtn::BTN.wait().await;
        let next_state = 
        match (state, btn_1) {
            (AppState::New,     ybtn::Event::Short) => AppState::Ready,
            (AppState::Ready,   ybtn::Event::Short) => AppState::Record, 
            (AppState::Record,  ybtn::Event::Short) => AppState::Ready,
            (_,                 ybtn::Event::Long)  => AppState::New,
            (_, _) => state,};
        

        if state != next_state {
            match next_state {
                AppState::New       => 
                    {yled::LED.signal(yled::State::Vibrate)},
                AppState::Ready     => 
                    {yled::LED.signal(yled::State::Blink);
                    yadc::CONTROL.signal(yadc::State::Ready)},
                AppState::Record    => 
                    {yled::LED.signal(yled::State::Steady);
                    yadc::CONTROL.signal(yadc::State::Record);
                    ydsp::MESG.signal(42);
                    spawner.spawn(storage_task()).unwrap();}
                }
        //STATE.signal(next_state);
        state = next_state;
        }
        /* if let result = yadc::RESULT.wait().await {
            log::info!("{:?}", result);
        };*/
    }

}
