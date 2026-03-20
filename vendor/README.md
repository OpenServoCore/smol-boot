# Vendor Bootloader Binaries

Factory system flash images for restoring the vendor bootloader. Useful if you've
overwritten system flash and need to recover.

## Files

| File                        | Chip         | Address      | Size       |
| --------------------------- | ------------ | ------------ | ---------- |
| `ch32v003-system-flash.bin` | CH32V003F4P6 | `0x1FFFF000` | 1920 bytes |

## Restoring

```sh
wlink flash vendor/ch32v003-system-flash.bin --address 0x1FFFF000
```
