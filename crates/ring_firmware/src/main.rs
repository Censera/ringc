#![no_std]
#![no_main]

mod clock;
mod controls;
mod rtc;
mod slot;
mod watchdog;

use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    loop {}
}
