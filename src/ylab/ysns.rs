pub use crate::*;
pub use yuio::disp::TEXT as DISP;
use hal::i2c;
use i2c::Async as Mode;
use crate::ytfk::bsu::SINK;

pub struct SensorResult<R> {
    pub time: Instant,
    pub reading: R,
}


pub mod fake {
    use super::*;
    /* data */
    pub type Reading = [u16;4];
    pub struct Result {
        pub time: Instant,
        pub reading: Reading
    }
    /* result channel */
    pub static RESULT: Signal<RawMutex, Result> 
    = Signal::new();
    
    /* control channel */
    pub enum State {Ready, Record}
    pub static CONTROL: Signal<RawMutex, State>  = Signal::new();
    
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



pub mod moi {
    use super::*;
    use hal::peripherals::{PIN_21, PIN_22, PIN_8, PIN_9};
    use hal::gpio::{Input, Pull};

    
    pub type Measure = bool;
    pub type Reading<const N: usize> = [Measure; N];
    pub type Sample<const N: usize> = crate::Sample<Measure, N>;
    
    pub struct Result<const N: usize> {
        pub time: Instant,
        pub reading: Reading<N>
    }
    
    /* result channel */
    //pub static RESULT: Signal<RawMutex, Sample>  = Signal::new();

    /* trigger channel */
    //pub static TRIGGER: Channel<RawMutex, Sample, 2> = Channel::new();
    
    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);
    
    #[embassy_executor::task]
    pub async fn task(moi_0: PIN_21, moi_1: PIN_22, moi_2: PIN_8, moi_3: PIN_9, sensory: u8) {
    //pub async fn task(pins: [AnyPin; 4], trigger: [(bool, Option<bool>); 4], hz: u64, sensory: u8) {
        let mut moi_0 = Input::new(moi_0, Pull::Up);
        let mut moi_1 = Input::new(moi_1, Pull::Up);
        let mut moi_2 = Input::new(moi_2, Pull::Up);
        let mut moi_3 = Input::new(moi_3, Pull::Up);

        //let last_reading: Reading = [false, false, false, false,];
        //let mut reading: Reading<4>;
        use embassy_futures::select::select;
        loop {
            if RECORD.load(ORD){
                select(
                    select(moi_0.wait_for_any_edge(), moi_1.wait_for_any_edge()),
                    select(moi_2.wait_for_any_edge(), moi_3.wait_for_any_edge())).await;
                //moi_3.wait_for_any_edge().await;
                let reading = [moi_0.get_level().into(), moi_1.get_level().into(), moi_2.get_level().into(), moi_3.get_level().into()];
                let sample = Sample {
                            sensory: sensory,
                            time: Instant::now(), 
                            read: reading};
                SINK.send(sample.into()).await;
                /*log::info!("{},{},{},{},{},{},,,,", 
                    sample.time.as_micros(),
                    sample.sensory,
                    sample.read[0] as u8,
                    sample.read[1] as u8,
                    sample.read[2] as u8,
                    sample.read[3] as u8);*/
                // TRIGGER.send(sample).await; <--- makes it hang
                };
            }                
        }


        #[embassy_executor::task]
        pub async fn task_2(moi_0: PIN_21, moi_1: PIN_22, sensory: u8) {
        //pub async fn task(pins: [AnyPin; 4], trigger: [(bool, Option<bool>); 4], hz: u64, sensory: u8) {
            let mut sample: Sample<2>; 
            let mut moi_0 = Input::new(moi_0, Pull::Up);
            let mut moi_1 = Input::new(moi_1, Pull::Up);
            
            let mut reading: Reading<2>;
            use embassy_futures::select::select;
            loop {
                if RECORD.load(ORD){
                    select(moi_0.wait_for_any_edge(), moi_1.wait_for_any_edge()).await;
                    reading = [moi_0.get_level().into(), moi_1.get_level().into()];
                    sample = Sample{
                                sensory: sensory,
                                time: Instant::now(), 
                                read: reading};
                    SINK.send(sample.into()).await;
                    /*log::info!("{},{},{},{},{},{},,,,", 
                        sample.time.as_micros(),
                        sample.sensory,
                        sample.read[0] as u8,
                        sample.read[1] as u8,
                        sample.read[2] as u8,
                        sample.read[3] as u8);*/
                    // TRIGGER.send(sample).await; <--- makes it hang
                    };
                }                
            }
    }
   

