#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, Instant, Ticker};
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pin, Pull};

// inter-thread communication
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;

// serial logging
/*use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb_logger as logger;*/

/* bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});*/

/* #[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, logger::LevelFilter::Info, driver);
}*/


// Define states
#[derive(Debug,  // used as fmt
    Clone, Copy, // because next_state 
    PartialEq, Eq)] // testing equality
enum AppState {New, Ready, Record}

static STATE: Signal<ThreadModeRawMutex, AppState> = Signal::new();



// LED control
#[embassy_executor::task]

async fn led_task(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::Low);
    led.set_low();
    let state: AppState = STATE.wait().await; // we need at least one signal to kick-start
    let mut ticker = Ticker::every(Duration::from_millis(200));

    loop {
        if STATE.signaled() {
            let state = STATE.wait().await;
            match state {
                AppState::Ready     => {led.set_high()},
                AppState::Record    => {led.toggle()},
                AppState::New       => {led.set_low()},
            };
        };

        if state == AppState::Record {
            ticker.next().await;
            led.toggle();
            ticker.next().await;
            led.toggle();
            ticker.next().await;
            led.toggle();
        }    
    }
}



// Button events
enum BtnEvent {Short, Long}
static BTN_1: Signal<ThreadModeRawMutex, BtnEvent> = Signal::new();

#[embassy_executor::task]
async fn btn_task(btn_pin: AnyPin) {
    let mut btn = Input::new(btn_pin, Pull::Up);
    let debounce = 100;
    let longpress = 1000;


    loop {
        btn.wait_for_low().await;
        let press_time = Instant::now().as_millis();
        Timer::after(Duration::from_millis(debounce)).await;
        
        btn.wait_for_high().await;
        if Instant::now().as_millis() - press_time >= longpress {
            BTN_1.signal(BtnEvent::Long);    
        } else {
            BTN_1.signal(BtnEvent::Short);    
        };
        Timer::after(Duration::from_millis(debounce)).await;
    };
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    //let usb_driver = Driver::new(p.USB, Irqs);
    //spawner.spawn(logger_task(usb_driver)).unwrap();
    spawner.spawn(led_task(p.PIN_25.degrade())).unwrap();
    spawner.spawn(btn_task(p.PIN_20.degrade())).unwrap();

    let mut state: AppState = AppState::New;
    STATE.signal(state);
    //info!("{:?}", STATE);
    loop {
        let btn_1 = BTN_1.wait().await;
        let next_state = match (state, btn_1) {
            (AppState::New,    BtnEvent::Short)  => AppState::Ready, 
            (AppState::Ready,  BtnEvent::Short)  => AppState::Record, 
            (AppState::Record,  BtnEvent::Short) => AppState::Ready, 
            (_, BtnEvent::Long)                  => AppState::New,
        };
      //  logger::info!("{:?}", STATE);
        STATE.signal(next_state);
        state = next_state;

    }

}
