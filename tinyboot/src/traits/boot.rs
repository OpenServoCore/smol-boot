use super::BootState;

/// Trait for firmware transfer protocol.
///
/// The const generic `D` is the maximum payload size per frame,
/// determined by the transport (e.g. UART frame size minus protocol overhead).
pub trait Transport<const D: usize>: embedded_io::Read + embedded_io::Write {}

/// Trait for reading and writing firmware to persistent storage.
///
/// Flash is memory-mapped, so [`as_slice`](Storage::as_slice) provides
/// zero-copy read access to the app region.
pub trait Storage:
    embedded_storage::nor_flash::NorFlash + embedded_storage::nor_flash::ReadNorFlash
{
    /// Direct read access to the app region (zero-copy).
    fn as_slice(&self) -> &[u8];
}

/// Trait for system boot control.
pub trait BootCtl {
    /// Returns true if the bootloader was explicitly requested (e.g. via boot mode register).
    fn is_boot_requested(&self) -> bool;

    /// Reset the system. `bootloader=true` sets boot mode to enter bootloader,
    /// `bootloader=false` clears it to boot the app.
    fn system_reset(&mut self, bootloader: bool) -> !;
}

/// Persistent boot metadata storage.
pub trait BootMetaStore {
    type Error: core::fmt::Debug;

    /// Current boot lifecycle state.
    fn boot_state(&self) -> BootState;

    /// Number of trial boots remaining (count of 1-bits in trials field).
    fn trials_remaining(&self) -> u8;

    /// Stored CRC16 of the application firmware.
    fn app_checksum(&self) -> u16;

    /// Step state down by one (1→0 bit clear).
    fn advance(&mut self) -> Result<BootState, Self::Error>;

    /// Consume one trial boot (clears one bit in the trials field).
    fn consume_trial(&mut self) -> Result<(), Self::Error>;

    /// Erase meta and rewrite with given checksum and state.
    /// Trials return to erased default (full).
    fn refresh(&mut self, checksum: u16, state: BootState) -> Result<(), Self::Error>;
}

pub struct Platform<const D: usize, T, S, B, C>
where
    T: Transport<D>,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    pub transport: T,
    pub storage: S,
    pub boot_meta: B,
    pub ctl: C,
}

impl<const D: usize, T, S, B, C> Platform<D, T, S, B, C>
where
    T: Transport<D>,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    pub fn new(transport: T, storage: S, boot_meta: B, ctl: C) -> Self {
        Self {
            transport,
            storage,
            boot_meta,
            ctl,
        }
    }
}
