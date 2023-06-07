#![no_std]

use panic_probe as _;
use defmt_rtt as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

