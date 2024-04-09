#![no_std]
#![no_main]

/// CONFIGURATION
/// 
/// Adc Tcm
static DEV: (bool, bool, bool, bool) = (true, true, true, false);
static HZ: (u64, u64, u64, u64) = (0, 103, 113, 0);
static SPEED: u32 = 100_000;
const LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;
const MULTI: bool = true;
use {defmt_rtt as _, panic_probe as _};


use embassy_executor::Executor;
#[allow(unused_imports)]
use hal::gpio::Pin;
use hal::adc::Async;
use hal::multicore::{spawn_core1, Stack};
use defmt::*;

/// The following code initializes the second stack, plus 
/// two heaps
static mut CORE1_STACK: Stack<4096> = Stack::new();
//use log::LevelFilter;
use static_cell::StaticCell;
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

/// +  multi-threading with async
// use embassy_executor::Spawner;
/// + timing using Embassy time 
// use embassy_time::{Duration, Ticker};
/// + peripherals

use ylab::*;
use ylab::ysns::moi;
use ylab::ysns::yxz_lsm6;
use ylab::yuio::led as yled;
use ylab::yuii::btn as ybtn;
use ylab::ysns::adc as yadc;
use ylab::ytfk::bsu as ybsu;

#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq, )] // testing equality
enum AppState {New, Ready, Record}


use ylab::hal;
use hal::i2c::{self, Config};
use hal::peripherals::{I2C0, I2C1};
use hal::adc;
use hal::bind_interrupts;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});



#[cortex_m_rt::entry]
fn init() -> ! {
    // Second core with I2C sensories
    let p = hal::init(Default::default());
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 
            = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner|{
            let i2c0  = p.I2C0;
            if DEV.2 {
                // LSM on Grove 1
                let mut config = Config::default();
                config.frequency = SPEED.into();
                let i2c 
                = i2c::I2c::new_async(i2c0, p.PIN_1, p.PIN_0,
                                        Irqs, config);
                if MULTI {
                    let n: u64 = 3;
                    spawner.spawn(ylab::ysns::yxz_lsm6::multi_task(i2c, n as u8, HZ.2/n, false, 2)).unwrap();
                } else {
                    spawner.spawn(ylab::ysns::yxz_lsm6::task(i2c, HZ.2, 2)).unwrap();
                }
                    
                
            }

        })
    });

    // First core with all IO and built-in sensors
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| { 
        if DEV.0 {
            spawner.spawn(moi::task(p.PIN_21, p.PIN_22, p.PIN_8, p.PIN_9, 0)).unwrap()
        }
        if DEV.1 {
            let adc0: adc::Adc<'_, Async> 
                = adc::Adc::new( p.ADC, Irqs, adc::Config::default());
            spawner.spawn(
                yadc::task( adc0, 
                            p.PIN_26, p.PIN_27, p.PIN_28, p.PIN_29, 
                            HZ.1, 1)).unwrap();
        };

        // task for controlling the led
        unwrap!(spawner.spawn(yled::task(p.PIN_25.degrade())));
        // task for listening to button presses.
        unwrap!(spawner.spawn(ybtn::task(p.PIN_20.degrade())));
        // task listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::logger_task(p.USB, LOG_LEVEL)));
        unwrap!(spawner.spawn(ybsu::task()));
        // task to control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()));
    });
}

#[embassy_executor::task]
async fn control_task() { 
    let mut state = AppState::Record;
    moi::RECORD.store(true, ORD);
    yadc::RECORD.store(true, ORD);
    yxz_lsm6::RECORD.store(true, ORD);
    
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
                            moi::RECORD.store(false, ORD);
                            yadc::RECORD.store(false, ORD);
                            yxz_lsm6::RECORD.store(false, ORD);
                            },
                        AppState::Ready     => {
                            // Pause all sensors and blink
                            yled::LED.signal(yled::State::Blink);
                            yadc::RECORD.store(false, ORD);
                            moi::RECORD.store(false, ORD);
                            yxz_lsm6::RECORD.store(false, ORD);
                            },
                        AppState::Record    => {
                            // Transmit sensor data and light up
                            yled::LED.signal(yled::State::Steady);
                            yadc::RECORD.store(true, ORD);
                            moi::RECORD.store(true, ORD);
                            yxz_lsm6::RECORD.store(true, ORD);
                            }
                    }
                    state = next_state;
            }
        }


}

