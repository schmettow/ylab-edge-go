use crate::*;
pub mod led {

    // LED control
    use embassy_time::{Duration, Timer};
    use embassy_rp::gpio::{AnyPin, Output, Level};
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;
    pub enum State {Vibrate, Blink, Steady, Interrupt, Off}
    pub static LED: Signal<CriticalSectionRawMutex, State> = Signal::new();
    
    #[embassy_executor::task]
    pub async fn task(led_pin: AnyPin) {
        let mut led 
                = Output::new(led_pin, Level::Low);
        loop {  
                let next_signal = LED.wait().await;
                match  next_signal {
                    State::Vibrate      => {
                        for _ in 1..10 {
                            led.set_high();
                            Timer::after(Duration::from_millis(25))
                            .await;
                            led.set_low();
                            Timer::after(Duration::from_millis(25))
                            .await;
                        };
                    },
                    State::Blink  => {
                        led.set_low();
                        Timer::after(Duration::from_millis(25))
                        .await;
                        led.set_high();
                        Timer::after(Duration::from_millis(50))
                        .await;
                        led.set_low()},
                    State::Steady => {led.set_high()},
                    State::Off    => {led.set_low()},
                    State::Interrupt  => {
                        led.toggle();
                        Timer::after(Duration::from_millis(5))
                        .await;
                        led.toggle();}
                }   
            };
        }
}

pub mod disp {
    use super::*;
    use hal::i2c;
    use hal::peripherals::I2C0; 

    //pub use heapless::String;
    // use itoa;
    /* use embedded_graphics::{ // <--- reactivate graphic output
        pixelcolor::BinaryColor,
        prelude::*,
        image::{Image, ImageRaw},
        text::{Baseline, Text},
        mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    };*/
    use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
    // inter-thread communication
    
    pub type OneLine = String<20>;
    pub type FourLines = [Option<OneLine>; 4];
    
    pub static TEXT: Signal<Mutex, FourLines> 
                = Signal::new();

    // Text display
    use core::fmt::Write;

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C0, i2c::Async>) {
        let interface 
            = I2CDisplayInterface::new(i2c);
        let mut display =
            Ssd1306::new(interface, 
                    DisplaySize128x64, 
                    DisplayRotation::Rotate0)
            .into_terminal_mode();
        match display.init() {
            Err(_) => {},
            Ok(_) => {
                display.init().unwrap();
                let _ = display.write_str("Ydsp");
                        
                loop {
                    let mesg: FourLines = TEXT.wait().await;
                    let _ = display.clear();
                    //let mut str_conv = itoa::Buffer::new(); // conversion to string
                    for row in mesg {
                        match row {
                            Some(text) => {let _ = display.write_str(text.as_str());},
                            None => {let _ = display.write_str("");}
                        }
                        
                    }
                    
                }

            }
        }
    }
}



/*pub mod disp {
    // I2C
    use embassy_rp::i2c::{self};
    use embassy_rp::peripherals::I2C0;
    pub use heapless::String;
    // use itoa;
    /* use embedded_graphics::{ // <--- reactivate graphic output
        pixelcolor::BinaryColor,
        prelude::*,
        image::{Image, ImageRaw},
        text::{Baseline, Text},
        mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    };*/
    use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
    // inter-thread communication
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;

    pub type OneLine = String<20>;
    pub type FourLines = [OneLine; 4];
    
    pub static TEXT: Signal<CriticalSectionRawMutex, FourLines> 
                = Signal::new();

    // Text display
    use core::fmt::Write;

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C0, i2c::Async>) {
        let interface 
            = I2CDisplayInterface::new(i2c);
        let mut display =
            Ssd1306::new(interface, 
                    DisplaySize128x64, 
                    DisplayRotation::Rotate0)
            .into_terminal_mode();
        display.init().unwrap();
        let _ = display.write_str("YLab");
                
        loop {
            let mesg: FourLines = TEXT.wait().await;
            let _ = display.clear();
            //let mut str_conv = itoa::Buffer::new(); // conversion to string
            for row in mesg {
                let _ = display.write_str(row.as_str());
            }
            
        }
    }
}*/
