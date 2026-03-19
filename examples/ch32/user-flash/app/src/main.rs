//! Example application for the user-flash bootloader.
//!
//! This app occupies the upper 8KB of user flash.
//! The `boot_request.x` linker script reserves 4 bytes at the start of RAM for
//! the boot request word, shared with the bootloader.
//!
//! To request a firmware update from the app, call `client.request_update()`.
//! Without the `system-flash` feature, this writes a magic word to RAM and
//! triggers a soft reset back into the bootloader.

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_halt as _;

use tinyboot_ch32_app::traits::BootClient;

#[qingke_rt::entry]
fn main() -> ! {
    // Confirm successful boot to the bootloader's trial-boot FSM.
    // This advances the boot state from Validating -> Confirmed.
    // Safe to call unconditionally; it's a no-op if not in Validating state.
    let mut client = tinyboot_ch32_app::BootClient::default();
    client.confirm();

    defmt::info!("Hello from app!");

    loop {
        core::hint::spin_loop();
    }
}
