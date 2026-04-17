//! Run-mode request persistence across reset.
//!
//! Exactly one variant is selected per build by `build.rs` cfgs:
//! - [`reg`]: flash BOOT_MODE register (system-flash, chips without `boot_pin`).
//! - [`ram`]: magic word in RAM (user-flash).
//! - [`gpio`]: [`ram`] plus a GPIO-driven BOOT0 circuit (system-flash + `boot_pin`).

core::cfg_select! {
    boot_req_reg => {
        mod reg;
        pub type Active = reg::RegRequest;
    }
    boot_req_gpio => {
        mod ram;
        mod gpio;
        pub type Active = gpio::GpioRequest;
    }
    boot_req_ram => {
        mod ram;
        pub type Active = ram::RamRequest;
    }
}
