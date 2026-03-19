use crate::protocol;
use crate::traits::BootState;
use crate::traits::boot::{BootCtl, BootMetaStore, Platform, Storage, Transport};

pub struct Core<const D: usize, T, S, B, C>
where
    T: Transport<D>,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    platform: Platform<D, T, S, B, C>,
}

impl<const D: usize, T, S, B, C> Core<D, T, S, B, C>
where
    T: Transport<D>,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    pub fn new(platform: Platform<D, T, S, B, C>) -> Self {
        Core { platform }
    }

    pub fn run(mut self) -> ! {
        log_info!("Bootloader started");

        match self.check_boot_state() {
            Ok(false) => self.platform.ctl.system_reset(false),
            Ok(true) | Err(_) => self.enter_bootloader(),
        }
    }

    fn check_boot_state(&mut self) -> Result<bool, B::Error> {
        if self.platform.ctl.is_boot_requested() {
            log_info!("Boot requested");
            self.platform.boot_meta.advance()?;
            return Ok(true);
        }

        match self.platform.boot_meta.boot_state() {
            BootState::Idle => {
                if !self.validate_app() {
                    return Ok(true);
                }
            }
            BootState::Updating => return Ok(true),
            BootState::Validating => {
                if self.platform.boot_meta.trials_remaining() == 0 {
                    return Ok(true);
                }
                self.platform.boot_meta.consume_trial()?;
            }
        }

        Ok(false)
    }

    fn validate_app(&self) -> bool {
        let stored = self.platform.boot_meta.app_checksum();
        if stored != 0xFFFF {
            use tinyboot_protocol::crc::{CRC_INIT, crc16};
            return crc16(CRC_INIT, self.platform.storage.as_slice()) == stored;
        }
        // No CRC stored (virgin/debugger-flashed) — check if app exists
        let data = self.platform.storage.as_slice();
        data.len() >= 4
            && unsafe { core::ptr::read_volatile(data.as_ptr() as *const u32) } != 0xFFFF_FFFF
    }

    fn enter_bootloader(&mut self) -> ! {
        log_info!("Entering bootloader mode");

        let mut d = protocol::Dispatcher::new(&mut self.platform);

        loop {
            let _ = d.dispatch();
        }
    }
}
