#![allow(dead_code)]
use core::sync::atomic::{Ordering, fence};

const KEY1: u32 = 0x4567_0123;
const KEY2: u32 = 0xCDEF_89AB;

/// All flash addresses passed to FlashWriter methods must use FPEC
/// programming addresses (0x0800_0000-based for user flash,
/// 0x1FFFF000-based for system flash).
fn flash() -> ch32_metapac::flash::Flash {
    ch32_metapac::FLASH
}

fn wait_busy() {
    while flash().statr().read().bsy() {}
}

fn unlock_keys() {
    flash().keyr().write(|w| w.set_keyr(KEY1));
    fence(Ordering::SeqCst);
    flash().keyr().write(|w| w.set_keyr(KEY2));
    fence(Ordering::SeqCst);
}

fn unlock_fast() {
    flash().modekeyr().write(|w| w.set_modekeyr(KEY1));
    fence(Ordering::SeqCst);
    flash().modekeyr().write(|w| w.set_modekeyr(KEY2));
    fence(Ordering::SeqCst);
}

fn unlock_boot() {
    flash().boot_modekeyp().write(|w| w.set_modekeyr(KEY1));
    fence(Ordering::SeqCst);
    flash().boot_modekeyp().write(|w| w.set_modekeyr(KEY2));
    fence(Ordering::SeqCst);
}

/// RAII guard for flash programming. Unlocks on creation, locks on drop.
pub struct FlashWriter;

impl FlashWriter {
    /// Unlock for standard operations on user flash (KEYR).
    pub fn standard() -> Self {
        unlock_keys();
        Self
    }

    /// Unlock for fast operations on user flash (KEYR + MODEKEYR).
    pub fn fast() -> Self {
        unlock_keys();
        unlock_fast();
        Self
    }

    /// Unlock for operations on system flash (KEYR + MODEKEYR + BOOT_MODEKEYP).
    pub fn system() -> Self {
        unlock_keys();
        unlock_fast();
        unlock_boot();
        Self
    }

    pub fn check_wrprterr(&self) -> bool {
        let statr = flash().statr().read();
        if statr.wrprterr() {
            flash().statr().modify(|w| w.set_wrprterr(true));
            return true;
        }
        if statr.eop() {
            flash().statr().modify(|w| w.set_eop(true));
        }
        false
    }

    /// Standard halfword (2-byte) write. Works on user and system flash.
    pub fn write_halfword(&self, addr: u32, value: u16) {
        flash().ctlr().modify(|w| w.set_pg(true));
        fence(Ordering::SeqCst);
        unsafe { core::ptr::write_volatile(addr as *mut u16, value) };
        wait_busy();
        flash().ctlr().modify(|w| w.set_pg(false));
    }

    /// Fast 64-byte page erase. Works on user and system flash.
    /// Note: standard 1K erase (PER) does NOT work on system flash.
    pub fn erase_page(&self, addr: u32) {
        flash().ctlr().modify(|w| w.set_page_er(true));
        fence(Ordering::SeqCst);
        flash().addr().write(|w| w.set_addr(addr));
        fence(Ordering::SeqCst);
        flash().ctlr().modify(|w| w.set_strt(true));
        wait_busy();
        flash().ctlr().modify(|w| w.set_page_er(false));
    }

    /// Fast 64-byte page write. Works on user and system flash.
    pub fn write_page(&self, addr: u32, data: &[u8]) {
        let prog_addr = addr;

        flash().ctlr().modify(|w| w.set_page_pg(true));
        flash().ctlr().modify(|w| w.set_bufrst(true));
        wait_busy();
        flash().ctlr().modify(|w| w.set_page_pg(false));

        let mut ptr = prog_addr as *mut u32;
        for chunk in data.chunks_exact(4) {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            flash().ctlr().modify(|w| w.set_page_pg(true));
            unsafe { core::ptr::write_volatile(ptr, word) };
            flash().ctlr().modify(|w| w.set_bufload(true));
            wait_busy();
            flash().ctlr().modify(|w| w.set_page_pg(false));
            ptr = unsafe { ptr.add(1) };
        }

        flash().ctlr().modify(|w| w.set_page_pg(true));
        flash().addr().write(|w| w.set_addr(prog_addr));
        flash().ctlr().modify(|w| w.set_strt(true));
        wait_busy();
        flash().ctlr().modify(|w| w.set_page_pg(false));
    }
}

impl Drop for FlashWriter {
    fn drop(&mut self) {
        flash().ctlr().modify(|w| {
            w.set_lock(true);
            w.set_flock(true);
        });
    }
}

pub fn is_boot_mode() -> bool {
    flash().statr().read().boot_mode()
}

pub fn set_boot_mode(mode: bool) {
    if flash().statr().read().boot_lock() {
        unlock_boot();
    }
    flash().statr().modify(|w| w.set_boot_mode(mode));
}
