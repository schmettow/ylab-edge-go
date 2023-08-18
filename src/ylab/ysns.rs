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
    use embassy_rp::peripherals::{PIN_26, PIN_27, PIN_28, PIN_29};
    use embassy_rp::adc::Adc;

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
    
    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    //type AdcPin: embedded_hal::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;
    
    #[embassy_executor::task]
    pub async fn task(mut adc: Adc<'static>,
                    mut adc_0: PIN_26,
                    mut adc_1: PIN_27,
                    mut adc_2: PIN_28,
                    mut adc_3: PIN_29,
                    hz: u64) {
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>; 
        loop {
            ticker.next().await;
            if RECORD.load(Ordering::Relaxed){
                reading = [
                    adc.read(&mut adc_0).await,
                    adc.read(&mut adc_1).await,
                    adc.read(&mut adc_2).await,
                    adc.read(&mut adc_3).await, ];
                result = SensorResult{
                            time: Instant::now(), 
                            reading: reading};
                
                //log::info!("{},{},{},{}", 
                log::info!("{},0,{},{},{},{}", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);
                };
            }                
        }
    }


///# ADS1015 on I2C1
pub mod ads1015 { 
    /// ## Sensor Generics
    use embassy_time::{Duration, Ticker, Instant};
    
    /// ## I2C    
    use embassy_rp::i2c::{self};
    ///
    /// Change this and Data Rate to switch I2C0/1
    use embassy_rp::peripherals::I2C1 as I2C;
    use embedded_hal::adc::OneShot;
    use ads1x1x::{channel, Ads1x1x, SlaveAddr};
    use ads1x1x::DataRate12Bit as DataRate;
    use nb::block;

    // ITC
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;

    // Data
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    type Reading = [i16;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<CriticalSectionRawMutex, Measure> = Signal::new();

    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);
 
    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C, i2c::Async>,
                      hz: u64) {
        let address = SlaveAddr::default();
        let mut ads 
                = Ads1x1x::new_ads1015(i2c, address);
        // ads.set_data_rate(DataRate16Bit::Sps860).unwrap();
        ads.set_data_rate(DataRate::Sps3300).unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, Ordering::Relaxed);
        loop {
            ticker.next().await;
            if RECORD.load(Ordering::Relaxed){
                reading = [
                    block!(ads.read(&mut channel::SingleA0)).unwrap(),
                    block!(ads.read(&mut channel::SingleA1)).unwrap(),
                    block!(ads.read(&mut channel::SingleA2)).unwrap(),
                    block!(ads.read(&mut channel::SingleA3)).unwrap(),];
                result = SensorResult{time: Instant::now(), 
                                      reading: reading};
                log::info!("{},2,{},{},{},{}", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);
                    };
                }
            }
    }


/* ADS1115 Sensor I2C1 */
pub mod ads1115 { 
    /* Sensor Generics */
    use embassy_time::{Duration, Ticker, Instant};
    
    // I2C    
    use embassy_rp::i2c::{self};
    use embassy_rp::peripherals::I2C1 as I2C;
    use embedded_hal::adc::OneShot;
    use ads1x1x::{channel, Ads1x1x, SlaveAddr};
    // ads1115 takes 16 bit
    use ads1x1x::DataRate16Bit as DataRate; // <-----
    use nb::block;

    // ITC
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;

    // Data
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    type Reading = [i16;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<CriticalSectionRawMutex, Measure> = Signal::new();

    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);
 
    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C, i2c::Async>,
                      hz: u64) {
        let address = SlaveAddr::default();
        let mut ads 
                = Ads1x1x::new_ads1115(i2c, address);
        ads.set_data_rate(DataRate::Sps860).unwrap();
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, Ordering::Relaxed);
        loop {
            ticker.next().await;
            if RECORD.load(Ordering::Relaxed){
                reading = [
                    block!(ads.read(&mut channel::SingleA0)).unwrap(),
                    block!(ads.read(&mut channel::SingleA1)).unwrap(),
                    block!(ads.read(&mut channel::SingleA2)).unwrap(),
                    block!(ads.read(&mut channel::SingleA3)).unwrap(),];
                result = SensorResult{time: Instant::now(), 
                                      reading: reading};
                log::info!("{},2,{},{},{},{}", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);
                    };
                }
            }
    }


