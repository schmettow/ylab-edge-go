# YLab Edge

In a sister project we have introduced the [YLab](https://github.com/schmettow/ylab) for building interactive sensor recording devices. 
Using the high-level API in CircuitPython, developing sensors for everyday research never was easier.

The purpose of *YLab eDGe* is to follow YLab in spirit, but improve on what Ylab lacks the most, and that is: speed! Highest achieved readings with YLab are in the range of 250 SPS, which is enough for many applications, but is insufficient for large sensor arrays with high sample rates, e.g. motion capture or EEG.

The solution is to re-implement the YLab API in the systems programming language [Rust](https://www.rust-lang.org/). 
It is expected that this will trade around 20% ease-of-use for a performance improvement anywhere in the range between 2x and 200x. 

# Current status

You should include this crate if you are writing code that you want to run on
a `Cytron Maker Pi Pico`. *Note* that the MP Pico is similar to the robotics-oriented Maker Pi RP2040, but not the same. (See [here](https://github.com/9names/makerpi_rp2040), if you have the latter).

Currenty, the following devices are implemented using [Embassy]: https://embassy.dev/. All devices are running in their own async task.

+ LED
+ 1 long-short button
+ on-board ADC
+ ADS1115 ADC
+ SSD1306 Oled display

[rp2040-hal]: https://github.com/rp-rs/rp-hal/tree/main/rp2040-hal

## Installing examples

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

