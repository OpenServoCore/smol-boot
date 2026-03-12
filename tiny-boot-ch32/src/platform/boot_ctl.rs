use tiny_boot::{traits::BootCtl as TBBootCtl, log_info};

use crate::common::*;
use crate::hal::pfic;

const BOOT_REQUEST_PTR: *mut u32 = RAM_BASE as *mut u32;
const BOOT_REQUEST_MAGIC: u32 = 0xB007_CAFE;

pub(crate) struct BootCtl;

impl BootCtl {
    pub fn new() -> Self {
        BootCtl {}
    }
}

impl TBBootCtl for BootCtl {
    fn jump_to_app(&self) -> ! {
        log_info!("Booting Application...");
        let ep = entry_point();
        unsafe { ep() };
    }

    fn system_reset(&mut self) -> ! {
        log_info!("Resetting...");
        pfic::system_reset(&ch32_metapac::PFIC);
    }

    fn take_boot_request(&mut self) -> bool {
        let val = unsafe { core::ptr::read_volatile(BOOT_REQUEST_PTR) };
        unsafe { core::ptr::write_volatile(BOOT_REQUEST_PTR, 0) };
        val == BOOT_REQUEST_MAGIC
    }
}

type EntryPoint = unsafe extern "C" fn() -> !;

fn entry_point() -> EntryPoint {
    unsafe { core::mem::transmute::<_, EntryPoint>(APP_PTR) }
}
