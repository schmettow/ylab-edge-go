//! This example shows how to communicate asynchronous using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
//use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

/* YLAB sensor */

/* ADS1115 Sensor */
mod ads1115 {
    /* Sensor Generics */
    use embassy_time::{Duration, Ticker, Instant};
    
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    
    // I2C    
    use embassy_rp::i2c::{self, Config, InterruptHandler};
    use embassy_rp::peripherals::{PIN_2, PIN_3, I2C1};
    use embassy_rp::bind_interrupts;
    use embedded_ads111x as ads111x;
    use embedded_ads111x::InputMultiplexer::{AIN0GND, AIN1GND, AIN2GND, AIN3GND};
    // ITC
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;
    type Reading = [f32;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<CriticalSectionRawMutex, Measure> = Signal::new();

    #[embassy_executor::task]
    pub async fn task(contr: I2C1, 
                      scl: PIN_3, 
                      sda: PIN_2,
                      hz: u64) {
        // ads1115
        // Init I2C
        bind_interrupts!(struct Irqs {
            I2C1_IRQ => InterruptHandler<I2C1>;
        });        
        let i2c: i2c::I2c<'_, I2C1, i2c::Async> = 
            i2c::I2c::new_async(contr, 
                                scl, sda, 
                                Irqs, 
                                Config::default());
        let config = 
            ads111x::ADS111xConfig::default()
            .dr(ads111x::DataRate::SPS8)
            .pga(ads111x::ProgramableGainAmplifier::V4_096);
        
        let mut ads: ads111x::ADS111x<i2c::I2c<'_, I2C1, i2c::Async>> = 
            ads111x::ADS111x::new(i2c,
                                0x48u8, config).unwrap();
        
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        loop {
            let reading: Reading =
               [ads.read_single_voltage(Some(AIN0GND)).unwrap(),
                ads.read_single_voltage(Some(AIN1GND)).unwrap(),
                ads.read_single_voltage(Some(AIN2GND)).unwrap(),
                ads.read_single_voltage(Some(AIN3GND)).unwrap()];
            let now = Instant::now();
            let result = 
                Measure{time: now, reading: reading};       
            RESULT.signal(result);
            ticker.next().await;
            }
    }
                           
}

/* MAIN */

// I2C
use embassy_rp::peripherals::{PIN_3, PIN_2};
// LED
use embassy_rp::gpio::{Output, Level};
// BSU
use ylab::ytfk::bsu as ybsu;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, 
                                                  Level::Low);
    // I2C
    let i2c_contr = p.I2C1;
    let scl = p.PIN_3;
    let sda = p.PIN_2;
    let hz = 2;
    
    spawner.spawn(ads1115::task(i2c_contr, scl, sda, hz)).unwrap();
    spawner.spawn(ybsu::task(p.USB)).unwrap();
        
    loop {
        let result = ads1115::RESULT.wait().await;
        //let _when = result.time;
        //let _what = result.reading;
        log::info!("{},{},{},{},{}", 
                result.time.as_millis(), 
                result.reading[0],
                result.reading[1],
                result.reading[2],
                result.reading[3]);
        led.toggle();
    }

}
