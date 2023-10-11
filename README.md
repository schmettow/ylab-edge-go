# YLab Edge

In a sister project we have introduced the [YLab](https://github.com/schmettow/ylab) for building interactive sensor recording devices. 
Using CircuitPython, developing sensors for everyday research never was easier.

The purpose of *YLab Edge* is to follow YLab in spirit, but improve on what Ylab lacks the most, and that is: speed! 
Highest achieved readings with YLab are in the range of 250 SPS, which is enough for many applications, 
but is insufficient for large sensor arrays with high sample rates, e.g. motion capture or EEG.

The solution is to re-implement the YLab API in the systems programming language [Rust](https://www.rust-lang.org/). 
It is expected that this will trade around 20% ease-of-use for a performance improvement anywhere in the range between 2x and 200x.

*YLab Edge Go* is the version to use for **RP2040** microcontrollers and currently is almost 4x faster 
Currently, Go is also the only version to support interaction via buttons, Led and Oled.

# Current status

Currenty, the following devices are implemented using [Embassy]: https://embassy.dev/. 
All devices are running in their own async task, which can be distributed to both cores of the RP2040.

+ LED
+ 1 long-short button
+ on-board ADC channels
+ ADS1015/ADS1115 ADC
+ SSD1306 Oled display

A test system with 

+ the built-in 4-channel ADC @ 800Hz
+ a 4-channel ADS1015 @ 120 Hz
+ a 4-channel ADS1115 @ 30 Hz

is able to produce a total sample rate of almost 4 kHz.

## Installing

All code in this crate is currently developed for 
a `Cytron Maker Pi Pico`. *Note* that the MP Pico is similar to the robotics-oriented Maker Pi RP2040, but not the same. (See [here](https://github.com/9names/makerpi_rp2040), if you have the latter).

To install the latest version of YLab Edge:

+ Install Rust and Cargo on your system
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