pub mod adc {
    
    use super::*;
    use hal::peripherals::{PIN_26, PIN_27, PIN_28, PIN_29};
    use hal::adc::{Adc, Async, Channel};
    use hal::gpio::Pull;

    pub type Reading = [u16; 4];
    pub struct Result {
        pub time: Instant,
        pub reading: Reading
    }
    /* result channel */
    pub static RESULT: Signal<RawMutex, Result>  = Signal::new();
    
    /* control channels */
    
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    //type AdcPin: embedded_hal::adc::Channel<embassy_rp::adc::Adc<'static>> + embassy_rp::gpio::Pin;
    
    #[embassy_executor::task]
    pub async fn task(mut adc: Adc<'static, Async>,
                    adc_0: PIN_26,
                    adc_1: PIN_27,
                    adc_2: PIN_28,
                    adc_3: PIN_29,
                    hz: u64,
                sensory: u8) {
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //let mut reading: Reading;
        //let mut result: SensorResult<Reading>; 
        let mut chan = 
        [ Channel::new_pin(adc_0, Pull::None),
          Channel::new_pin(adc_1, Pull::None),
          Channel::new_pin(adc_2, Pull::None),
          Channel::new_pin(adc_3, Pull::None),];

        loop {
            ticker.next().await;
            if RECORD.load(ORD){
                let reading = [ adc.read(&mut chan[0]).await.unwrap(),
                            adc.read(&mut chan[1]).await.unwrap(),
                            adc.read(&mut chan[2]).await.unwrap(),
                            adc.read(&mut chan[3]).await.unwrap()];
                
                let sample = Sample{
                            sensory: sensory,
                            time: Instant::now(), 
                            read: reading};
                
                SINK.send(sample.into()).await;
                
                //log::info!("{},{},{},{}", 
                /*log::info!("{},{},{},{},{},{},,,,", 
                    result.time.as_micros(),
                    sensory,
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);*/
                };
            }                
        }
    }

/*
///# ADS1015 on I2C1
pub mod ads1015 { 
    use super::*;
    /// ## Sensor Generics
    use embassy_time::{Duration, Ticker, Instant};
    
    /// ## I2C    
    use embassy_rp::i2c::{self};
    ///
    /// Change this and Data Rate to switch I2C0/1
    use hal::peripherals::I2C1 as I2C;
    use ads1x1x::{channel, Ads1x1x, SlaveAddr};
    use ads1x1x::DataRate12Bit as DataRate;
    use nb::block;

    // ITC
    // Data
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    type Reading = [i16;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<Mutex, Measure> = Signal::new();

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
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD){
                reading = [0; 4
                    /*block!(ads.read(&mut channel::SingleA0)).unwrap(),
                    block!(ads.read(&mut channel::SingleA1)).unwrap(),
                    block!(ads.read(&mut channel::SingleA2)).unwrap(),
                    block!(ads.read(&mut channel::SingleA3)).unwrap(),*/
                    ];
                result = SensorResult{time: Instant::now(), 
                                      reading: reading};
                log::info!("{},2,{},{},{},{},,,,", 
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
    use super::*;
    /* Sensor Generics */
    use embassy_time::{Duration, Ticker, Instant};
    
    // I2C    
    use hal::i2c::{self};
    use hal::peripherals::I2C1 as I2C;
    use ads1x1x::{channel, Ads1x1x, SlaveAddr};
    // ads1115 takes 16 bit
    use ads1x1x::DataRate16Bit as DataRate; // <-----
    use nb::block;

    // Data
    pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }
    type Reading = [i16;4];
    type Measure = SensorResult<Reading>;
    pub static RESULT:Signal<Mutex, Measure> = Signal::new();

    /* control channels */
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
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD){
                reading = [0; 4];
                    /*block!(ads.read(&mut channel::SingleA0)).unwrap(),
                    block!(ads.read(&mut channel::SingleA1)).unwrap(),
                    block!(ads.read(&mut channel::SingleA2)).unwrap(),
                    block!(ads.read(&mut channel::SingleA3)).unwrap(),];*/
                result = SensorResult{time: Instant::now(), 
                                      reading: reading};
                log::info!("{},2,{},{},{},{},,,,", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],
                    result.reading[3],);
                    };
                }
            }
    }

