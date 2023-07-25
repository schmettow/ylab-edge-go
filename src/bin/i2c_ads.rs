//! This example shows how to communicate asynchronous using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C1;
use embassy_time::{Duration, Ticker};
//use embedded_hal_async::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

use embedded_ads111x as ads111x;


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let sda = p.PIN_14;
    let scl = p.PIN_15;

    let i2c = i2c::I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());
    
    let config = ads111x::ADS111xConfig::default()
        .mux(ads111x::InputMultiplexer::AIN0GND)
        .dr(ads111x::DataRate::SPS8)
        .pga(ads111x::ProgramableGainAmplifier::V4_096);


    let mut adc = 
        ads111x::ADS111x::new(i2c, 0x48u8, config).unwrap();
    
    let mut ticker = Ticker::every(Duration::from_hz(500));

    loop {
        let _reading = adc.read_single_voltage(None).unwrap();
        ticker.next().await;
    }

}
