
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
    // I2C
    use embassy_rp::i2c::{self, Config, InterruptHandler};
    use embassy_rp::peripherals::{PIN_4, PIN_5, I2C0};
    use embassy_rp::bind_interrupts;
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
    pub async fn task(contr: I2C0, sda: PIN_4, scl: PIN_5) {
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
}