*/

pub mod yxz_lsm6_old {
    use super::*;
    use hal::peripherals::I2C0 as I2C;
    use lsm6ds33::Lsm6ds33 as Lsm6;
    use i2c::Blocking as Mode;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    // Generic result
    /*pub struct SensorResult<R> {
        pub time: Instant,
        pub reading: R,
    }*/
    pub type Reading = [f32; 3]; /// <--- 4 channel is total accel for now
    pub type Measure = SensorResult<Reading>;

    #[embassy_executor::task]
    pub async fn task(  i2c: i2c::I2c<'static, I2C, Mode>,
                        hz: u64,
                        sensory: u8) { 
        DISP.signal([None, None, None, Some("Lsm6 task".try_into().unwrap())]);         
        let sensor_res: Result<Lsm6<i2c::I2c<'_, I2C, Mode>>, (i2c::I2c<'_, I2C, Mode>, lsm6ds33::Error<i2c::Error>)> = Lsm6::new(i2c, 0x6Au8);
        let mut sensor 
                = match sensor_res {
                    Result::Ok(sensor) => sensor,
                    Result::Err(_) => 
                        {DISP.signal([None, None, None, 
                                Some("Lsm6 =/= I2C".try_into().unwrap())]);
                        panic!()}};
        let mut ticker 
                = Ticker::every(Duration::from_hz(hz));
        let mut reading: Reading;
        let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        DISP.signal([None, None, None, Some("Lsm6 ticking".try_into().unwrap())]);
        loop {
            //DISP.signal([None, None, None, Some("Lsm6 reading".try_into().unwrap())]);
            ticker.next().await;
            if RECORD.load(ORD){
                reading = sensor.read_accelerometer().unwrap().into();   
                result = Measure{time: Instant::now(), reading: reading};
                log::info!("{},{},{},{},{},,,,", 
                    sensory,
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1],
                    result.reading[2],);
                };
            }
        }
    }




pub mod yxz_lsm6 {
    
    use super::*;
    use hal::peripherals::I2C0 as I2C;
    use lsm6dsox::*;
    use Lsm6dsox as Lsm6;
    use accelerometer::Accelerometer;
    
    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(true);
    const N: usize = 6;
    pub type Measure  = f32;
    pub type Reading  = [Measure; N];
    pub type Sample = crate::Sample<Measure, N>;
    
    

    #[embassy_executor::task]
    pub async fn task(  i2c: i2c::I2c<'static, I2C, Mode>,
                        hz: u64,
                        sensory: u8) { 
        
        let mut sensor 
            = Lsm6::new(i2c, SlaveAddress::Low, time::Delay);
        log::debug!("Yxz init");
        sensor.setup().unwrap();
        sensor.set_accel_sample_rate(DataRate::Freq1660Hz).unwrap();
        sensor.set_accel_scale(AccelerometerScale::Accel2g).unwrap();
        sensor.set_gyro_sample_rate(DataRate::Freq1660Hz).unwrap();
        sensor.set_gyro_scale(GyroscopeScale::Dps250).unwrap();
        log::debug!("Yxz set");
        let mut ticker 
                = Ticker::every(Duration::from_hz(hz));
        //let mut reading: Reading;
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        
        loop {
            if RECORD.load(ORD){
                log::debug!("Yxz get");
                let accel = sensor.accel_norm().unwrap();
                let gyro = sensor.angular_rate().unwrap();
                let reading = [ accel.x, accel.y, accel.z,
                            gyro.x.as_hertz() as f32, 
                            gyro.y.as_hertz() as f32, 
                            gyro.z.as_hertz() as f32];

                let sample = Sample{sensory: sensory, time: Instant::now(), read: reading};
                SINK.send(sample.into()).await;
                log::debug!("Yxz read");
                ticker.next().await;
                };
            }
        }


