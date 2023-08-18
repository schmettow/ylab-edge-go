#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
use {defmt_rtt as _, panic_probe as _};

/// # YLab Edge
/// 
/// __YLab Edge__ is a sensor recording firmware for the Cytron Maker Pi Pico
/// board.
/// 
/// ## Dependencies
/// 
/// YLab Edge makes use of the Embassy framework, in particular:
/// 
/// + multi-core
/// For running multicore, we need Executor (not just spawner) 
/// and deformat macros (!unwrap)
use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
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
use embassy_rp::gpio::Pin;
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
use ylab::ysns::ads1015_conti as yads0;
use ylab::ysns::ads1115 as yads1;
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
/// the complexity of signalling.
///
/// ## Init
/// 
/// + Initializing peripherals 
/// + spawning tasks
/// + assigning periphs to tasks


use embassy_rp::i2c;
use embassy_rp::peripherals::{I2C0, I2C1};
use embassy_rp::adc;
use embassy_rp::bind_interrupts;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

/// CONFIGURATION
/// 
/// Adc Ads0 Ads1 Lsm6_1
static DEV: [bool; 3] = [true, true, true];
static HZ: [u64; 3] = [800, 120, 30];



#[cortex_m_rt::entry]
fn init() -> ! {
    let p = embassy_rp::init(Default::default());
                        // Activating both I2C controllers
    
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 
            = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner|{

            // Activating the built-in ADC controller
            if DEV[0]{
                let adc0: adc::Adc<'_> 
                    = adc::Adc::new( p.ADC, Irqs, adc::Config::default());
                unwrap!(spawner.spawn(
                    yadc::task( adc0, 
                                p.PIN_26, p.PIN_27, p.PIN_28, p.PIN_29, 
                                HZ[0])));
            };
            if DEV[1]{
                let i2c0: i2c::I2c<'_, I2C0, i2c::Async>
                    = i2c::I2c::new_async(p.I2C0, 
                                    //p.PIN_1, p.PIN_0, 
                                    //p.PIN_5, p.PIN_4, 
                                    p.PIN_9, p.PIN_8,
                                    Irqs,
                                    i2c::Config::default());        
                unwrap!(spawner.spawn(yads0::task(i2c0, HZ[1])));
            }
            if DEV[2]{
                let i2c1: i2c::I2c<'_, I2C1, i2c::Async>
                = i2c::I2c::new_async(p.I2C1, 
                                    p.PIN_3, p.PIN_2, 
                                    Irqs,
                                    i2c::Config::default());
                unwrap!(spawner.spawn(yads1::task(i2c1, HZ[2])));
            }
            /* if DEV[2]{
                let i2c1: i2c::I2c<'_, I2C1, i2c::Blocking>
                    = i2c::I2c::new_blocking(p.I2C1, 
                                            p.PIN_7, p.PIN_6,
                                            //Irqs,
                                            i2c::Config::default());
        
                unwrap!(spawner.spawn(yxz_lsm6::task(i2c1, HZ[2])));
            }*/
             /* if DEV[2]{ 
                let i2c1: i2c::I2c<'_, I2C1, i2c::Blocking>
                    = i2c::I2c::new_blocking(p.I2C1, 
                                            p.PIN_3, p.PIN_2,
                                            //Irqs,
                                            i2c::Config::default());
        
                unwrap!(spawner.spawn(yxz_bmi160::task(i2c1, HZ[2])));
            }*/
        })
    });

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {   
        unwrap!(spawner.spawn(yled::task(p.PIN_25.degrade())));
        //unwrap!(spawner.spawn(ydsp::task(i2c0)));
        unwrap!(spawner.spawn(ybtn::task(p.PIN_20.degrade())));
        unwrap!(spawner.spawn(ybsu::task(p.USB)));
        unwrap!(spawner.spawn(control_task()));
    });
}

/// ## Control task
/// 
/// + capturing button presses
/// + sending commands to the other tasks
/// + giving signals via LED
/// + put text on a display
/// 
/// 

pub use core::sync::atomic::Ordering;

#[embassy_executor::task]
async fn control_task() { 
    let mut state = AppState::New;
    yled::LED.signal(yled::State::Off);
    let disp_text: ydsp::FourLines = [ "YLab".into(), "".into(), "".into(),"".into()];
    ydsp::TEXT.signal(disp_text);
    loop {
        let btn_1 = ybtn::BTN.wait().await;
        let next_state = 
        match (state, btn_1) {
            (AppState::New,     ybtn::Event::Short) => AppState::Ready,
            (AppState::Ready,   ybtn::Event::Short) => AppState::Record, 
            (AppState::Record,  ybtn::Event::Short) => AppState::Ready,
            (_,                 ybtn::Event::Long)  => AppState::New,
            (_, _) => state,};
        

        if next_state != state {
            match next_state {
                AppState::New       => {
                    yled::LED.signal(yled::State::Vibrate);
                    yadc::RECORD.store(false, Ordering::Relaxed);
                    yads0::RECORD.store(false, Ordering::Relaxed);
                    yads1::RECORD.store(false, Ordering::Relaxed);
                    yxz_lsm6::RECORD.store(false, Ordering::Relaxed);
                    yxz_bmi160::RECORD.store(false, Ordering::Relaxed);
                    //yirt::RECORD.store(false, Ordering::Relaxed);
                    //let disp_text: ydsp::FourLines = [ "New".into(), "".into(), "".into(),"".into()];
                    //ydsp::TEXT.signal(disp_text);
                    },
                AppState::Ready     => {
                    yled::LED.signal(yled::State::Blink);
                    yadc::RECORD.store(false, Ordering::Relaxed);
                    yads0::RECORD.store(false, Ordering::Relaxed);
                    yads1::RECORD.store(false, Ordering::Relaxed);
                    yxz_lsm6::RECORD.store(false, Ordering::Relaxed);
                    yxz_bmi160::RECORD.store(false, Ordering::Relaxed);
                    //yirt::RECORD.store(false, Ordering::Relaxed);
                    //let disp_text: ydsp::FourLines = [ "Ready".into(), "".into(), "".into(),"".into()];
                    //ydsp::TEXT.signal(disp_text);
                    },
                AppState::Record    => {
                    yled::LED.signal(yled::State::Steady);
                    yadc::RECORD.store(true, Ordering::Relaxed);
                    yads0::RECORD.store(true, Ordering::Relaxed);
                    yads1::RECORD.store(true, Ordering::Relaxed);
                    yxz_lsm6::RECORD.store(true, Ordering::Relaxed);
                    yxz_bmi160::RECORD.store(true, Ordering::Relaxed);
                    //yirt::RECORD.store(true, Ordering::Relaxed);
                    //let disp_text: ydsp::FourLines = [ "Record".into(), "".into(), "".into(),"".into()];
                    //ydsp::TEXT.signal(disp_text);
                    }
            }
        state = next_state;
        }
    }


}

