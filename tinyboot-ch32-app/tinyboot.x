/* Place app version immediately after all other flash content.
 * The bootloader reads it at storage[app_size - 2]. */
SECTIONS
{
    .tinyboot_version ALIGN(2) :
    {
        KEEP(*(.tinyboot_version));
    } > FLASH
} INSERT AFTER .data;
