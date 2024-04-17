#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};
use embassy_rp as hal;
use embassy_time as time;
use embassy_executor::Executor;

use time::{Duration, Ticker, Instant, Delay};
use defmt::*;

/// The following code initializes the second stack, plus 
/// two heaps
use hal::multicore::{Stack, spawn_core1};
static mut CORE1_STACK: Stack<4096> = Stack::new();
use static_cell::StaticCell;
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

use hal::i2c::{self, Config};
use hal::peripherals::{I2C0, I2C1};
use hal::bind_interrupts;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});



#[cortex_m_rt::entry]
fn init() -> ! {
    let p = hal::init(Default::default());
    // Second core running MPU
    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 
            = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner|{
            let i2c = i2c::I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, Config::default());
            spawner.spawn(task(i2c, 2, 1)).unwrap();
        })
    });

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        let i2c = i2c::I2c::new_async(p.I2C1, p.PIN_3, p.PIN_2, Irqs, Config::default());
        spawner.spawn(task(i2c, 4, 1)).unwrap();
    }) // <-- don't put a semicol here. Never-ending(!)
}

use embedded_hal::i2c::I2c;
use mpu6886 as mpu;
use mpu::device::{AccelRange, GyroRange};

#[embassy_executor::task]
async fn task ( i2c:  impl I2c + 'static,
                hz: u64,
                sensory: u8) { 
    let mut delay = Delay{}; // <- curly brackets
    let mut sensor = mpu::Mpu6886::new(i2c);

    sensor.init(&mut delay).unwrap();
    sensor.set_accel_range(AccelRange::G2).unwrap();
    sensor.set_gyro_range(GyroRange::D250).unwrap();

    let mut ticker = Ticker::every(Duration::from_hz(hz));
    
    loop {
        ticker.next().await;
        let acc: [f32; 3] = sensor.get_acc().unwrap().into();
        let gyr: [f32; 3] = sensor.get_gyro().unwrap().into();
        let reading: [f32; 6] = [acc[0], acc[1], acc[2], gyr[0], gyr[1], gyr[2]];                
        println!("{},{},{}", Instant::now(), sensory, reading[0]);
        }
    }

