pub use super::*;
pub use core::fmt::Write;

//pub type Ytf = Sample<[Option<f32>; 8]>; // standard transport format
type YtfLine = Vec<u8, 256>;

trait YtfSend{
    fn msg_csv(&self) -> Result<YtfLine, core::fmt::Error>;
    fn msg_bin(&self) -> Result<YtfLine, core::fmt::Error>;
}

impl YtfSend for Ytf {
    fn msg_csv(&self) -> Result<YtfLine, core::fmt::Error>{
        let mut msg: YtfLine = Vec::new();
        match core::write!(&mut msg, "{}", self) {
            Ok(_) => return Ok(msg),
            Err(e) => return Err(e)
       }
    }

    fn msg_bin(&self) -> Result<YtfLine, core::fmt::Error>{
        todo!()
    }
}



pub mod bsu {
    use super::*;
    use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
    use hal::bind_interrupts;
    use hal::peripherals::USB;
    use hal::usb::{Driver, InterruptHandler};
    use log::LevelFilter;
    use embassy_usb_logger::*;

    pub static SINK: Channel<RawMutex, Ytf, 3> = Channel::new();

    #[embassy_executor::task]
    pub async fn logger_task(usb: USB, level: LevelFilter) {
        bind_interrupts!(struct Irqs {
            USBCTRL_IRQ => InterruptHandler<USB>;
        }); 
        let driver = Driver::new(usb, Irqs);
        run!(1024, level, driver);
    }

    #[embassy_executor::task]
    pub async fn task() {
        loop {
            let sample: Ytf = SINK.receive().await;
            log::info!("{}", sample);
            //time::Timer::after_nanos(500).await;
        }
    }

}

