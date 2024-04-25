#![no_std]

pub use embassy_rp as hal;
pub use embassy_time as time;
use embedded_hal::i2c::Error;
pub use time::{Duration, Ticker, Instant, Delay};
pub use heapless::{Vec, String};
pub use embassy_sync::mutex::Mutex as Mutex;
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as RawMutex;
pub use embassy_sync::signal::Signal;
pub use embassy_sync::channel::Channel;

pub use core::sync::atomic::Ordering;
pub use core::sync::atomic::AtomicBool;
//pub static RLX: Ordering = Ordering::Relaxed;
pub static ORD: Ordering = Ordering::SeqCst;

/*use core::fmt;
use fmt::Write;*/


pub mod ysns; // Ylab sensors
pub mod yuio; // YLab UI Output
pub mod yuii; // YLab UI Input
pub mod ytfk; // YLab transfer formats & kodices

pub type Reading<M: Into<YtfType>, const N: usize> = [M; N];

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Sample<M, const N: usize>
    where M: Into<YtfType>
    {
    pub sensory: u8,
    pub time: Instant,
    pub read: Reading<M, N>,
}

pub const YTF_LEN: usize = 8;
pub type YtfType = f64;
pub type YtfMeasure = Option<YtfType>;
pub type YtfRead = [YtfMeasure; YTF_LEN];

pub struct Ytf {
    pub sensory: u8,
    pub time: Instant,
    pub read: YtfRead,
}

impl<M: Into<YtfType>, const N: usize> Into<Ytf> for Sample<M, N> {
    fn into(self) -> Ytf {
        let mut out: YtfRead = [None; YTF_LEN];
        for (i, r) in self.read.into_iter().enumerate() {
            out[i] = Some(r.into());
        }
        Ytf {
            sensory: self.sensory,
            time: self.time,
            read: out
        }
    }
}


impl core::fmt::Display for Ytf {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}, {}", self.time.as_micros(), self.sensory).unwrap();
        for r in self.read {
            match r {
                Some(v) => {
                    write!(f, ",{:.3}", v).unwrap();},
                None => {write!(f, ",").unwrap();}
            }
        }
        write!(f, "")
    }
}

#[derive(Debug)]
enum SensoryError{Init, Read}

trait Sensor<M, const N: usize> {
    fn prepare(&mut self) -> Result<(), SensoryError>;
    
    async fn read(&mut self) -> Result<Reading<M, N>, SensoryError>;
    
    async fn sample(&mut self, sensory: u8) -> Result<Sample<M, N>, SensoryError>
        where M: Into<YtfType> {
        let reading = self.read().await;
        match reading {
            Ok(reading) => Ok(Sample{sensory: sensory, time: Instant::now(), read: reading}),
            Err(E) => Err(SensoryError::Read),
        } 
    }
    
    async fn run(&mut self, sensory: u8, interval: Duration) -> ! 
        where M: Into<YtfType> {
        let mut ticker = Ticker::every(interval);
        loop{
            match self.sample(sensory).await {
                Ok(sample) => ytfk::bsu::SINK.send(sample.into()).await,
                Err(_) => {},
            }
            ticker.next().await;
        }
    }
}


