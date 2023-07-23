#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, Instant};
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
enum AppState {Ready, Record, New}

static STATE: Signal<ThreadModeRawMutex, AppState> = Signal::new();



// LED control
#[embassy_executor::task]

async fn led_task(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::Low);
    loop {
            match STATE.wait().await {
                AppState::Ready     => {led.set_low();},
                AppState::Record    => {led.set_high();},
                AppState::New       => {/*for _ in 1..5 {
                                            led.toggle();
                                            Timer::after(Duration::from_millis(200)).await;};
                                        led.set_low();*/
                                        led.set_high();},
            };
        };
    }



// Button events
enum BtnEvent {Press, Short, Long}
static BTN_1: Signal<ThreadModeRawMutex, BtnEvent> = Signal::new();

#[embassy_executor::task]
async fn btn_task(btn_pin: AnyPin) {
    let mut btn = Input::new(btn_pin, Pull::Up);
    let debounce = 100;
    //let longpress = 1000;


    loop {
        btn.wait_for_low().await;
        //let press_time = Instant::now().as_millis();
        //Timer::after(Duration::from_millis(debounce)).await;
        BTN_1.signal(BtnEvent::Press); 
        
        /*btn.wait_for_rising_edge().await;
        if Instant::now().as_millis() - press_time >= longpress {
            BTN_1.signal(BtnEvent::Long);    
        } else {
            BTN_1.signal(BtnEvent::Short);    
        };
        Timer::after(Duration::from_millis(debounce)).await;*/
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
            (AppState::Ready,  BtnEvent::Press)  => AppState::Record, 
            (AppState::Record,  BtnEvent::Press) => AppState::Ready,
            (AppState::New,  BtnEvent::Press) => AppState::Ready,
            (_, _) => state,
        };
      //  logger::info!("{:?}", STATE);
        STATE.signal(next_state);
        state = next_state;

    }

}
