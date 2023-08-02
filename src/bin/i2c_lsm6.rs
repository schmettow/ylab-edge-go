//! This example shows how to communicate asynchronous using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};
use fmt::Debug;

use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::{PIN_14, PIN_15, I2C1};

use embassy_rp::bind_interrupts;
bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});


/* device-specific imports*/
use lsm6ds33 as lsm6;

#[embassy_executor::task]
async fn sensor_task(i2c: I2C1, hz: u64) {
                  }

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_rp::gpio::{Output, Level};
static RESULT: Signal<ThreadModeRawMutex, f32> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, 
                                                  Level::Low);
    let sda: PIN_14 = p.PIN_14;
    let scl: PIN_15 = p.PIN_15;
    let i2c: i2c::I2c<'_, I2C1, i2c::Async> = 
        i2c::I2c::new_async(p.I2C1, 
                            scl, sda, 
                            Irqs, 
                            Config::default());
    let mut sensor = lsm6::Lsm6ds33::new(i2c, Address::default()).unwrap();
    let _measure = sensor.read_gyro();

    //let mut ticker = Ticker::every(Duration::from_hz(10));

    //spawner.spawn(sensor_task(i2c, 5000)).unwrap();
    
    /* loop {
        let measure = sensor.read_gyro();
        // if RESULT.signaled() {
            led.set_high();
            Timer::after(Duration::from_millis(10)).await;
            led.set_low();
        //}
        ticker.next().await;
    } */
}
