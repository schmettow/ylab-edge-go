//! # YLab Edge
//!
//! Records Sensor Data
//!
//! Uses the `ws2812_pio` driver to control the LEDs, which in turns uses the
//! RP2040's PIO block.
//! 
//! 

#![no_std]
#![no_main]

//use std::path::Prefix;

use cortex_m_rt::entry;
use panic_halt as _;

//use smart_leds::RGB;
// Make an alias for our board support package so copying examples to other boards is easier
use ylab_edge as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{FunctionPio0, Pin},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use rp2040_hal::pio::PIOExt;
use ws2812_pio::{Ws2812Direct};
//use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;

//use debounced_button::ButtonState::*;

mod yui{    
    use ylab_edge as bsp;
    use bsp::hal::{
        clocks::{init_clocks_and_plls, Clock},
        gpio::{FunctionPio0, Pin},
        pac,
        sio::Sio,
        watchdog::Watchdog,
    };
    use rp2040_hal::{pio::PIOExt};
    use smart_leds::{brightness, SmartLedsWrite, RGB8};
    use ws2812_pio::{Ws2812Direct};
    use embedded_hal::digital::v2::InputPin;
    use embedded_hal::digital::v2::OutputPin;

    
    type RgbStatusLed = Ws2812Direct<pac::PIO0, 
                        bsp::hal::pio::SM0, 
                        bsp::hal::gpio::pin::bank0::Gpio28>;

    pub trait StaticOutput {
        fn write<T>(&mut self, value: T);
    }
                    
    pub struct RgbLed {
        led: RgbStatusLed,
    }

    impl RgbLed {
        pub fn new(led: RgbStatusLed) -> Self {
            RgbLed{led: led}
            }
        
        pub fn write(&mut self, value:RGB8){
            let col = [value,];
            self.led.write(brightness(col.iter().cloned(), 32)).unwrap();
        }

        pub fn red(&mut self){
            self.write((255, 20, 0).into());
        }

        pub fn green(&mut self){
            self.write((0, 255, 20).into());
        }

        pub fn blue(&mut self){
            self.write((0, 20, 255).into());
        }

        pub fn white(&mut self){
            self.write((255, 255, 255).into());
        }

        pub fn off(&mut self){
            self.write((0, 0, 0).into());
        }

    }

    // Stateful button
    pub struct Button<T: InputPin> {
        pub pin: T,
        pub last_state: bool,
        pub state: bool,
    }

    impl<T: InputPin> Button<T> {
        pub fn new(pin: T) -> Self {
            Button { pin: pin, last_state: false, state: false}
        }

        pub fn update(&mut self) -> bool {
            let this_value = self.read();
            self.last_state = self.state;
            self.state = this_value;
            return self.state
        }

    }

    // Contact sensor trait
    pub trait ContactSensor {
        fn read(&self) -> bool;
    }

    impl<T: InputPin> ContactSensor for Button<T> {
        fn read(&self) -> bool {
            self.pin.is_low().unwrap_or(true)
        }
    }

}

// User events and Interaction
// 
use debounced_button::Button as DeButton;
use debounced_button::ButtonState::*;

#[derive(Debug,  // used as fmt
         Clone, Copy, // because next_state 
         PartialEq, Eq)] // testing equality

// Define states
enum State {Init, New, Ready, Record, Send}



#[entry]
fn main() -> ! {
    // Configure the RP2040 peripherals

    let mut peri: pac::Peripherals = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(peri.WATCHDOG);

    let clocks: rp2040_hal::clocks::ClocksManager = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        peri.XOSC,
        peri.CLOCKS,
        peri.PLL_SYS,
        peri.PLL_USB,
        &mut peri.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let sio = Sio::new(peri.SIO);
    let pins = bsp::Pins::new(
        peri.IO_BANK0,
        peri.PADS_BANK0,
        sio.gpio_bank0,
        &mut peri.RESETS,
    );

    // INIT RGB
    let smartleds_pin: Pin<_, FunctionPio0> 
        = pins.smartleds.into_mode();
    // Configure the addressable LED
    let (mut pio, sm0, _, _, _) 
        = peri.PIO0.split(&mut peri.RESETS);
    let mut rgb
        = yui::RgbLed::new(Ws2812Direct::new(smartleds_pin,
                            &mut pio,
                            sm0,
                            clocks.peripheral_clock.freq()));
    
    // Init Button
    let mut btn_1 = 
        yui::Button::new(pins.button1.into_pull_up_input());    
    
    // Init Led
    let mut led = pins.led.into_push_pull_output();

    // Init ADC
    use embedded_hal::adc::OneShot;
    use rp2040_hal::{adc::Adc};
    
    //let core = pac::CorePeripherals::take().unwrap();
    //let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    

    // Prepare interaction flow
    //let mut state = "Stop";
    let mut trial: i8 = 0;
    let n_trials: i8 = 3;
    let mut adc = Adc::new(peri.ADC, &mut peri.RESETS);
    let mut adc_pin_0 = pins.grove_6_b.into_floating_input();
    let mut this_value: u16 = adc.read(&mut adc_pin_0).unwrap();
    

    // Button 2 and interaction
    let mut btn_2 = 
    DeButton::new(pins.button2.into_pull_up_input(),
                  5_000,
                  Default::default());
    
    #[derive(Debug,  // used as fmt
         Clone, Copy, // because next_state 
         PartialEq, Eq)] // testing equality
    enum State {Init, New, Ready, Record, Send}
    let mut state: State = State::Init;
    //let mut next_state: State;

    loop{
        // Interaction
        btn_1.update();
        if btn_1.state {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap()
        }

        btn_2.poll();

        // Collecting events, initiate transitions
        let next_state = 
        match (&state, btn_2.read()) {
            (State::Init,       _)  => State::New, // automatic transitional
            (State::New,    Press)  => State::Ready,
            (State::Ready,  Press)  => State::Record,
            (State::Record, Press)  => State::Ready,
            (State::New, LongPress) => State::Send,
            (_,          LongPress) => State::New,
            _                       => state,};

        // Static UI, doing transition
        if next_state != state {
            match (state, next_state) {
                (_,State::New)      => rgb.white(),
                (_,State::Ready)    => {rgb.green(); trial = trial + 1; },
                (_,State::Record)   => rgb.red(),
                (_,State::Send)     => rgb.blue(),
                (_,_)               => rgb.off(),};
            state = next_state;
            // transition complete

        // Continuous stuff
        match &state {
            State::New      => {},
            State::Ready    => {},
            State::Record   => {this_value = adc.read(&mut adc_pin_0).unwrap();},
            State::Send     => {},
            _ => (),};
        };


// EATME
/*         if btn_1.state {
            if state == "Stop" {        
                trial = 0;
                rgb.write(white);
                //sleep(&mut delay, 2000); // waiting for user input
                state = "Pause";
                continue
            }
            if state == "Pause" {
                trial += 1;
                rgb.write(green);
                //sleep(&mut delay, 500);
                state = "Record";
                continue
            }
            if state == "Record" {
                rgb.write(red);        
                //sleep(&mut delay, 2000);
                if trial < n_trials {
                    state = "Pause";
                } else {
                    state = "Stop";
                }
                continue
        }

        // Continuous processing
 */

/*         }
        if state == "Pause" || state == "Record" {
            this_value = adc.read(&mut adc_pin_0).unwrap();
            delay.delay_ms(20);
        }
 */
    }
  }
