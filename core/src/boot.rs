unsafe extern "C" {
    static __APP_ADDR: u8;
    static __APP_SIZE: u8;
}

// Application Magic located at the beginning of the application's binary.
const APP_MAGIC: u32 = 0xC0FF_EEEE;

// Application origin address from linker symbol.
fn app_addr() -> *const u32 {
    &raw const __APP_ADDR as *const u32
}

// Application flash area size
fn app_size() -> usize {
    &raw const __APP_SIZE as usize
}

// Utilize Ch32's Optional User Data DATA0 to store boot request flag.
// This is a 16-bit register, so we need to read both bytes with
// upper as inverse, and lower as data to ensure data integrity,
// thus the u16 pointer.
const OB_DATA0: *const u16 = 0x1FFFF804 as *const u16;

#[derive(Debug, PartialEq)]
enum AppState {
    Valid,
    Invalid,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootMode {
    Application = 0x00,
    Bootloader = 0x01,
}

pub struct BootControl {
    app_state: AppState,
    mode: Option<BootMode>,
}

impl AppState {
    fn new(addr: *const u32) -> Self {
        let first_word = unsafe { core::ptr::read_volatile(addr) };

        if first_word == APP_MAGIC {
            AppState::Valid
        } else {
            AppState::Invalid
        }
    }
}

impl BootMode {
    fn new(addr: *const u16) -> Option<Self> {
        read_ob(addr).and_then(|b| match b {
            0x00 => Some(BootMode::Application),
            0x01 => Some(BootMode::Bootloader),
            _ => None,
        })
    }
}

impl BootControl {
    pub fn new(app_addr: *const u32, ob_addr: *const u16) -> Self {
        Self {
            app_state: AppState::new(app_addr),
            mode: BootMode::new(ob_addr),
        }
    }

