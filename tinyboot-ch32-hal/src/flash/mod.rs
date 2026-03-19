#[cfg_attr(flash_v0, path = "v0.rs")]
mod family;

pub use family::*;

// OB boot metadata — shared across all CH32 families.
// OB layout: 8 config halfwords (16 B) + boot meta halfwords at +16.

const OB_BASE: u32 = 0x1FFFF800;
const META_OB_BASE: u32 = OB_BASE + 16;

/// OB meta address for state step-down.
pub const OB_STATE_ADDR: u32 = META_OB_BASE;
/// OB meta address for trials step-down.
pub const OB_TRIALS_ADDR: u32 = META_OB_BASE + 2;

fn read_ob_byte(addr: u32) -> u8 {
    unsafe { core::ptr::read_volatile(addr as *const u8) }
}

/// Read boot state byte from OB.
pub fn ob_boot_state() -> u8 {
    read_ob_byte(META_OB_BASE)
}

/// Read trials byte from OB.
pub fn ob_trials() -> u8 {
    read_ob_byte(META_OB_BASE + 2)
}

/// Read stored CRC16 from OB.
pub fn ob_checksum() -> u16 {
    u16::from_le_bytes([
        read_ob_byte(META_OB_BASE + 4),
        read_ob_byte(META_OB_BASE + 6),
    ])
}

/// Step down a single OB byte (1→0 bit clear). Returns new value or None if at floor.
pub fn ob_step_down(addr: u32, floor: u8) -> Option<u8> {
    let current = read_ob_byte(addr);
    if current <= floor {
        return None;
    }
    let next = current & (current >> 1);
    let writer = FlashWriter::ob();
    writer.write(addr, next as u16);
    Some(next)
}

/// Erase OB and rewrite with given state and checksum, preserving chip config.
/// Requires `unlock()` to have been called first.
#[allow(clippy::needless_range_loop)]
pub fn ob_refresh(state: u8, checksum: u16) {
    let mut buf = [0xFFu8; 12];
    for i in 0..8 {
        buf[i] = read_ob_byte(OB_BASE + (i as u32 * 2));
    }
    buf[8] = state;
    let cksum = checksum.to_le_bytes();
    buf[10] = cksum[0];
    buf[11] = cksum[1];

    let writer = FlashWriter::ob();
    writer.erase(OB_BASE);
    for (i, &byte) in buf.iter().enumerate() {
        if byte != 0xFF {
            writer.write(OB_BASE + (i as u32 * 2), byte as u16);
        }
    }
}
