const KEY1: u32 = 0x4567_0123;
const KEY2: u32 = 0xCDEF_89AB;

fn flash() -> ch32_metapac::flash::Flash {
    ch32_metapac::FLASH
}

fn wait_busy() {
    while flash().statr().read().bsy() {}
}

/// Unlock flash controller for all operations (KEYR + OBKEYR).
pub fn unlock() {
    flash().keyr().write(|w| w.set_keyr(KEY1));
    flash().keyr().write(|w| w.set_keyr(KEY2));
    flash().modekeyr().write(|w| w.set_modekeyr(KEY1));
    flash().modekeyr().write(|w| w.set_modekeyr(KEY2));
    flash().obkeyr().write(|w| w.set_optkey(KEY1));
    flash().obkeyr().write(|w| w.set_optkey(KEY2));
}

/// Lock flash controller.
pub fn lock() {
    flash().ctlr().modify(|w| {
        w.set_lock(true);
        w.set_flock(true);
    });
}

/// Flash/OB writer. Thin wrapper selecting CTLR bit positions.
/// Requires `unlock()` to have been called first.
pub struct FlashWriter {
    erase_pos: u8,
    write_pos: u8,
}

impl FlashWriter {
    /// Writer for user flash (PG / PAGE_ER).
    pub const fn standard() -> Self {
        Self {
            erase_pos: 17,
            write_pos: 0,
        }
    }

    /// Writer for option bytes (OBPG / OBER).
    pub const fn ob() -> Self {
        Self {
            erase_pos: 5,
            write_pos: 4,
        }
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

    /// Halfword (2-byte) write.
    pub fn write(&self, addr: u32, value: u16) {
        let pos = self.write_pos as usize;
        flash().ctlr().modify(|w| w.0 |= 1 << pos);
        unsafe { core::ptr::write_volatile(addr as *mut u16, value) };
        wait_busy();
        flash().ctlr().modify(|w| w.0 &= !(1 << pos));
    }

    /// Erase (64-byte page for flash, full OB erase for option bytes).
    pub fn erase(&self, addr: u32) {
        let pos = self.erase_pos as usize;
        flash().ctlr().modify(|w| w.0 |= 1 << pos);
        flash().addr().write(|w| w.set_addr(addr));
        flash().ctlr().modify(|w| w.set_strt(true));
        wait_busy();
        flash().ctlr().modify(|w| w.0 &= !(1 << pos));
    }
}

pub fn is_boot_mode() -> bool {
    flash().statr().read().boot_mode()
}

pub fn set_boot_mode(mode: bool) {
    flash().boot_modekeyp().write(|w| w.set_modekeyr(KEY1));
    flash().boot_modekeyp().write(|w| w.set_modekeyr(KEY2));
    flash().statr().write(|w| w.set_boot_mode(mode));
}
