# YLab Edge Go

In a sister project we have introduced the [YLab](https://github.com/schmettow/ylab) for building interactive sensor recording devices. 
Using CircuitPython, developing sensors for everyday research never was easier.

The purpose of *YLab Edge* is to follow YLab in spirit, but improve on what Ylab lacks the most, and that is: speed! 
Highest achieved readings with YLab are in the range of 250 SPS, which is enough for many applications, 
but is insufficient for large sensor arrays with high sample rates, e.g. motion capture or EEG.

The solution is to re-implement the YLab API in the systems programming language [Rust](https://www.rust-lang.org/). 
It is expected that this will trade around 20% ease-of-use for a performance improvement anywhere in the range between 2x and 200x.

*YLab Edge Go* is the version to use for **RP2040** microcontrollers and currently reaches sample rates up to 3.000 SPS. Currently, Go is also the only version to support interaction via buttons, Led and Oled.

## Current status

Currenty, the following devices are implemented using [Embassy]: https://embassy.dev/. All devices are running a separate task, which can be distributed to both cores of the RP2040.

+ LED
+ 1 long-short button
+ on-board ADC channels
+ ADS1015/ADS1115 external ADC controller
+ SSD1306 Oled display
+ TCA9548A I2C hub with 8 channels
+ TSM6DS33 6-DoF acceleration sensor
+ SCD40 air quality sensor (humidity, temp, CO2)

## Installing from binary

The Pico MPU board has a very simple way of flashing new firmware onto it:

1. Choose a firmware binary (below) that matches your system.
1. Download desired the firmware binary (e.g. `ylab_dg.uf2`)
1. Connect the board via USB to your computer
1. Get the Pico into boot mode: Press Run and Boot buttons simultaneously, let go of Run, then let go of Boot. The boot drive should appear on your computer (e.g. ()`RPI-RP2 (D:)`).
1. Copy the `.uf2` file to the boot drive

If all went well, the boot drive will disappear and the Pico starts the new firmware. You can now download and use Ystudio [Ystudio](../ystudio-zero/) to view and capture the data produced by YLab Edge.

## Available firmware binaries

**YLab DG** is the basic version, reading the four built in ADCs (bank 0). [Download](uf2/ylab_dg.uf2)

**YLab Motion** uses a TCA9548 I2C bridge on Grove port 1 (Pins 0/1) with three TSM6DS33 motion sensors attached (bank 1). It also puts out built-in ADC (bank 0) [Download](uf2/ylab_motion.uf2)

**YLab Stress** runs a Scd40 CO2/temp/humidity sensor on Grove 5 (Pins 8/9)[Download](uf2/ylab_stress.uf2)


## Installing from source

All code in this crate is currently developed for 
a `Cytron Maker Pi Pico`. *Note* that the MP Pico is similar to the robotics-oriented Maker Pi RP2040, but not the same. (See [here](https://github.com/9names/makerpi_rp2040), if you have the latter).

To compile the latest version of YLab Edge:

+ Install Rust and Cargo on your system (Rustup)
+ clone this repository (e.g. in VSC)
+ connect the MP Pico via USB
+ Find the BOOT button (on the green part) and the RUN button (above the three push buttons) 
+ Simultaneously, push both buttons, release RUN, release BOOT. Now the Pico is in flash mode.
+ open a terminal in the ylab-edge folder and type:

```console
$ cargo run --bin ylab_dg
```
If you get an error about not being able to find `elf2uf2-rs`, try:

```console
$ cargo install elf2uf2-rs
```
then try repeating the `cargo run` command above.



`SPDX-License-Identifier: Apache-2.0 OR MIT`

