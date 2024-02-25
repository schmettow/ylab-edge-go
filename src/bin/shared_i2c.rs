#![no_std]
#![no_main]

/// CONFIGURATION
/// 
/// Adc Lsm6 Lsm6 Bmi CO2
static DEV: (bool, bool, bool, bool, bool) = (false, false, true, false, false);
static HZ: (u64, u64, u64, u64, u64) = (1, 7, 6, 5, 1);


use embassy_futures::block_on;
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
/// + thread-safe data transfer and control
///
/// Furthermore, YLab Edge brings its own high-level modules
/// for rapidly developing interactive data collection devices.
/// 
/// + YUI input (LED) and output (button)
use ylab::yuio::led as yled;
use ylab::yuio::disp as ydsp;
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
/// The initial state is set and a signal is send to the LED.
/// The event loop waits for button events (long or short press) 
/// and changes the states, accordingly.
/// If an actual state change has occured, the state is signaled to the UI 
/// and initialized if that is necessary. In this case, entering Record 
/// starts the sensor sampling.
/// 
/// From an architectural point of view, this is a nice setup, too. 
/// Basically, we are separating the very different tasks of 
/// peripherals/spawning and ui handling. It would be easy to just plugin a 
/// different ui, by just reqriting this task. For example, another ui
/// could use the RGB led to signal states, or collect commands from a serial console.
///
/// Conclusion so far: If we take the Embassy promise for granted, that async is zero-cost, 
/// the separation of functionality into different tasks reduces dependencies. It introduces 
/// the complexity of concurreny-safe signalling.
///
/// ## Init
/// 
/// + Initializing peripherals 
/// + spawning tasks
/// + assigning periphs to tasks

use ylab::hal;
use hal::i2c::{self, Config};
use hal::peripherals::{I2C0, I2C1, PIN_0, PIN_1, PIN_4, PIN_5};
use hal::adc;
use hal::bind_interrupts;
bind_interrupts!(struct Irqs {
    //I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});



use ylab::*;


/// Set up I2C bus
///
/// Make a connection between I2C controller and two wires.
///
/// Opposed to a *controller* a *bus* is the connection of a controller with a pair of pins.
/// For this to work with multiple sensors in separate tasks, we use Mutexes throughout.
/// 
/// These Mutexes carry the peripheral as an Option. This is the only way to make them static,
/// which is a concurrency requirement for handing peripherals down to parallel tasks.
/// 

pub enum I2C0Grove {
    One(PIN_1, PIN_0),
    Three(PIN_5, PIN_4),
}

type SharedI2C = Mutex<RawMutex, Option<I2C0>>;
static I2C: SharedI2C = Mutex::new(None);
static SCL_1: Mutex<RawMutex, Option<PIN_1>> = Mutex::new(None);
static SDA_1: Mutex<RawMutex, Option<PIN_0>> = Mutex::new(None);

async fn i2c_bus(i2c: I2C0, scl_1: PIN_1, sda_1: PIN_0)
{
    //DISP.signal([None, Some("0-3-LSM".try_into().unwrap()), None,None]);
    //DISP.signal([ Some("LSM now".try_into().unwrap()), None, None, None]);
    *(I2C.lock().await) = Some(i2c);
    *(SCL_1.lock().await) = Some(scl_1);
    *(SDA_1.lock().await) = Some(sda_1);
    //DISP.signal([ Some("LSM conf".try_into().unwrap()), None, None, None]);
}

/// Init
/// Because the program runs on two cores,
/// the `init` function is the entry point, not `main`.
#[cortex_m_rt::entry]
fn init() -> ! {
    let p = hal::init(Default::default());
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 
            = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner|{
            let _ = block_on(i2c_bus(p.I2C0, p.PIN_1, p.PIN_0));

            unwrap!(spawner.spawn(ylab::ysns::yxz_lsm6::sharing_task(
                &I2C, 
                &SCL_1,
                &SDA_1,
                HZ.2
            )));
        });
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
pub use core::sync::atomic::Ordering;
static RLX: Ordering = Ordering::Relaxed;

#[embassy_executor::task]
async fn control_task() { 
    let mut state = AppState::Record;
    yadc::RECORD.store(true, RLX);
    yxz_lsm6::RECORD.store(true, RLX);
    yxz_bmi160::RECORD.store(true, RLX);
    yco2::RECORD.store(true, RLX);
    yirt_max::RECORD.store(true, RLX);
    //yads0::RECORD.store(true, RLX);
    //yads1::RECORD.store(true, RLX);
    
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

