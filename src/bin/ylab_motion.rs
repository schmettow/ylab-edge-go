#![no_std]
#![no_main]

/// CONFIGURATION
/// 
/// Adc Tcm
static DEV: (bool, bool) = (true, false);
static HZ: (u64, u64) = (1, 500);

use ylab::{ysns::{yco2, yirt_max}, yuio::disp::TEXT as DISP};
use {defmt_rtt as _, panic_probe as _};

/// # YLab Edge Go
/// 
/// __YLab Edge Go__ is a sensor recording firmware for the Cytron Maker Pi Pico
/// board. It uses Grove connectors for a variety of sensor modules. 
/// 
/// The system runs on both cores of the RP2040. 
/// It use a concurrent, cooperative multi-actor system,
/// with one task per sensor or ui element. 
/// The control flow is kept in a separate task, which receives and sends 
/// messages to the other actors.
/// 
/// 
/// ## Dependencies
/// 
/// YLab Edge makes use of the Embassy framework, in particular:
/// 
/// + multi-core
/// + async routines
/// + Timing
/// + concurrency-safe data containers
/// 
/// For running multicore, we need Executor (not just spawner) 
/// and deformat macros (!unwrap)
use embassy_executor::Executor;
#[allow(unused_imports)]
use hal::adc::{Async, Blocking};
use hal::multicore::{spawn_core1, Stack};
use defmt::*;

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
use hal::gpio::Pin;
use ylab::ysns::yxz_bmi160;
use ylab::ysns::yxz_lsm6;
use ylab::yuio::led as yled;
use ylab::yuio::disp as ydsp;
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
    let p = hal::init(Default::default());
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 
            = EXECUTOR1.init(Executor::new());

        executor1.run(|spawner|{
          
            let i2c_contr  = p.I2C0;
            match DEV {
                (_,true) 
                =>  {   // LSM on Grove 1
                        let i2c 
                        = i2c::I2c::new_async(i2c_contr, p.PIN_1, p.PIN_0,
                                        Irqs,
                                        i2c::Config::default());
                        DISP.signal([None, Some("0-1-LSM".try_into().unwrap()), None,None]);
                        unwrap!(spawner.spawn(ylab::ysns::yxz_lsm6::multi_task(i2c, 200, true)));
                    },
                (_,false) 
                =>  {}
                }

            
        })
    });



    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {   
        // task for controlling the led
        unwrap!(spawner.spawn(yled::task(p.PIN_25.degrade())));
        // task for receiving text and put it on an OLED 1306
        // Display will use I2C1 on 
        let i2c_contr = p.I2C1;
        let i2c 
            = i2c::I2c::new_async(i2c_contr, p.PIN_3, p.PIN_2, Irqs, Config::default());
        unwrap!(spawner.spawn(ydsp::task(i2c)));
        // task for listening to button presses.
        unwrap!(spawner.spawn(ybtn::task(p.PIN_20.degrade())));
        // task listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::task(p.USB)));
        // task to control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()));
        if DEV.0{
            let adc0: adc::Adc<'_, Async> 
                = adc::Adc::new( p.ADC, Irqs, adc::Config::default());
            unwrap!(spawner.spawn(
                yadc::task( adc0, 
                            p.PIN_26, p.PIN_27, p.PIN_28, p.PIN_29, 
                            HZ.0)));
        };
    });
}


pub use core::sync::atomic::Ordering;
static RLX: Ordering = Ordering::Relaxed;

#[embassy_executor::task]
async fn control_task() { 
    let mut state = AppState::Record;
    yadc::RECORD.store(true, RLX);
    yxz_lsm6::RECORD.store(true, RLX);
    
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
                            yadc::RECORD.store(false, RLX);
                            //yads0::RECORD.store(false, RLX);
                            //yads1::RECORD.store(false, RLX);
                            yxz_lsm6::RECORD.store(false, RLX);
                            yxz_bmi160::RECORD.store(false, RLX);
                            yirt_max::RECORD.store(false, RLX);
                            DISP.signal([ Some("New".try_into().unwrap()), None, None, None]);
                            },
                        AppState::Ready     => {
                            // Pause all sensors and blink
                            yled::LED.signal(yled::State::Blink);
                            yadc::RECORD.store(false, RLX);
                            //yads0::RECORD.store(false, RLX);
                            //yads1::RECORD.store(false, RLX);
                            yxz_lsm6::RECORD.store(false, RLX);
                            yxz_bmi160::RECORD.store(false, RLX);
                            yirt_max::RECORD.store(false, RLX);
                            DISP.signal([ Some("Ready".try_into().unwrap()),None, None,None]);
                            },
                        AppState::Record    => {
                            // Transmit sensor data and light up
                            yled::LED.signal(yled::State::Steady);
                            yadc::RECORD.store(true, RLX);
                            //yads0::RECORD.store(true, RLX);
                            //yads1::RECORD.store(true, RLX);
                            yxz_lsm6::RECORD.store(true, RLX);
                            yxz_bmi160::RECORD.store(true, RLX);
                            yco2::RECORD.store(true, RLX);
                            yirt_max::RECORD.store(true, RLX);
                            DISP.signal([ Some("Record".try_into().unwrap()),None, None,None]);
                            }
                    }
                    state = next_state;
            }
        }


}

