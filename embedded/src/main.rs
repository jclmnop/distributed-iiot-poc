#![no_std]
#![no_main]

use cortex_m_rt::entry;
use nb::block;

use embedded as _;

use nrf52840_dk_bsp::{
    hal::{
        prelude::*,
        timer::{self, Timer},
    },
    Board,
};

#[entry]
fn main() -> ! {
    let mut nrf52 = Board::take().unwrap();
    let mut timer = Timer::new(nrf52.TIMER0);
    let before = nrf52.UICR.approtect.read().bits();
    defmt::println!("Before: {}", before);
    nrf52.UICR.approtect.write(|w| unsafe {w.bits(0x5A)});
    let after = nrf52.UICR.approtect.read().bits();
    defmt::println!("After: {}", after);

    loop {
        nrf52.leds.led_2.enable();
        delay(&mut timer, 250_000);

        nrf52.leds.led_2.disable();
        nrf52.leds.led_3.enable();
        delay(&mut timer, 250_000);

        nrf52.leds.led_3.disable();
    }
}

fn delay<T: timer::Instance>(timer: &mut Timer<T>, cycles: u32) {
    timer.start(cycles);
    let _ = block!(timer.wait());
}