        use xca9548a::{Xca9548a, SlaveAddr};
        #[embassy_executor::task]
        pub async fn multi_task(i2c: i2c::I2c<'static, I2C, Mode>,
                                n: u8,
                                hz: u64, just_spin: bool,
                                sensory: u8) { 
            DISP.signal([None, None, None, Some("Multi-Lsm6 task".try_into().unwrap())]);
            let tca = Xca9548a::new(i2c, SlaveAddr::default());
            DISP.signal([None, None, None, Some("TCA |==| I2C".try_into().unwrap())]);
            let hub = tca.split();
            let sen_0 = Lsm6::new(hub.i2c0, SlaveAddress::Low, time::Delay);
            let sen_1 = Lsm6::new(hub.i2c1, SlaveAddress::Low, time::Delay);
            let sen_2 = Lsm6::new(hub.i2c2, SlaveAddress::Low, time::Delay);
            let sen_3 = Lsm6::new(hub.i2c3, SlaveAddress::Low, time::Delay);
            let sen_4 = Lsm6::new(hub.i2c4, SlaveAddress::Low, time::Delay);
            let sen_5 = Lsm6::new(hub.i2c5, SlaveAddress::Low, time::Delay);
            let sen_6 = Lsm6::new(hub.i2c6, SlaveAddress::Low, time::Delay);
            let sen_7 = Lsm6::new(hub.i2c7, SlaveAddress::Low, time::Delay);
            let mut sensory = [sen_0, sen_1, sen_2, sen_3, sen_4, sen_5, sen_6, sen_7];
            //let mut sensory = [Some(sen_0), Some(sen_1), Some(sen_2), Some(sen_3), Some(sen_4), Some(sen_5), Some(sen_6), Some(sen_7)];
            let data_rate = DataRate::Freq6660Hz;
            for (s, sens) in sensory.as_mut().into_iter().enumerate() {
                if s >= n as usize {continue}
                else {  sens.set_accel_sample_rate(data_rate).unwrap();
                        sens.set_gyro_sample_rate(data_rate).unwrap();};
            }
            //DISP.signal([None, None, None, Some("LSM6x3".try_into().unwrap())]);
            let mut ticker 
                    = Ticker::every(Duration::from_hz(hz));
            //let mut reading: Reading;
            //let mut result: Sample;
            READY.store(true, ORD);
            loop {
                if RECORD.load(ORD){
                    for (s, sensor) in sensory.as_mut().into_iter().enumerate() {
                        if s >= n as usize {continue}
                        let accel = sensor.accel_norm().unwrap();
                        let gyro = sensor.angular_rate().unwrap();
                        let reading = [ accel.x, accel.y, accel.z,
                                gyro.x.as_hertz() as f32, 
                                gyro.y.as_hertz() as f32, 
                                gyro.z.as_hertz() as f32];
                        let sample = Sample{sensory: s as u8, time: Instant::now(), read: reading};
                        SINK.send(sample.into()).await;
                        /*log::info!("{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},,", 
                            result.time.as_micros(),
                            s,
                            result.read[0],
                            result.read[1],
                            result.read[2],
                            result.read[3],
                            result.read[4],
                            result.read[5],);*/
                        }
                    };
                    if !just_spin {ticker.next().await;};
                }
            }
    



    //use hal::bind_interrupts;
    type SharedI2C = Mutex<RawMutex, Option<I2C>>;
    
