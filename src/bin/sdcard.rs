//! This example shows how to communicate asynchronous using i2c with external chips.
//!
//! Example written for the [`MCP23017 16-Bit I2C I/O Expander with Serial Interface`] chip.
//! (https://www.microchip.com/en-us/product/mcp23017)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Duration, Ticker};

use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::{PIN_4, PIN_5, I2C0};
use embassy_rp::bind_interrupts;
bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

use itoa;

/* SD card */

#[embassy_executor::task]
async fn sdcard_task() {

}



use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;

static MESG: Signal<ThreadModeRawMutex, i32> 
    = Signal::new();

use embedded_sdmmc as sd;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    // Build an SD Card interface out of an SPI device, a chip-select pin and a delay object
    let sdcard = sd::SdCard::new(sdmmc_spi, sdmmc_cs, delay);
    // Get the card size (this also triggers card initialisation because it's not been done yet)
    let _sdc_size = sdcard.num_bytes()?;
    // Now let's look for volumes (also known as partitions) on our block device.
    // To do this we need a Volume Manager. It will take ownership of the block device.
    let mut volume_mgr = embedded_sdmmc::VolumeManager::new(sdcard, time_source);
    // Try and access Volume 0 (i.e. the first partition).
    // The volume object holds information about the filesystem on that volume.
    // It doesn't hold a reference to the Volume Manager and so must be passed back
    // to every Volume Manager API call. This makes it easier to handle multiple
    // volumes in parallel.
    let mut volume0 = volume_mgr.get_volume(embedded_sdmmc::VolumeIdx(0))?;
    // Open the root directory (passing in the volume we're using).
    let root_dir = volume_mgr.open_root_dir(&volume0)?;
    // Open a file called "MY_FILE.TXT" in the root directory
    let mut my_file = volume_mgr.open_file_in_dir(
        &mut volume0,
        &root_dir,
        "MY_FILE.TXT",
        embedded_sdmmc::Mode::ReadOnly,
    )?;
    // Print the contents of the file
/*    while !my_file.eof() {
        let mut buffer = [0u8; 32];
        let num_read = volume_mgr.read(&volume0, &mut my_file, &mut buffer)?;
        for b in &buffer[0..num_read] {
            print!("{}", *b as char);
        }
    }*/
    volume_mgr.close_file(&volume0, my_file)?;
    volume_mgr.close_dir(&volume0, root_dir);
    

}
