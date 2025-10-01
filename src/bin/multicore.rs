//! This example shows how to send messages between the two cores in the RP2040 chip.
//!
//! The LED on the RP Pico W board is connected differently. See wifi_blinky.rs.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIN_25;
use embassy_time::{Duration, Timer};

/// For running multicore, we need Executor (not just spawner)
use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};

/// The following code initializes the second stack, plus 
/// two heaps
static mut CORE1_STACK: Stack<4096> = Stack::new();
use static_cell::StaticCell;
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

/// Inter-core communication requires a special raw mutex.
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
static CHANNEL: Channel<CriticalSectionRawMutex, LedState, 1> = Channel::new();

enum LedState {
    On,
    Off,
}

/// # Main
/// 
/// The main program consists of three twasks on two cores: 
/// 
/// 1.  Core 0 is running a recurrent timer that sends On and Off commands to a channel. 
/// 2.  Core 0 is running the usb logger
/// 3.  Core 1 runs a task that receives the commands via the channel, 
///     operates the Led and sends a confirmation to the logger. 

use ylab::ytfk::bsu as ybsu;
#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(p.PIN_25, Level::Low);
    let usb = p.USB;

    spawn_core1(p.CORE1, unsafe { &mut CORE1_STACK }, move || {
        let executor1 = EXECUTOR1.init(Executor::new());
        executor1.run(|spawner| unwrap!(spawner.spawn(core1_task(led))));
    });

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {   unwrap!(spawner.spawn(core0_task()));
                                         unwrap!(spawner.spawn(ybsu::task(usb)));
                                            });
}

#[embassy_executor::task]
async fn core0_task() {
    log::info!("Hello from core 0");
    loop {
        CHANNEL.send(LedState::On).await;
        Timer::after(Duration::from_millis(100)).await;
        CHANNEL.send(LedState::Off).await;
        Timer::after(Duration::from_millis(400)).await;
    }
}


#[embassy_executor::task]
async fn core1_task(mut led: Output<'static, PIN_25>) {
    log::info!("Hello from core 1");
    loop {
        match CHANNEL.recv().await {
            LedState::On => {led.set_high();
                             log::info!("On")},
            LedState::Off => {led.set_low();
                              log::info!("Off")},
            
        }
    }
}

/* mod ylab_bsu {
    /* USB Logging */
    use embassy_rp::bind_interrupts;
    use embassy_rp::peripherals::USB;
    use embassy_rp::usb::{Driver, InterruptHandler};
    use log::{LevelFilter};
    use embassy_usb_logger;

    #[embassy_executor::task]
    pub async fn task(usb: USB) {
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

*/