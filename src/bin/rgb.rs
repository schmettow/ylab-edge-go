

use rp2040_hal::clocks::init_clocks_and_plls;
 let clocks = init_clocks_and_plls(...);
 let pins = rp2040_hal::gpio::pin::bank0::Pins::new(...);

 let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
 let mut ws = Ws2812Direct::new(
     pins.gpio4.into_mode(),
     &mut pio,
     sm0,
     clocks.peripheral_clock.freq(),
 );

 // Then you will make sure yourself to not write too frequently:
 loop {
     use smart_leds::{SmartLedsWrite, RGB8};
     let color : RGB8 = (255, 0, 255).into();

     ws.write([color].iter().copied()).unwrap();
     delay_for_at_least_60_microseconds();
 };