    #[embassy_executor::task(pool_size = 3)]
    pub async fn sharing_task(i2c: &'static SharedI2C,
        scl: &'static  Mutex<RawMutex, Option<impl i2c::SclPin<I2C>>>,
        sda: &'static  Mutex<RawMutex, Option<impl i2c::SdaPin<I2C>>>,
        hz: u64)
        -> ()
        {
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //DISP.signal([None, None, None, Some("LSM shared ticks".try_into().unwrap())]);
        //let mut reading: Reading;
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD){
                match read(i2c, scl, sda).await{
                    Ok(Some(_reading)) 
                        => todo!(),
                    _   => {}
                    }        
                };
            }
        }

    /// One-shot read
    /// 
    /// does a full round of initialization and one
    /// DEACTIVATED, because interrupt binding collides.

    pub async fn read(_i2c: &'static SharedI2C,
        
        _scl: &'static  Mutex<RawMutex, Option<impl i2c::SclPin<I2C>>>,
        _sda: &'static  Mutex<RawMutex, Option<impl i2c::SdaPin<I2C>>>,)
        -> Result<Option<Reading>,Error>
        {
            
        {   todo!();
            // inner scope   
            /* bind_interrupts!(struct Irqs {
                I2C0_IRQ => i2c::InterruptHandler<I2C>;
            });
            let mut i2c_unlocked = i2c.lock().await;
            let mut scl = scl.lock().await;
            let mut sda = sda.lock().await;
            //DISP.signal([None, Some("Got lock".try_into().unwrap()), None,None]);
            if let Some(i2c) = i2c_unlocked.as_mut() {
                let i2c 
                    = i2c::I2c::new_async(i2c,  scl.as_mut().unwrap(), sda.as_mut().unwrap(), Irqs, i2c::Config::default());
                //DISP.signal([None, Some("Got I2C".try_into().unwrap()), None,None]);
                let mut sensor 
                    = Lsm6::new(i2c, SlaveAddress::Low, time::Delay);
                //DISP.signal([None, Some("Got LSM".try_into().unwrap()), None,None]);
                sensor.setup().unwrap();
                sensor.set_accel_sample_rate(DataRate::Freq416Hz).unwrap();
                sensor.set_gyro_sample_rate(DataRate::Freq416Hz).unwrap();
                //sensor.set_accel_scale(AccelerometerScale::Accel2g).unwrap();
                //sensor.set_gyro_scale(GyroscopeScale::Dps250).unwrap();
                //DISP.signal([None, Some("All set".try_into().unwrap()), None,None]);
                let accel = sensor.accel_norm();
                let gyro = sensor.angular_rate();
                match (accel, gyro) {
                    (Ok(accel), Ok(gyro)) 
                    => {//DISP.signal([None, Some("Got Read".try_into().unwrap()), None,None]);
                        let reading = [ accel.x, accel.y, accel.z,
                                    gyro.x.as_rpm() as f32, 
                                    gyro.y.as_rpm() as f32, 
                                    gyro.z.as_rpm() as f32];
                        DISP.signal([None, Some("Got Sense".try_into().unwrap()), None,None]);
                        return Ok(Some(reading))
                        },
                    (Err(_), Err(_)) => return Ok(None),
                    (_,_) => Ok(None),
                    }
            } else {return Err(lsm6dsox::Error::ResetFailed)}*/
        }// inner scope
        }

    }
    



/// ## BMI Acceleration Sensor

pub mod yxz_bmi160 {
        use super::*;
        #[allow(unused)]
        use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
        use hal::peripherals::I2C1 as I2C;

        /* control channels */
        pub static READY: AtomicBool = AtomicBool::new(false);
        pub static RECORD: AtomicBool = AtomicBool::new(true);
        
        const N: usize = 6;
        pub type Measure = f32;
        pub type Reading = [Measure; N]; /// <--- 4 channel is total accel for now
        pub type Sample = crate::Sample<Measure, N>;