/* ADS1015 Sensor in Continuous*/
pub mod ads1015_conti {
    // use embassy_rp::gpio::{AnyPin, Input};
    /* Sensor Generics */
    use embassy_time::{Duration, Ticker, Instant};
    // I2C    
    use embassy_rp::i2c::{self};
    use embassy_rp::peripherals::I2C0 as I2C; // <----
    use ads1x1x::{channel, Ads1x1x, SlaveAddr};
    // ads1115 takes 12 bit
    use ads1x1x::DataRate12Bit as DataRate; // <----
    // use embedded_hal_async::digital::Wait;

    // ITC
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;

    // Data
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }

    type Reading = [i16;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<CriticalSectionRawMutex, Measure> = Signal::new();

    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);
 
    // use embassy_rp::gpio::Pull;

    #[embassy_executor::task]
    pub async fn task(i2c: i2c::I2c<'static, I2C, i2c::Async>,
                      // ready_pin: AnyPin,
                      hz: u64) {
        let address = SlaveAddr::default();
        let ads 
                = Ads1x1x::new_ads1015(i2c, address);   // <------
        let mut ads 
                = Ads1x1x::into_continuous(ads).ok().unwrap();
        ads.set_data_rate(DataRate::Sps1600).unwrap();
        // ads.use_alert_rdy_pin_as_ready().unwrap(); // <-------
        
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, Ordering::Relaxed);

        /* let mut ready 
                = Input::new(ready_pin, Pull::Up);*/
        let mut ticker: Ticker = Ticker::every(Duration::from_hz(hz * 4)); // <--- times 4
        loop {
            if RECORD.load(Ordering::Relaxed){
                // Including wait time saves much of the blocking time.
                ads.select_channel(&mut channel::SingleA0).unwrap();
                ticker.next().await;
                let a0 = ads.read();
                ads.select_channel(&mut channel::SingleA1).unwrap();
                ticker.next().await;
                //ready.wait_for_falling_edge().await;
                let a1 = ads.read();
                ads.select_channel(&mut channel::SingleA2).unwrap();
                ticker.next().await;
                //ready.wait_for_falling_edge().await;
                let a2 = ads.read();
                ads.select_channel(&mut channel::SingleA3).unwrap();
                ticker.next().await;
                //ready.wait_for_falling_edge().await;
                let a3 = ads.read();
                
                reading = [a0.unwrap(), a1.unwrap(), a2.unwrap(), a3.unwrap()];
                result = SensorResult{time: Instant::now(), 
                                      reading: reading};
                log::info!("{},1,{},{},{},{}", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);
                    };
                }
            }
    }



pub mod yxz_lsm6 {
    /* Sensor Generics */
    use embassy_time::{Duration, Ticker, Instant};
        
    // Generic result
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    pub type Reading = [f32; 3];
    pub type Measure = SensorResult<Reading>;
    
    // I2C    
    use embassy_rp::i2c;
    use embassy_rp::peripherals::I2C1;
    use lsm6ds33 as lsm6;

    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);


    #[embassy_executor::task]
    pub async fn task(  i2c: i2c::I2c<'static, I2C1, i2c::Async>,
                        hz: u64) {           
        let sensor_res 
            = lsm6::Lsm6ds33::new(i2c, 0x6Au8);
        // Debug is not implemented, that's why unwrap won't work
        let mut sensor 
                = match sensor_res {
                    Result::Ok(thing) => thing,
                    Result::Err(_) => panic!("Nooo!")};
        
        let mut ticker 
                = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, Ordering::Relaxed);
        loop {
            ticker.next().await;
            if RECORD.load(Ordering::Relaxed){
                reading = sensor.read_accelerometer().unwrap().into();   
                result = Measure{time: Instant::now(), reading: reading};
                log::info!("{},1,{},{},{}", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],);
                };
            }
        }
    }



