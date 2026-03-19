/* App version placed at the last 2 bytes of the FLASH region.
 * The bootloader reads this address to report app_version in Info. */
SECTIONS
{
    .tinyboot_version ORIGIN(FLASH) + LENGTH(FLASH) - 2 :
    {
        KEEP(*(.tinyboot_version));
    } > FLASH
}
