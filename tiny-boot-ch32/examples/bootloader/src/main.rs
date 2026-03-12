#![no_std]
#![no_main]

use panic_halt as _;

#[cfg(feature = "defmt")]
use defmt_rtt as _;

use qingke_rt::entry;
use tiny_boot_ch32::Bootloader;

#[entry]
fn main() -> ! {
    let mut bl = Bootloader::default();
    bl.run();
}
