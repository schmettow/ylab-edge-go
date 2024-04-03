#![no_std]
#![no_main]

/// CONFIGURATION
/// 
/// Adc Tcm
static DEV: (bool, bool, bool) = (true, true, true);
static HZ: (u64, u64, u64) = (0, 211, 0);
static SPEED: u32 = 100_000;
static RUN_DISP: bool = false;
use {defmt_rtt as _, panic_probe as _};


/// The following code initializes the second stack, plus 
/// two heaps
static mut CORE1_STACK: Stack<4096> = Stack::new();
use static_cell::StaticCell;
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

/// +  multi-threading with async
// use embassy_executor::Spawner;
/// + timing using Embassy time 
// use embassy_time::{Duration, Ticker};
/// + peripherals
use ylab::*;
use ysns::moi;
use yuio::led as yled;
use yuio::disp as ydsp;
use ydsp::TEXT as DISP;
use yuii::btn as ybtn;
use ysns::adc as yadc;
use ysns::yco2;
use ytfk::bsu as ybsu;

#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq, )] // testing equality
enum AppState {New, Ready, Record}

use ylab::hal;
use hal::gpio::Pin;
use hal::i2c::{self, Config};
use hal::peripherals::{I2C0, I2C1};
use hal::adc;
use hal::bind_interrupts;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});


use embassy_executor::Executor;
#[allow(unused_imports)]
use hal::adc::{Async, Blocking};
use hal::multicore::{spawn_core1, Stack};
use defmt::*;


#[cortex_m_rt::entry]
fn init() -> ! {
    let p = hal::init(Default::default());
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 
            = EXECUTOR1.init(Executor::new());

        executor1.run(|spawner|{   
            let i2c_contr  = p.I2C0;
            if DEV.2 {
                let mut config = Config::default();
                        config.frequency = SPEED.into();
                        let i2c 
                            = i2c::I2c::new_async(i2c_contr, p.PIN_9, p.PIN_8,
                                        Irqs,
                                        config);
                        unwrap!(spawner.spawn(ylab::ysns::yco2::task(i2c, 2)));
            }
        })
    });



    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {   
        // task for controlling the led
        unwrap!(spawner.spawn(yled::task(p.PIN_25.degrade())));
        // task for receiving text and put it on an OLED 1306
        // Display will use I2C1 on 
        if RUN_DISP{
            let i2c_contr = p.I2C1;
            let i2c 
                = i2c::I2c::new_async(i2c_contr, p.PIN_3, p.PIN_2, Irqs, Config::default());
            unwrap!(spawner.spawn(ydsp::task(i2c)));}
        // task for listening to button presses.
        unwrap!(spawner.spawn(ybtn::task(p.PIN_20.degrade())));
        // task listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::task(p.USB)));
        // task to control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()));
        if DEV.0{
            if DEV.0 {
                spawner.spawn(ylab::ysns::moi::task_2(p.PIN_6, p.PIN_7, 0)).unwrap()
            }
            if DEV.1 {
                let adc0: adc::Adc<'_, Async> 
                    = adc::Adc::new( p.ADC, Irqs, adc::Config::default());
                spawner.spawn(
                    yadc::task( adc0, 
                                p.PIN_26, p.PIN_27, p.PIN_28, p.PIN_29, 
                                HZ.1, 1)).unwrap();
            };
        };
    });
}


pub use core::sync::atomic::Ordering;
static RLX: Ordering = Ordering::Relaxed;

#[embassy_executor::task]
async fn control_task() { 
    let mut state = AppState::Record;
    moi::RECORD.store(true, RLX);
    yadc::RECORD.store(true, RLX);
    yco2::RECORD.store(true, RLX);
    
    yled::LED.signal(yled::State::Steady);
    loop {
        let event = ybtn::BTN.wait().await;
        // Only when a new user event appears,
        // a state transition may occur.
        if let Some(next_state) = 
            match (state, event) {
                (AppState::New,     ybtn::Event::Short) => Some(AppState::Ready),
                (AppState::Ready,   ybtn::Event::Short) => Some(AppState::Record), 
                (AppState::Record,  ybtn::Event::Short) => Some(AppState::Ready),
                (_,                 ybtn::Event::Long)  => Some(AppState::New),
                (_, _) => None,}
                {
                    // When a new event has been announced we do the transition.
                    // This happens by sending the right messages to all our tasks.
                    match next_state {
                        AppState::New => {
                            // Reset all sensors and vibrate
                            yled::LED.signal(yled::State::Vibrate);
                            moi::RECORD.store(false, RLX);
                            yadc::RECORD.store(false, RLX);
                            yco2::RECORD.store(false, RLX);
                            DISP.signal([ Some("New".try_into().unwrap()), None, None, None]);
                            },
                        AppState::Ready     => {
                            // Pause all sensors and blink
                            yled::LED.signal(yled::State::Blink);
                            moi::RECORD.store(false, RLX);
                            yadc::RECORD.store(false, RLX);
                            yco2::RECORD.store(false, RLX);
                            DISP.signal([ Some("Ready".try_into().unwrap()),None, None,None]);
                            },
                        AppState::Record    => {
                            // Transmit sensor data and light up
                            moi::RECORD.store(true, RLX);
                            yled::LED.signal(yled::State::Steady);
                            yadc::RECORD.store(true, RLX);
                            yco2::RECORD.store(true, RLX);
                            DISP.signal([ Some("Record".try_into().unwrap()),None, None,None]);
                            }
                    }
                    state = next_state;
            }
        }


}

