pub mod fake {
    use embassy_time::{Duration, Ticker, Instant};

    /* data */
    pub type Reading = [u16;4];
    pub struct Result {
        pub time: Instant,
        pub reading: Reading
    }
    /* result channel */
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;
    pub static RESULT: Signal<CriticalSectionRawMutex, Result> 
    = Signal::new();
    
    /* control channel */
    pub enum State {Ready, Record}
    pub static CONTROL: Signal<CriticalSectionRawMutex, State> 
    = Signal::new();
    
    #[embassy_executor::task]
    pub async fn task(hz: u64) {
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        loop {
                let reading: Reading = [0,0,0,0];
                let now: Instant = Instant::now();
                let result = Result{time: now, reading: reading};
                RESULT.signal(result);
                ticker.next()
                .await;
                };
            }
        }



    pub mod adc {
        use embassy_time::{Duration, Ticker, Instant};
        use embassy_rp::peripherals::{PIN_26, PIN_27, PIN_28, PIN_29, ADC};
        use embassy_rp::adc::{Adc, Config, InterruptHandler};
        use embassy_rp::bind_interrupts;
        bind_interrupts!(struct Irqs {
            ADC_IRQ_FIFO => InterruptHandler;
        });

        /* data */
        pub struct SensorResult<R> { // <-- redundant
            pub time: Instant,
            pub reading: R,
        }
        pub type Reading = [u16; 4];
        pub struct Result {
            pub time: Instant,
            pub reading: Reading
        }
        /* result channel */
        use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
        use embassy_sync::signal::Signal;
        pub static RESULT: Signal<CriticalSectionRawMutex, Result> 
        = Signal::new();
        
        /* control channel */
        pub enum State {Ready, Record}
        pub static CONTROL: Signal<CriticalSectionRawMutex, State> 
        = Signal::new();
        //type AdcPin: embedded_hal::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;
        
        #[embassy_executor::task]
        pub async fn task(//mut adc: Adc<'static>, 
                        adc_contr: ADC,
                        mut adc_0: PIN_26,
                        mut adc_1: PIN_27,
                        mut adc_2: PIN_28,
                        mut adc_3: PIN_29,
                        hz: u64) {
            //let adc: Adc<'_> = Adc::new(p.ADC, Irqs, Config::default());
            let mut adc: Adc<'_> = Adc::new(adc_contr, Irqs, Config::default());
            let mut ticker = Ticker::every(Duration::from_hz(hz));
            //let _state: State = State::Ready;
            loop {
               /* match CONTROL.wait().await { 
                    State::Ready => {},
                    State::Record => {*/
                let reading: Reading = [
                    adc.read(&mut adc_0).await,
                    adc.read(&mut adc_1).await,
                    adc.read(&mut adc_2).await,
                    adc.read(&mut adc_3).await, ];
                let now: Instant = Instant::now();
                let result = Result{time: now, 
                                            reading: reading};
                RESULT.signal(result);
                ticker.next()
                .await;
                  //  },
                //}
            }
        }
    }


/* ADS1115 Sensor */
pub mod ads1115 {
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
    //use embedded_ads111x as ads111x;
    //use embedded_ads111x::InputMultiplexer::{AIN0GND, AIN1GND, AIN2GND, AIN3GND};
    use embedded_hal::adc::OneShot;
    use ads1x1x::{channel, Ads1x1x, SlaveAddr, DataRate16Bit, DataRate12Bit};
    use nb::block;

    // ITC
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;

