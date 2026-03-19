/* CH32V003 user-flash bootloader memory layout.
 *
 * The bootloader occupies the first 4KB of the 16KB user flash, with
 * boot metadata (state, trial counter) in the last 64 bytes of that region.
 * The remaining 12KB is entirely available for the application.
 *
 * CODE is the execution address (flash mirrored at 0x00000000).
 * FLASH is the programming address (FPEC requires 0x08000000-based addresses).
 *
 * Flash map:
 *   0x0800_0000 .. 0x0800_0FFF  bootloader  4KB
 *   0x0800_1000 .. 0x0800_3FFF  application 12KB
 */
MEMORY
{
    CODE  : ORIGIN = 0x00000000, LENGTH = 4K  /* execution alias   */
    FLASH : ORIGIN = 0x08000000, LENGTH = 4K  /* physical address  */
    RAM   : ORIGIN = 0x20000000, LENGTH = 2K
}
