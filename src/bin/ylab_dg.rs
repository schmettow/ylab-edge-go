#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use {defmt_rtt as _, panic_probe as _};
//use cortex_m::prelude::_embedded_hal_digital_InputPin;
use embassy_executor::{Spawner, Executor};
use embassy_rp::peripherals::PIN_26;
use embassy_time::{Duration, Timer, Ticker, Instant, block_for};
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pull, Pin};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;


// LED control
#[embassy_executor::task]

async fn led_task(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::Low);
    loop {
            match STATE.wait().await {
                AppState::Ready     => {led.set_low();},
                AppState::Record    => {led.set_high();},
                AppState::New       => {for _ in 1..20 {
                                            led.toggle();
                                            block_for(Duration::from_millis(25));};
                                            led.set_low()},
            };
        };
    }



// BUTTON
enum BtnEvent {Press, Short, Long}
static BTN_1: Signal<ThreadModeRawMutex, BtnEvent> = Signal::new();

#[embassy_executor::task]
async fn btn_task(btn_pin: AnyPin) {
    let mut btn = Input::new(btn_pin, Pull::Up);
    let longpress = 1000;
    let debounce = 10;


    loop {
        btn.wait_for_low().await;
        BTN_1.signal(BtnEvent::Press);
        let when_pressed = Instant::now().as_millis();
        block_for(Duration::from_millis(debounce));
        btn.wait_for_high().await;
        if Instant::now().as_millis() - when_pressed >= longpress {
            BTN_1.signal(BtnEvent::Long);    
        } else {
            BTN_1.signal(BtnEvent::Short);    
        };
        block_for(Duration::from_millis(debounce));
    };
}

/* use embassy_rp::adc::{Adc, Config, InterruptHandler};
use embassy_rp::bind_interrupts;
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});
*/
static RESULT: Signal<ThreadModeRawMutex, u64> = Signal::new();

#[embassy_executor::task]
async fn fake_task() {
    let spm = 100;
    let mut ticker = Ticker::every(Duration::from_hz(spm));
    loop {
        let level = Instant::now().as_millis();
        RESULT.signal(level);
        ticker.next().await;
    };
}


///// MAIN
// Define states
#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq)] // testing equality
enum AppState {New, Ready, Record}
static STATE: Signal<ThreadModeRawMutex, AppState> = Signal::new();


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    //let usb_driver = Driver::new(p.USB, Irqs);
    
    spawner.spawn(led_task(p.PIN_25.degrade())).unwrap();
    spawner.spawn(btn_task(p.PIN_20.degrade())).unwrap();
    spawner.spawn(fake_task()).unwrap();

    let mut state = AppState::New;
    STATE.signal(state);

    //info!("{:?}", STATE);
    loop {
        let btn_1 = BTN_1.wait().await;
        let next_state = 
            match (state, btn_1) {
                (AppState::New,  BtnEvent::Short)    => AppState::Ready,
                (AppState::Ready,  BtnEvent::Short)  => AppState::Record, 
                (AppState::Record,  BtnEvent::Short) => AppState::Ready,
                (_,  BtnEvent::Long)                 => AppState::New,
                (_, _) => state,
            };
        //  logger::info!("{:?}", STATE);
        if state != next_state {
            STATE.signal(next_state);
            state = next_state;
        }
    }

}
