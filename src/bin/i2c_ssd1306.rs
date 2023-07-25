//! This example shows how to communicate asynchronous using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Duration, Ticker};

use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::{PIN_4, PIN_5, I2C0};
use embassy_rp::bind_interrupts;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

use itoa;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    //image::{Image, ImageRaw},
    text::{Baseline, Text},
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

//use core::fmt::Write;
//use heapless::String;
//fn make_mesg(mesg: i32) -> String {
//    let mut data = String::<32>::new(mesg);
//    return data;
    //let _ = write!(data, "data:{mesg}");
    //data
//}

#[embassy_executor::task]
async fn disp_task(i2c: i2c::I2c<'static, I2C0, i2c::Async>) {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = 
        Ssd1306::new(interface, 
                    DisplaySize128x64, 
                    DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
        display.init().unwrap();
    //let mut ticker = Ticker::every(Duration::from_hz(hz));
    let text_style = MonoTextStyleBuilder::new()
    .font(&FONT_6X10)
    .text_color(BinaryColor::On)
    .build();
    
    loop {
        Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

        Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
        
        let mesg: i32 = MESG.wait().await;
        let mut str_conv = itoa::Buffer::new(); // conversion to string
        display.flush().unwrap();

        Text::with_baseline(str_conv.format(mesg), Point::new(0, 32), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    }
                           }
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;

static MESG: Signal<ThreadModeRawMutex, i32> 
    = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let sda: PIN_4 = p.PIN_4;
    let scl: PIN_5 = p.PIN_5;
    let i2c: i2c::I2c<'_, I2C0, i2c::Async> = 
        i2c::I2c::new_async(p.I2C0, 
                            scl, sda, 
                            Irqs, 
                            Config::default());
    spawner.spawn(disp_task(i2c)).unwrap();
    let mut ticker = Ticker::every(Duration::from_hz(1));
    let mut counter = 0;
    MESG.signal(counter);
    loop {
        counter = counter + 1;
        MESG.signal(counter);
        ticker.next().await;
    }

}
