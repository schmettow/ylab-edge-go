#![no_std]

pub use embassy_rp as hal;
pub use embassy_time as time;
pub use time::{Duration, Ticker, Instant, Delay};
pub use heapless::{Vec, String};
pub use embassy_sync::mutex::Mutex as Mutex;
pub use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as RawMutex;
pub use embassy_sync::signal::Signal;
pub use embassy_sync::channel::Channel;

pub use core::sync::atomic::Ordering;
pub use core::sync::atomic::AtomicBool;
pub static RLX: Ordering = Ordering::Relaxed;

/*use core::fmt;
use fmt::Display;
use fmt::Write;*/


pub mod ysns; // Ylab sensors
pub mod yuio; // YLab UI Output
pub mod yuii; // YLab UI Input
pub mod ytfk; // YLab transfer formats & kodices

#[derive(Debug,Eq, PartialEq)]
pub struct Sample<T> {
    pub sensory: u8,
    pub time: Instant,
    pub read: T,
}

