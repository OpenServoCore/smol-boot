/// App-side boot client interface.
///
/// Provides the two operations an application needs from the bootloader:
/// confirming a successful trial boot, and requesting bootloader entry
/// for a firmware update.
pub trait BootClient {
    /// Confirm a successful boot.
    ///
    /// If the boot state is `Validating`, refreshes OB back to Idle.
    /// Otherwise does nothing (already confirmed or no update in progress).
    fn confirm(&mut self);

    /// Request bootloader entry for a firmware update.
    ///
    /// Writes the boot request flag and performs a soft reset.
    /// This function does not return.
    fn request_update(&mut self) -> !;
}
