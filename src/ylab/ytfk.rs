pub use crate::*;

/* YLab transport formats  */
pub mod bsu {
    use super::*;
    /* USB Logging */
    use hal::bind_interrupts;
    use hal::peripherals::USB;
    use hal::usb::{Driver, InterruptHandler};
    use log::LevelFilter;
    use embassy_usb_logger;

    /* The task of saving data from multiple sources has the exact same straucture
    as the logging task.*/
    #[embassy_executor::task]
    pub async fn task(usb: USB) {
        bind_interrupts!(struct Irqs {
            USBCTRL_IRQ => InterruptHandler<USB>;
        }); 

        let driver = 
            Driver::new(usb, Irqs);
        // This is copied from the crate. The run! macro is using set_max_level, where the racy version is needed.
        static LOGGER: embassy_usb_logger::UsbLogger<2048> = 
            embassy_usb_logger::UsbLogger::new();
        unsafe {
            let _ = log::set_logger_racy(&LOGGER)
                    .map(|()| log::set_max_level_racy(LevelFilter::Info));
        };
        let _ = LOGGER.run(&mut embassy_usb_logger::LoggerState::new(), driver)
                .await;
        }

        
        /* Main
        use embassy_executor::Spawner;
        use embassy_time::{Duration, Timer};
        use {defmt_rtt as _, panic_probe as _};
        #[embassy_executor::main]
        async fn main(spawner: Spawner) {
        let p = embassy_rp::init(Default::default());
        spawner.spawn(task(p.USB)).unwrap();
    
        let mut counter = 0;
        loop {
            counter += 1;
            log::info!("Tick {}", counter);
            Timer::after(Duration::from_secs(1)).await;
        }
    } */
}

