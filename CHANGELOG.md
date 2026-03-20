# Changelog

## [0.2.0] - 2026-03-20

### Changed
- **Breaking:** Verify command now carries `app_size` in the addr field
- **Breaking:** `BootMetaStore` trait adds `app_size()` method; `refresh()` takes an additional `app_size` parameter
- CRC16 validation now covers only actual firmware bytes, not the entire flash region
- CLI only writes actual firmware data — no more 0xFF padding to fill the region
- App version read from end of binary (`flash[app_size-2..app_size]`) instead of end of flash region
- Linker script places `.tinyboot_version` after all other flash content (end of binary) instead of at end of flash region
- OB metadata expanded from 8 to 16 bytes (added app_size u32 field)

### Performance
- Flash time reduced proportionally to firmware size (e.g. 5KB app on 16KB chip: ~8s → ~3s)
- CRC verification faster — only covers firmware bytes

## [0.1.0] - 2026-03-20

Initial release.