/// ## BMI Acceleration Sensor

    pub mod yxz_bmi160 {
        /* Sensor Generics */
        // extern crate bmi160;
        use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
        use embassy_time::{Duration, Ticker, Instant};
        
        // Generic result
        pub struct SensorResult<R> {
            pub time: Instant,
            pub reading: R,
        }
        pub type Reading = [f32; 3]; /// <--- 4 channel is total accel for now
        pub type Measure = SensorResult<Reading>;
        
        // I2C    
        use embassy_rp::i2c;
        use embassy_rp::peripherals::I2C1;
        

        /* control channels */
        pub use core::sync::atomic::Ordering;
        use core::sync::atomic::AtomicBool;
        pub static READY: AtomicBool = AtomicBool::new(false);
        pub static RECORD: AtomicBool = AtomicBool::new(false);
    
        #[embassy_executor::task]
        pub async fn task(  i2c: i2c::I2c<'static, I2C1, i2c::Blocking>,
                            hz: u64) {    
            let address = SlaveAddr::default();
            let mut sensor 
                    = Bmi160::new_with_i2c(i2c, address);
            sensor.set_accel_power_mode(AccelerometerPowerMode::Normal).unwrap();
            sensor.set_gyro_power_mode(GyroscopePowerMode::Normal).unwrap();                       
            let mut ticker 
                    = Ticker::every(Duration::from_hz(hz));
            let mut reading: Reading;
            let mut result: SensorResult<Reading>;
            READY.store(true, Ordering::Relaxed);
            loop {
                ticker.next().await;
                if RECORD.load(Ordering::Relaxed){
                    let data = sensor.data(SensorSelector::new().accel().gyro()).unwrap();
                    let accel = data.accel.unwrap();
                    reading = [accel.x.into(), accel.y.into(), accel.z.into()];
                    //let gyro = data.gyro.unwrap();
                    result = 
                        Measure{time: Instant::now(), 
                            reading: reading.into()};
                    log::info!("{},1,{},{},{},{}", 
                        result.time.as_micros(),
                        result.reading[0],
                        result.reading[1],
                        result.reading[2],
                        result.reading[0] + result.reading[1] + result.reading[2]);
                    };
                }
            }
        }

pub mod yirt { // MLX90614
    /* Sensor Generics */
    use mlx9061x::{Mlx9061x, SlaveAddr};
    use embassy_time::{Duration, Ticker, Instant};
    
    // Generic result
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    pub type Reading = [f32; 2];
    pub type Measure = SensorResult<Reading>;
    
    // I2C    
    use embassy_rp::i2c;
    use embassy_rp::peripherals::I2C0;
    

    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);


    #[embassy_executor::task]
    pub async fn task(  i2c: i2c::I2c<'static, I2C0, i2c::Blocking>,
                        hz: u64) {    
        let address = SlaveAddr::default();
        let mut sensor = Mlx9061x::new_mlx90614(i2c, address, 5).unwrap();
        let mut ticker 
                = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, Ordering::Relaxed);
        loop {
            ticker.next().await;
            if RECORD.load(Ordering::Relaxed){
                let obj_temp:f32 = sensor.object1_temperature().unwrap().into();
                let amb_temp:f32 = sensor.ambient_temperature().unwrap().into();
                //let amb_temp: f32 = 1.0;
                //let obj_temp: f32 = 2.0;
                reading = [obj_temp.into(), amb_temp.into()];
                result = 
                    Measure{time: Instant::now(), 
                        reading: reading.into()};
                log::info!("{},T,{},{}", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1]);
                };
            }
        }
    }

