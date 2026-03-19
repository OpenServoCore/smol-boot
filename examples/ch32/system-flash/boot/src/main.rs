//! System-flash bootloader example for CH32V003.
//!
//! The bootloader lives in the 1920-byte system flash region, leaving the
//! entire 16KB user flash available for the application. Because of the tight
//! size constraint, defmt is not used here.
//!
//! The `system-flash` feature uses the hardware BOOT_MODE register to signal
//! boot requests across resets (no RAM reservation needed).
//! Boot metadata is stored in option bytes (0x1FFFF800).

#![no_std]
#![no_main]

use panic_halt as _;

use tinyboot_ch32_boot::{
    BaudRate, BootCtl, BootCtlConfig, BootMetaStore, Core, Duplex, Platform, Pull, Storage,
    StorageConfig, Usart, UsartConfig, UsartMapping,
};

// --- Flash layout ---

const APP_BASE: u32 = 0x0800_0000;
const APP_SIZE: usize = 16 * 1024;

#[unsafe(export_name = "main")]
fn main() -> ! {
    tinyboot_ch32_boot::flash_unlock();

    // Configure USART transport for firmware updates.
    // Adjust mapping, pins, and baud rate to match your hardware.
    let transport = Usart::new(&UsartConfig {
        duplex: Duplex::Full,
        baud: BaudRate::B115200,
        pclk: 8_000_000, // HSI default clock, no PLL
        mapping: UsartMapping::Usart1Remap0, // TX=PD5, RX=PD6 (default)
        rx_pull: Pull::None,
        tx_en: None,
    });

    let storage = Storage::new(StorageConfig {
        app_base: APP_BASE,
        app_size: APP_SIZE,
    });
    let boot_meta = BootMetaStore::default();
    let ctl = BootCtl::new(BootCtlConfig {});

    let platform = Platform::new(transport, storage, boot_meta, ctl);
    Core::new(platform).run();
}
