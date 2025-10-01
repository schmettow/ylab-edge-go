//! Display support
//!
//! Example written for the common SSD1306 Oled display

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Duration, Ticker};

/* DISPLAY */
mod ylab_display {
    // I2C
    use embassy_rp::i2c::{self, Config, InterruptHandler};
    use embassy_rp::peripherals::{PIN_4, PIN_5, I2C0};
    use embassy_rp::bind_interrupts;
    use itoa;
    /* use embedded_graphics::{ // <--- reactivate graphic output
        pixelcolor::BinaryColor,
        prelude::*,
        image::{Image, ImageRaw},
        text::{Baseline, Text},
        mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    };*/
    use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
    // inter-thread communication
    use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
    use embassy_sync::signal::Signal;
    pub static MESG: Signal<ThreadModeRawMutex, i32> 
        = Signal::new();



    // Text display
    use core::fmt::Write;

    #[embassy_executor::task]
    pub async fn disp_text(contr: I2C0, sda: PIN_4, scl: PIN_5) {
        // Init I2C display
        bind_interrupts!(struct Irqs {
            I2C0_IRQ => InterruptHandler<I2C0>;
        });        
        let i2c: i2c::I2c<'_, I2C0, i2c::Async> = 
            i2c::I2c::new_async(contr, 
                                scl, sda, 
                                Irqs, 
                                Config::default());
        let interface = I2CDisplayInterface::new(i2c);
        let mut display =
            Ssd1306::new(interface, 
                    DisplaySize128x64, 
                    DisplayRotation::Rotate0)
            .into_terminal_mode();
        display.init().unwrap();
                
        loop {
            let mesg: i32 = MESG.wait().await;
            let _ = display.clear();
            let mut str_conv = itoa::Buffer::new(); // conversion to string
            let _ = display.write_str(str_conv.format(mesg));
        }
    }
}

/* 
    // Graphics display
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
            let mesg: i32 = MESG.wait().await;
            let mut str_conv = itoa::Buffer::new(); // conversion to string

            let _ = display.clear(BinaryColor::Off);
            Text::with_baseline("Hello world!", 
                Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

            Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
            
            Text::with_baseline(str_conv.format(mesg), Point::new(0, 32), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
            
            display.flush().unwrap();
        }
    }*/

/* MAIN */


use embassy_rp::peripherals::{PIN_4, PIN_5};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Setting up I2C0 on pins 4 and 5 (Grove 3)
    let p = embassy_rp::init(Default::default());
    let sda: PIN_4 = p.PIN_4;
    let scl: PIN_5 = p.PIN_5;
    // Kicking off the display task
    spawner.spawn(ylab_display::disp_text(p.I2C0, sda, scl)).unwrap();
    // main loop sends a message every 1 sec
    let mut ticker = Ticker::every(Duration::from_hz(1));
    let mut counter = 0;
    ylab_display::MESG.signal(counter);
    loop {
        counter = counter + 1;
        ylab_display::MESG.signal(counter);
        ticker.next().await;
    }

}
