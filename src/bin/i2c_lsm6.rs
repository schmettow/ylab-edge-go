//! This example shows how to communicate asynchronous using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;

use embassy_time::{Duration, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});



use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::{PIN_14, PIN_15, I2C1};
use embedded_ads111x as ads111x;
use embedded_ads111x::InputMultiplexer::{AIN0GND, AIN1GND, AIN2GND, AIN3GND};

#[embassy_executor::task]
async fn ads_task(i2c: i2c::I2c<'static, I2C1, i2c::Async>,
                  hz: u64) {
    let config = 
        ads111x::ADS111xConfig::default()
        .dr(ads111x::DataRate::SPS8)
        .pga(ads111x::ProgramableGainAmplifier::V4_096);
    
    let mut ads: ads111x::ADS111x<i2c::I2c<'_, I2C1, i2c::Async>> = 
        ads111x::ADS111x::new(i2c,
                              0x48u8, config).unwrap();
    
    let mut ticker = Ticker::every(Duration::from_hz(hz));
    loop {
        let _reading = ads.read_single_voltage(Some(AIN0GND)).unwrap();
        RESULT.signal(_reading);
        let _reading = ads.read_single_voltage(Some(AIN1GND)).unwrap();
        RESULT.signal(_reading);
        let _reading = ads.read_single_voltage(Some(AIN2GND)).unwrap();
        RESULT.signal(_reading);
        let _reading = ads.read_single_voltage(Some(AIN3GND)).unwrap();
        RESULT.signal(_reading);
        ticker.next().await;
    }
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
    let mut ticker = Ticker::every(Duration::from_hz(10));

    spawner.spawn(ads_task(i2c, 5000)).unwrap();

    loop {
        if RESULT.signaled() {
            led.set_high();
            Timer::after(Duration::from_millis(10)).await;
            led.set_low();
        }

    }

}
