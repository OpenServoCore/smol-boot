#![no_std]

pub mod platform;

#[cfg(all(target_arch = "riscv32", feature = "rt"))]
mod rt;

pub use platform::{
    BaudRate, BootCtl, BootCtlConfig, BootMetaStore, Duplex, Storage, StorageConfig, TxEnConfig,
    Usart, UsartConfig,
};

// Re-exports so boot examples only need this one crate.
pub use tinyboot::Core;
pub use tinyboot::traits::boot::Platform;
pub use tinyboot_ch32_hal::flash::unlock as flash_unlock;
pub use tinyboot_ch32_hal::gpio::Pull;
pub use tinyboot_ch32_hal::{Pin, UsartMapping};