        #[embassy_executor::task]
        pub async fn task(  i2c: i2c::I2c<'static, I2C, Mode>,
                            hz: u64,
                        sensory: u8) {    
            //DISP.signal([None, None, None, Some("BMI160 task".try_into().unwrap())]);
            let address = SlaveAddr::default();
            let mut sensor 
                    = Bmi160::new_with_i2c(i2c, address);
            //DISP.signal([None, Some("BMI160 |==| I2C".try_into().unwrap()), None, None]);
            sensor.set_accel_power_mode(AccelerometerPowerMode::Normal).unwrap();
            //DISP.signal([None, Some("BMI160 accel".try_into().unwrap()), None, None]);
            sensor.set_gyro_power_mode(GyroscopePowerMode::Normal).unwrap();
            //DISP.signal([None, Some("BMI160 gyro".try_into().unwrap()), None, None]);
            //DISP.signal([None, None, None, Some("BMI160 set".try_into().unwrap())]);
            let mut ticker = Ticker::every(Duration::from_hz(hz));
            //DISP.signal([None, None, None, Some("BMI160 ticks".try_into().unwrap())]);
            //let mut reading: Reading;
            //let mut result: Sample;
            READY.store(true, ORD);
            loop {
                ticker.next().await;
                if RECORD.load(ORD){
                    DISP.signal([None, None, None, Some("BMI160 ...".try_into().unwrap())]);
                    let data = sensor.data(SensorSelector::new().accel().gyro()).unwrap();
                    let acc = data.accel.unwrap();
                    let gyr = data.gyro.unwrap();
                    DISP.signal([None, None, None, Some("BMI160     ...".try_into().unwrap())]);
                    let reading = [ acc.x.into(), acc.y.into(), acc.z.into(),
                                gyr.x.into(), gyr.y.into(), gyr.z.into()];
                    let sample = Sample{time: Instant::now(),
                                    sensory: sensory, 
                                    read: reading.into()};
                    SINK.send(sample.into()).await;
                    /*log::info!("{},{},{},{},{},{},{},{},,",
                        result.time.as_micros(),
                        sensory,
                        result.read[0],
                        result.read[1],
                        result.read[2],
                        result.read[3],
                        result.read[4],
                        result.read[5]);*/
                    };
                }
            }
        }



pub mod yirt_max {
    use super::*;
    use max3010x::{Max3010x, Led, SampleAveraging};
    use hal::peripherals::I2C0 as I2C;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);
    pub type Reading = [u32; 8]; /// <--- 4 channel is total accel for now
    pub type Measure = SensorResult<Reading>;

    #[embassy_executor::task]
    pub async fn task(  i2c: i2c::I2c<'static, I2C, Mode>,
                        hz: u64,
                        sensory: u8) {
        // Sensor specific
        let mut sensor 
            = Max3010x::new_max30102(i2c).into_multi_led().unwrap();
        sensor.set_sampling_rate(max3010x::SamplingRate::Sps1600).unwrap();
        sensor.set_sample_averaging(SampleAveraging::Sa16).unwrap();
        sensor.set_pulse_amplitude(Led::All, 15).unwrap();
        sensor.enable_fifo_rollover().unwrap();
        sensor.wake_up().unwrap();

        let mut data: [u32; 1] = [0; 1];
        let _ = sensor.read_fifo(&mut data).unwrap();
        DISP.signal([None, None, None, Some("IRTmax can read".try_into().unwrap())]);
        // Ticker
        let mut ticker = Ticker::every(Duration::from_hz(hz));
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        loop {
            if RECORD.load(ORD){
                let mut reading = [0;1];
                let _ = sensor.read_fifo(&mut reading);
                
                let sample = Sample{sensory: sensory, 
                    time: Instant::now(), read: reading};
                SINK.send(sample.into()).await;
                /*log::info!("{},1,{},,,,,,,",
                    Instant::now().as_micros(),
                    reading[0]);*/
                ticker.next().await;
                };
                
            }
        }
    }