    pub fn should_boot_app(&self) -> bool {
        if self.app_state == AppState::Invalid {
            return false;
        }

        match self.mode {
            None => true, // fresh chip, valid app, go for it
            Some(BootMode::Bootloader) => false,
            Some(BootMode::Application) => true,
        }
    }

}

impl Default for BootControl {
    fn default() -> Self {
        Self::new(app_addr(), OB_DATA0)
    }
}

impl BootControl {
    /// Jump to the application. Does not return.
    ///
    /// # Safety
    /// Caller must ensure a valid application is present at the app address.
    pub unsafe fn jump_to_app(&self) -> ! {
        unsafe {
            let entry = app_addr().add(1); // skip past 4-byte magic word
            let entry: unsafe extern "C" fn() -> ! = core::mem::transmute(entry);
            entry()
        }
    }
}

/// Read the u16 pointer from the Optional User Data register via MMIO.
/// If upper byte is not the inverse of the lower byte, return None to represent invalid data.
/// Otherwise, return the lower byte as value.
fn read_ob(addr: *const u16) -> Option<u8> {
    let raw = unsafe { core::ptr::read_volatile(addr) };
    let data = raw as u8;
    let inv = (raw >> 8) as u8;
    (data == !inv).then_some(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    // read_ob tests

    #[test]
    fn read_ob_valid_zero() {
        let raw: u16 = 0xFF00; // data=0x00, inv=0xFF
        assert_eq!(read_ob(&raw as *const u16), Some(0x00));
    }

    #[test]
    fn read_ob_valid_one() {
        let raw: u16 = 0xFE01; // data=0x01, inv=0xFE
        assert_eq!(read_ob(&raw as *const u16), Some(0x01));
    }

    #[test]
    fn read_ob_valid_arbitrary() {
        let raw: u16 = 0x0AF5; // data=0xF5, inv=0x0A
        assert_eq!(read_ob(&raw as *const u16), Some(0xF5));
    }

    #[test]
    fn read_ob_erased_flash() {
        let raw: u16 = 0xFFFF; // never written
        assert_eq!(read_ob(&raw as *const u16), None);
    }

    #[test]
    fn read_ob_corrupted() {
        let raw: u16 = 0x1234; // invalid inverse
        assert_eq!(read_ob(&raw as *const u16), None);
    }

    // should_boot_app tests

    fn boot(app_state: AppState, mode: Option<BootMode>) -> BootControl {
        BootControl { app_state, mode }
    }

    #[test]
    fn no_magic_never_boots() {
        assert!(!boot(AppState::Invalid, None).should_boot_app());
        assert!(!boot(AppState::Invalid, Some(BootMode::Application)).should_boot_app());
        assert!(!boot(AppState::Invalid, Some(BootMode::Bootloader)).should_boot_app());
    }

    #[test]
    fn valid_app_fresh_chip_boots() {
        assert!(boot(AppState::Valid, None).should_boot_app());
    }

    #[test]
    fn valid_app_mode_application_boots() {
        assert!(boot(AppState::Valid, Some(BootMode::Application)).should_boot_app());
    }

    #[test]
    fn valid_app_mode_bootloader_stays() {
        assert!(!boot(AppState::Valid, Some(BootMode::Bootloader)).should_boot_app());
    }

    // AppState::new tests

    #[test]
    fn app_state_valid_magic() {
        let magic: u32 = APP_MAGIC;
        assert_eq!(AppState::new(&magic as *const u32), AppState::Valid);
    }

    #[test]
    fn app_state_wrong_magic() {
        let magic: u32 = 0xDEADBEEF;
        assert_eq!(AppState::new(&magic as *const u32), AppState::Invalid);
    }

    #[test]
    fn app_state_erased_flash() {
        let magic: u32 = 0xFFFFFFFF;
        assert_eq!(AppState::new(&magic as *const u32), AppState::Invalid);
    }

    #[test]
    fn app_state_zero() {
        let magic: u32 = 0x00000000;
        assert_eq!(AppState::new(&magic as *const u32), AppState::Invalid);
    }

    // BootMode::new tests

    #[test]
    fn boot_mode_application() {
        let raw: u16 = 0xFF00; // data=0x00
        assert_eq!(BootMode::new(&raw as *const u16), Some(BootMode::Application));
    }

    #[test]
    fn boot_mode_bootloader() {
        let raw: u16 = 0xFE01; // data=0x01
        assert_eq!(BootMode::new(&raw as *const u16), Some(BootMode::Bootloader));
    }

    #[test]
    fn boot_mode_erased() {
        let raw: u16 = 0xFFFF; // never written
        assert_eq!(BootMode::new(&raw as *const u16), None);
    }

    #[test]
    fn boot_mode_unknown_value() {
        let raw: u16 = 0xFD02; // data=0x02, valid inverse but unknown mode
        assert_eq!(BootMode::new(&raw as *const u16), None);
    }

    // BootControl::new tests

    #[test]
    fn boot_control_valid_app_fresh_chip() {
        let magic: u32 = APP_MAGIC;
        let ob: u16 = 0xFFFF; // never written
        let bc = BootControl::new(&magic as *const u32, &ob as *const u16);
        assert!(bc.should_boot_app());
    }

    #[test]
    fn boot_control_valid_app_mode_application() {
        let magic: u32 = APP_MAGIC;
        let ob: u16 = 0xFF00; // data=0x00
        let bc = BootControl::new(&magic as *const u32, &ob as *const u16);
        assert!(bc.should_boot_app());
    }

    #[test]
    fn boot_control_valid_app_mode_bootloader() {
        let magic: u32 = APP_MAGIC;
        let ob: u16 = 0xFE01; // data=0x01
        let bc = BootControl::new(&magic as *const u32, &ob as *const u16);
        assert!(!bc.should_boot_app());
    }

    #[test]
    fn boot_control_no_app_fresh_chip() {
        let magic: u32 = 0xFFFFFFFF;
        let ob: u16 = 0xFFFF;
        let bc = BootControl::new(&magic as *const u32, &ob as *const u16);
        assert!(!bc.should_boot_app());
    }

    #[test]
    fn boot_control_no_app_mode_application() {
        let magic: u32 = 0xFFFFFFFF;
        let ob: u16 = 0xFF00;
        let bc = BootControl::new(&magic as *const u32, &ob as *const u16);
        assert!(!bc.should_boot_app());
    }
}
