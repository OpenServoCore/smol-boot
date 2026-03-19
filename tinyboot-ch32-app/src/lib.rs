#![no_std]

use tinyboot::traits::BootState;
use tinyboot::traits::app::BootClient as TBBootClient;
use tinyboot_ch32_hal::{flash, pfic};
use tinyboot_protocol::frame::{Frame, InfoData};
use tinyboot_protocol::{Cmd, Status};

// Re-exports so app examples only need this one crate.
pub use tinyboot::traits::app as traits;
pub use tinyboot_protocol::pkg_version;

/// Define the `.tinyboot_version` static using the calling crate's version.
/// Place this at module scope in your application binary.
#[macro_export]
macro_rules! app_version {
    () => {
        #[unsafe(link_section = ".tinyboot_version")]
        #[used]
        static _APP_VERSION: u16 = $crate::pkg_version!();
    };
}

/// Hardware configuration for the app-side tinyboot client.
pub struct AppConfig {
    pub boot_base: u32,
    pub boot_size: u32,
    pub app_size: u32,
    pub erase_size: u16,
}

/// App-side tinyboot client. Handles Info/Reset commands and boot confirmation.
pub struct App {
    frame: Frame,
    info: AppInfo,
}

struct AppInfo {
    capacity: u32,
    erase_size: u16,
    boot_version: u16,
    app_version: u16,
}

impl App {
    pub fn new(config: &AppConfig) -> Self {
        let boot_ver_addr = (config.boot_base + config.boot_size - 2) as *const u16;
        Self {
            frame: Frame::default(),
            info: AppInfo {
                capacity: config.app_size,
                erase_size: config.erase_size,
                boot_version: unsafe { boot_ver_addr.read_volatile() },
                app_version: pkg_version!(),
            },
        }
    }

    /// Confirm boot — transitions Validating → Idle, preserving checksum.
    /// Call after all peripherals are initialized.
    pub fn confirm(&mut self) {
        BootClient.confirm();
    }

    /// Poll for tinyboot commands (blocking).
    pub fn poll<R: embedded_io::Read, W: embedded_io::Write>(&mut self, rx: &mut R, tx: &mut W) {
        let status = match self.frame.read(rx) {
            Ok(s) => s,
            Err(_) => return,
        };
        if status == Status::Ok {
            handle_cmd(&mut self.frame, &self.info);
        } else {
            self.frame.len = 0;
            self.frame.status = status;
        }
        if self.frame.cmd != Cmd::Reset {
            let _ = self.frame.send(tx);
            let _ = tx.flush();
        }
    }

    /// Poll for tinyboot commands (async).
    pub async fn poll_async<R: embedded_io_async::Read, W: embedded_io_async::Write>(
        &mut self,
        rx: &mut R,
        tx: &mut W,
    ) {
        let status = match self.frame.read_async(rx).await {
            Ok(s) => s,
            Err(_) => return,
        };
        if status == Status::Ok {
            handle_cmd(&mut self.frame, &self.info);
        } else {
            self.frame.len = 0;
            self.frame.status = status;
        }
        if self.frame.cmd != Cmd::Reset {
            let _ = self.frame.send_async(tx).await;
            let _ = tx.flush().await;
        }
    }
}

#[derive(Default)]
struct BootClient;

impl TBBootClient for BootClient {
    fn confirm(&mut self) {
        critical_section::with(|_| {
            if BootState::from_u8(flash::ob_boot_state()) != BootState::Validating {
                return;
            }
            flash::unlock();
            flash::ob_refresh(BootState::Idle as u8, flash::ob_checksum());
            flash::lock();
        });
    }

    fn request_update(&mut self) -> ! {
        critical_section::with(|_| {
            #[cfg(feature = "system-flash")]
            flash::set_boot_mode(true);
            #[cfg(not(feature = "system-flash"))]
            tinyboot_ch32_hal::boot_request::set_boot_request(true);
        });
        pfic::system_reset()
    }
}

fn handle_cmd(frame: &mut Frame, info: &AppInfo) {
    frame.status = Status::Ok;
    match frame.cmd {
        Cmd::Info => {
            frame.len = 12;
            frame.data.info = InfoData {
                capacity: info.capacity,
                erase_size: info.erase_size,
                boot_version: info.boot_version,
                app_version: info.app_version,
                mode: 1, // app
            };
        }
        Cmd::Reset => {
            #[cfg(feature = "defmt")]
            defmt::info!("Reset received (addr={})", frame.addr);
            if frame.addr == 1 {
                BootClient.request_update();
            } else {
                pfic::system_reset();
            }
        }
        _ => {
            frame.len = 0;
            frame.status = Status::Unsupported;
        }
    }
}