pub mod yirt { // MLX90614
    /* Sensor Generics */
    use super::*;
    use mlx9061x::{Mlx9061x, SlaveAddr};
    use embassy_time::{Duration, Ticker, Instant};
    
    // Generic result
    pub type Reading = [f32; 2];
    pub type Measure = SensorResult<Reading>;
    
    // I2C    
    use hal::i2c;
    use hal::peripherals::I2C0;
    

    /* control channels */
    pub use core::sync::atomic::Ordering;
    use core::sync::atomic::AtomicBool;
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);


    #[embassy_executor::task]
    pub async fn task(  i2c: i2c::I2c<'static, I2C0, i2c::Blocking>,
                        hz: u64,
                        sensory: u8) {    
        let address = SlaveAddr::default();
        let mut sensor = Mlx9061x::new_mlx90614(i2c, address, 5).unwrap();
        let mut ticker 
                = Ticker::every(Duration::from_hz(hz));
        //let mut reading: Reading;
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        loop {
            ticker.next().await;
            if RECORD.load(ORD){
                let obj_temp:f32 = sensor.object1_temperature().unwrap().into();
                let amb_temp:f32 = sensor.ambient_temperature().unwrap().into();
                let reading: Reading = [obj_temp.into(), amb_temp.into()];
                let sample = 
                    Sample{sensory: sensory, time: Instant::now(),
                            read: reading};
                SINK.send(sample.into()).await;
                /*log::info!("{},T,{},{}", 
                    result.time.as_micros(),
                    result.reading[0],
                    result.reading[1]);*/
                };
            }
        }
    }

pub mod yco2 {
    use super::*;
    use hal::peripherals::I2C0;
    use scd4x;

    /* control channels */
    pub static READY: AtomicBool = AtomicBool::new(false);
    pub static RECORD: AtomicBool = AtomicBool::new(false);

    // Generic result
    pub type Reading = [f32; 3]; 
    pub type Measure = SensorResult<Reading>;

    #[embassy_executor::task]
    pub async fn task(  i2c: i2c::I2c<'static, I2C0, Mode>, sensory: u8) { 
        //DISP.signal([None, None, None, Some("CO2 start".try_into().unwrap())]);        
        let mut sensor = scd4x::Scd4x::new(i2c, time::Delay);
        //sensor.wake_up(); <---- This fails
        sensor.stop_periodic_measurement().unwrap();
        match sensor.reinit() {
            Ok(_) => {},
            Err(_) => {DISP.signal([None, None, None, Some("Reinit failed".try_into().unwrap())]);
                        return},
        }
        //DISP.signal([None, None, None, Some("CO2 init".try_into().unwrap())]);
        let mut ticker = Ticker::every(Duration::from_secs(5));
        //let mut result: SensorResult<Reading>;
        READY.store(true, ORD);
        //DISP.signal([None, None, None, Some("CO2 ticking".try_into().unwrap())]);
        loop {
            if RECORD.load(ORD){
                match sensor.measure_single_shot_non_blocking() {
                    Err(_) => {DISP.signal([None, None, None, Some("CO2 prep failed".try_into().unwrap())]);},
                    Ok(_) => {
                        ticker.next().await;
                        match sensor.measurement() {
                            Err(_) => {DISP.signal([None, None, None, Some("CO2 read failed".try_into().unwrap())]);},
                            Ok(raw) => {
                                let reading: Reading = [raw.co2 as f32, raw.humidity as f32, raw.temperature as f32];
                                let sample = Sample{sensory: sensory, time: Instant::now(), read: reading};
                                SINK.send(sample.into()).await;
                                //DISP.signal([None, None, None, Some("CO2 read".try_into().unwrap())]);
                                /*log::info!("{},{},{},{},{},,,,,",
                                    result.time.as_micros(),
                                    sensory,
                                    result.reading[0],
                                    result.reading[1],
                                    result.reading[2],);*/
                                //DISP.signal([None, None, None,Some("CO2 sent".try_into().unwrap())]);
                            },
                        };
                        
                        }
                    };
                
                };               
            }
        }
    }

