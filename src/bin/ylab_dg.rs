#![no_std]
#![no_main]

/// CONFIGURATION
/// 
/// Adc Lsm6 Lsm6 Bmi 
static DEV: (bool, bool) = (true, true);
static HZ: (u64, u64) = (0, 419);

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
use ylab::*;
use ylab::ysns::moi;
/// + thread-safe data transfer and control
///
/// Furthermore, YLab Edge brings its own high-level modules
/// for rapidly developing interactive data collection devices.
/// 
/// + YUI input (LED) and output (button)
use ylab::yuio::led as yled;
use ylab::yuii::btn as ybtn;
/// + four built-in ADC sensors
use ylab::ysns::adc as yadc;
/// + four ADCs on a ADS1115;
// use ylab::ysns::ads1015_conti as yads0;
// use ylab::ysns::ads1115 as yads1;
/// + accel sensor
// use ylab::ysns::yxz_bmi160;
/// + IR tempereture
// use ylab::ysns::yirt;
/// + data transport/storage
use ylab::ytfk::bsu as ybsu;

/// ## Storage task
/// 
/// The storage task currently employs the usb-logger module of Embassy.

/// ## UI task
/// 
/// The ui task collects events, e.g. button presses, 
/// updates the output (LED, display) and controls the
/// recording task using an atomic lock.
/// 
/// The ui orchestrates the input/output, sensors and storage 
/// using a state machine. 
/// The allowed states are defines by the enum AppState
/// Down the line we have to make copies of the AppState. This is why 
/// a couple of traits need to be derived for AppState.

#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq, )] // testing equality
enum AppState {New, Ready, Record}

/// In a usual multi-threaded app you would write the interaction model
/// in the main task. However, with dual-core the main task is no longer 
/// async. Since all communication channels are static, this really doesn't matter.
/// 
/// ## Init
/// 
/// + Initializing peripherals 
/// + spawning tasks
/// + assigning periphs to tasks

use ylab::hal;
use hal::adc;
use hal::bind_interrupts;
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

/// Because the program runs on two cores,
/// the `init` function is the entry point, not `main`.

#[cortex_m_rt::entry]
fn init() -> ! {
    // Getting hold of the peripherals, 
    // like pins, ADC, and I2C controllers.
    let p = hal::init(Default::default());
    // Spawning a process on the second core
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        // The second core has its own executor, which is 
        // is the Embassy mechanism to handle concurrency.
        let executor1 
            = EXECUTOR1.init(Executor::new());
        // Here we start spawning our actors as separate tasks. 
        // The DEV vector simply is a static register 
        // to easily switch on and off components at dev time.

        executor1.run(|spawner|{
            if DEV.0 {
                spawner.spawn(ylab::ysns::moi::task(p.PIN_21, p.PIN_22, p.PIN_8, p.PIN_9, 0)).unwrap()
                }
            if DEV.1 {
                let adc0: adc::Adc<'_, Async> 
                    = adc::Adc::new( p.ADC, Irqs, adc::Config::default());
                spawner.spawn(
                    yadc::task( adc0, 
                                p.PIN_26, p.PIN_27, p.PIN_28,  
                                HZ.1, 1)).unwrap();
                };
            })
        });


    // Initializing the Embassy executor on the first core.
    // Starting all tasks that are respnsible for:
    // + ui events
    // + control flow
    // + sending data
    

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {   
        // task for controlling the led
        unwrap!(spawner.spawn(yled::task(p.PIN_25.degrade())));
        // task for listening to button presses.
        unwrap!(spawner.spawn(ybtn::task(p.PIN_20.degrade())));
        // task listening for data packeges to send up the line (reverse USB ;)
        unwrap!(spawner.spawn(ybsu::logger_task(p.USB, log::LevelFilter::Info)));
        unwrap!(spawner.spawn(ybsu::task()));
        // task to control sensors, storage and ui
        unwrap!(spawner.spawn(control_task()))
    });
}

/// ## Control task
/// 
/// + capturing user input
/// + controlling user output
/// + controlling storage
/// + put text on a display
/// 
/// 
/// 

/// To be parallel-safe, we are using boolean atomics.
/// Atomics are capsules around basic data types that 
/// efficiently handle concurrent or parallel access.

#[embassy_executor::task]
async fn control_task() { 
    let mut state = AppState::Record;
    moi::RECORD.store(true, ORD);
    yadc::RECORD.store(true, ORD);
    
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
                            },
                        AppState::Ready     => {
                            // Pause all sensors and blink
                            yled::LED.signal(yled::State::Blink);
                            moi::RECORD.store(false, ORD);
                            yadc::RECORD.store(false, ORD);                        
                            },
                        AppState::Record    => {
                            // Transmit sensor data and light up
                            yled::LED.signal(yled::State::Steady);
                            moi::RECORD.store(true, ORD);
                            yadc::RECORD.store(true, ORD);                        
                            }
                    }
                    state = next_state;
            }
        }


}