    // Data
    type Reading = [i16;4];
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
        let address = SlaveAddr::default();
        let mut ads = Ads1x1x::new_ads1015(i2c, address);
        // ads.set_data_rate(DataRate16Bit::Sps860).unwrap();
        ads.set_data_rate(DataRate12Bit::Sps3300).unwrap();
        //ads.into_continuous();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        loop {
            let reading: Reading = [
                block!(ads.read(&mut channel::SingleA0)).unwrap(),
                block!(ads.read(&mut channel::SingleA1)).unwrap(),
                block!(ads.read(&mut channel::SingleA2)).unwrap(),
                block!(ads.read(&mut channel::SingleA3)).unwrap(),
            ];
            let now = Instant::now();
            let result = 
                Measure{time: now, reading: reading};       
            RESULT.signal(result);
            ticker.next().await;
            }
    }
                           
}



    /* pub mod yxz {
        /* Sensor Generics */
        use embassy_time::{Duration, Ticker, Instant};
        
        // Generic result
        pub struct SensorResult<R> {
            pub time: Instant,
            pub reading: R,
        }
        
        // I2C    
        use embassy_rp::i2c::{self, Config, InterruptHandler};
        use embassy_rp::peripherals::{PIN_2, PIN_3, I2C1};
        use embassy_rp::bind_interrupts;
        use lsm6ds33 as lsm6;
        // ITC
        use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
        use embassy_sync::signal::Signal;
        pub type Reading = (f32, f32, f32);
        pub type Measure = SensorResult<Reading>;
        pub static RESULT:Signal<CriticalSectionRawMutex, Measure> 
                    = Signal::new();
    
        #[embassy_executor::task]
        pub async fn task(contr: I2C1,
                          scl: PIN_3,
                          sda: PIN_2,
                          hz: u64) {
            // Init I2C
            bind_interrupts!(struct Irqs {
                I2C1_IRQ => InterruptHandler<I2C1>;
            });
            let i2c: i2c::I2c<'_, I2C1, i2c::Async> = 
                i2c::I2c::new_async(contr, 
                                    scl, sda, 
                                    Irqs, 
                                    Config::default());
            
            
            let sensor_res = lsm6::Lsm6ds33::new(i2c, 0x6Au8);
            // Debug is not implemented, that's why unwrap won't work
            let mut sensor 
                    = match sensor_res {
                        Result::Ok(thing) => thing,
                        Result::Err(_) => panic!("Nooo!")};
            
            let mut ticker 
                    = Ticker::every(Duration::from_hz(hz));
            loop {
                let reading: Reading = sensor.read_gyro().unwrap();   
                let now = Instant::now();
                let result = 
                    Measure{time: now, reading: reading};       
                RESULT.signal(result);
                ticker.next().await;
            }
        }
                               
    
    }*/

    /* ADS1115 Sensor */
/* pub mod ads1115 {
    /* Sensor Generics */
    use embassy_time::{Duration, Ticker, Instant};
    
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    
    // I2C    
    use embassy_rp::i2c::{self, Config, InterruptHandler};
    use embassy_rp::peripherals::{PIN_3, PIN_2, I2C1};
    use embassy_rp::bind_interrupts;
    use embedded_ads111x as ads;
    use embedded_ads111x::InputMultiplexer::{AIN0GND, AIN1GND, AIN2GND, AIN3GND};
    // ITC
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;
    pub type Reading = (f32, f32, f32, f32);
    pub type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<CriticalSectionRawMutex, Measure> 
                = Signal::new();

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
            ads::ADS111xConfig::default()
            .dr(ads::DataRate::SPS8)
            .pga(ads::ProgramableGainAmplifier::V4_096)
            .mode(ads::Mode::Continuous);
        
        let mut ads: ads::ADS111x<i2c::I2c<'_, I2C1, i2c::Async>> = 
            ads::ADS111x::new(i2c,
                                0x48u8, config).unwrap();
        
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        loop {
            let reading: Reading =
                (ads.read_single_voltage(Some(AIN0GND)).unwrap(),
                ads.read_single_voltage(Some(AIN1GND)).unwrap(),
                ads.read_single_voltage(Some(AIN2GND)).unwrap(),
                ads.read_single_voltage(Some(AIN3GND)).unwrap());
            let now = Instant::now();
            let result = 
                Measure{time: now, reading: reading};       
            RESULT.signal(result);
            ticker.next().await;
            }
    }
                           
}*/

    