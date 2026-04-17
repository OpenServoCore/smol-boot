//! Boot-time platform container.

use crate::traits::{BootCtl, BootMetaStore, Storage, Transport};

/// Concrete platform holding all boot-time peripherals.
///
/// Constructed by the board-specific crate and passed to [`Core::new`](crate::Core::new).
pub struct Platform<T, S, B, C>
where
    T: Transport,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    /// UART / RS-485 transport.
    pub transport: T,
    /// Flash storage for reading and writing firmware.
    pub storage: S,
    /// Persistent boot metadata (state, trials, checksum).
    pub boot_meta: B,
    /// Boot control (reset, boot mode selection).
    pub ctl: C,
}

impl<T, S, B, C> Platform<T, S, B, C>
where
    T: Transport,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    /// Assemble a platform from its components.
    #[inline(always)]
    pub fn new(transport: T, storage: S, boot_meta: B, ctl: C) -> Self {
        Self {
            transport,
            storage,
            boot_meta,
            ctl,
        }
    }
}
