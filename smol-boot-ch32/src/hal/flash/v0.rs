use core::sync::atomic::{Ordering, fence};

use super::Ch32FlashError;
use crate::hal::common::FLASH_WRITE_SIZE;

const KEY1: u32 = 0x4567_0123;
const KEY2: u32 = 0xCDEF_89AB;

pub(crate) fn unlock(regs: &ch32_metapac::flash::Flash) {
    regs.keyr().write(|w| w.set_keyr(KEY1));
    fence(Ordering::SeqCst);
    regs.keyr().write(|w| w.set_keyr(KEY2));
    fence(Ordering::SeqCst);

    regs.modekeyr().write(|w| w.set_modekeyr(KEY1));
    fence(Ordering::SeqCst);
    regs.modekeyr().write(|w| w.set_modekeyr(KEY2));
    fence(Ordering::SeqCst);
}

pub(crate) fn lock(regs: &ch32_metapac::flash::Flash) {
    regs.ctlr().modify(|w| {
        w.set_lock(true);
        w.set_flock(true);
    });
}

fn wait_busy(regs: &ch32_metapac::flash::Flash) {
    while regs.statr().read().bsy() {}
}

fn check_error(regs: &ch32_metapac::flash::Flash) -> Result<(), Ch32FlashError> {
    let statr = regs.statr().read();
    if statr.wrprterr() {
        regs.statr().modify(|w| w.set_wrprterr(true));
        return Err(Ch32FlashError::Protected);
    }
    if statr.eop() {
        regs.statr().modify(|w| w.set_eop(true));
    }
    Ok(())
}

pub(crate) fn erase_page(
    regs: &ch32_metapac::flash::Flash,
    addr: u32,
) -> Result<(), Ch32FlashError> {
    regs.ctlr().modify(|w| w.set_page_er(true));
    fence(Ordering::SeqCst);
    regs.addr().write(|w| w.set_addr(addr));
    fence(Ordering::SeqCst);
    regs.ctlr().modify(|w| w.set_strt(true));
    wait_busy(regs);
    regs.ctlr().modify(|w| w.set_page_er(false));
    check_error(regs)
}

/// Write a single FLASH_WRITE_SIZE (64-byte) page using fast page programming.
/// `addr` must be absolute and FLASH_WRITE_SIZE-aligned.
/// `data` must be exactly FLASH_WRITE_SIZE bytes.
pub(crate) fn write_page(
    regs: &ch32_metapac::flash::Flash,
    addr: u32,
    data: &[u8],
) -> Result<(), Ch32FlashError> {
    debug_assert_eq!(data.len(), FLASH_WRITE_SIZE);

    regs.ctlr().modify(|w| w.set_bufrst(true));
    wait_busy(regs);
    regs.ctlr().modify(|w| w.set_bufrst(false));

    let mut ptr = addr as *mut u16;
    for chunk in data.chunks_exact(2) {
        let half = u16::from_le_bytes([chunk[0], chunk[1]]);
        unsafe { core::ptr::write_volatile(ptr, half) };
        fence(Ordering::SeqCst);
        regs.ctlr().modify(|w| w.set_bufload(true));
        wait_busy(regs);
        ptr = unsafe { ptr.add(1) };
    }

    regs.ctlr().modify(|w| w.set_page_pg(true));
    fence(Ordering::SeqCst);
    regs.ctlr().modify(|w| w.set_strt(true));
    wait_busy(regs);
    regs.ctlr().modify(|w| w.set_page_pg(false));
    check_error(regs)
}
