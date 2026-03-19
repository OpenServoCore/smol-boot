#![no_std]

use tinyboot::traits::BootState;
use tinyboot::traits::app::BootClient as TBBootClient;
use tinyboot_ch32_hal::{flash, pfic};
use tinyboot_protocol::frame::{Frame, InfoData};
use tinyboot_protocol::{Cmd, Status};

// Re-exports so app examples only need this one crate.
pub use tinyboot::traits::app as traits;
pub use tinyboot_protocol::frame::{self, payload_size};

/// App info configuration — must match the bootloader's storage geometry.
#[derive(Clone, Copy)]
pub struct AppInfo {
    pub capacity: u32,
    pub payload_size: u16,
    pub erase_size: u16,
    pub version: u16,
}

#[derive(Default)]
pub struct BootClient;

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

/// Poll for tinyboot commands (blocking).
/// Info: responds with device geometry. Reset: addr=0 normal, addr=1 enter bootloader.
pub fn poll_cmd<const D: usize, R: embedded_io::Read, W: embedded_io::Write>(
    rx: &mut R,
    tx: &mut W,
    frame: &mut Frame<D>,
    info: &AppInfo,
) {
    if frame.read(rx).is_err() {
        return;
    }
    handle_cmd(frame, info);
    if frame.cmd != Cmd::Reset {
        let _ = frame.send(tx);
        let _ = tx.flush();
    }
}

/// Poll for tinyboot commands (async).
/// Info: responds with device geometry. Reset: addr=0 normal, addr=1 enter bootloader.
pub async fn poll_cmd_async<
    const D: usize,
    R: embedded_io_async::Read,
    W: embedded_io_async::Write,
>(
    rx: &mut R,
    tx: &mut W,
    frame: &mut Frame<D>,
    info: &AppInfo,
) {
    if frame.read_async(rx).await.is_err() {
        return;
    }
    handle_cmd(frame, info);
    if frame.cmd != Cmd::Reset {
        let _ = frame.send_async(tx).await;
        let _ = tx.flush().await;
    }
}

fn handle_cmd<const D: usize>(frame: &mut Frame<D>, info: &AppInfo) {
    frame.status = Status::Ok;
    match frame.cmd {
        Cmd::Info => {
            frame.len = 10;
            frame.data.info = InfoData {
                capacity: info.capacity,
                payload_size: info.payload_size,
                erase_size: info.erase_size,
                version: info.version,
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
        }
    }
}
