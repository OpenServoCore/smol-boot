/* App version placed after all other flash content (end of binary).
 * The bootloader reads it at flash[app_size-2..app_size] using the
 * app_size stored in OB metadata. */
SECTIONS
{
    .tinyboot_version : ALIGN(2)
    {
        KEEP(*(.tinyboot_version));
    } > FLASH
} INSERT AFTER .data;
