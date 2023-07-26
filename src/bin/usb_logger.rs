//! This example shows how to use USB (Universal Serial Bus) in the RP2040 chip.
//!
//! This creates the possibility to send log::info/warn/error/debug! to USB serial port.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use log::{LevelFilter};
use embassy_usb_logger;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    // This is copied from the crate. The run! macro is using set_mac_level, where the racy version is needed.
        static LOGGER: embassy_usb_logger::UsbLogger<1024> = embassy_usb_logger::UsbLogger::new();
        unsafe {
            let _ = log::set_logger_racy(&LOGGER)
                    .map(|()| log::set_max_level_racy(LevelFilter::Info));
        };
        let _ = LOGGER.run(&mut embassy_usb_logger::LoggerState::new(), driver).await;
    }

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let mut counter = 0;
    loop {
        counter += 1;
        log::info!("Tick {}", counter);
        Timer::after(Duration::from_secs(1)).await;
    }
}
