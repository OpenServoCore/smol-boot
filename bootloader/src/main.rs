#![no_std]
#![no_main]

use qingke_rt::entry;

#[cfg(not(rtt_log))]
use panic_halt as _;

#[cfg(rtt_log)]
use defmt_rtt as _;

use ch32_iap_core::{BootControl, log_error, log_info};

#[cfg(rtt_log)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    log_error!("panic");
    loop {}
}

#[entry]
fn main() -> ! {
    log_info!("Bootloader started");

    let boot = BootControl::default();

    if boot.should_boot_app() {
        log_info!("Jumping to application");
        unsafe { boot.jump_to_app() }
    }

    log_info!("Entering bootloader mode");

    loop {}
}
