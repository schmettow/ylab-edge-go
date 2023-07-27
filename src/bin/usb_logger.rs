//! USB Logging
//!
//! This creates the possibility to send log::info/warn/error/debug! to USB serial port.
//! Can be used to improvise data streaming via USB 

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};


mod usb_logger {
    /* USB Logging */
    use embassy_rp::bind_interrupts;
    use embassy_rp::peripherals::USB;
    use embassy_rp::usb::{Driver, InterruptHandler};
    use log::{LevelFilter};
    use embassy_usb_logger;

    #[embassy_executor::task]
    pub async fn logger_task(usb: USB) {
        bind_interrupts!(struct Irqs {
            USBCTRL_IRQ => InterruptHandler<USB>;
        });

        let driver = 
            Driver::new(usb, Irqs);
        // This is copied from the crate. The run! macro is using set_max_level, where the racy version is needed.
        static LOGGER: embassy_usb_logger::UsbLogger<1024> = 
            embassy_usb_logger::UsbLogger::new();
        unsafe {
            let _ = log::set_logger_racy(&LOGGER)
                    .map(|()| log::set_max_level_racy(LevelFilter::Info));
        };
        let _ = LOGGER.run(&mut embassy_usb_logger::LoggerState::new(), driver)
                .await;
        }
}


/* MAIN */
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.spawn(usb_logger::logger_task(p.USB)).unwrap();
    
    let mut counter = 0;
    loop {
        counter += 1;
        log::info!("Tick {}", counter);
        Timer::after(Duration::from_secs(1)).await;
    }
}
