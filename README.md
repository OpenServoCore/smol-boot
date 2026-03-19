# tinyboot

Rust bootloader for resource-constrained microcontrollers. Fits in the CH32V003's 1920-byte system flash with full trial boot, CRC16 app validation, OB-based metadata, and version reporting — leaving the entire 16KB user flash for the application.

## Architecture

- **tinyboot** — Platform-agnostic bootloader core (protocol dispatcher, boot state machine, app validation)
- **tinyboot-protocol** — Wire protocol (frame format, CRC16, commands)
- **tinyboot-ch32-hal** — Minimal CH32 HAL (flash, GPIO, USART, RCC)
- **tinyboot-ch32-boot** — CH32 bootloader platform (storage, boot control, OB metadata)
- **tinyboot-ch32-app** — CH32 app-side boot client (confirm, request update)
- **tinyboot-cli** — Host-side CLI for flashing firmware over UART

## Boot State Machine

```
Idle (0xFF) ──Erase──▶ Updating (0x7F) ──Verify──▶ Validating (0x3F) ──App confirm──▶ Idle (0xFF)
```

Boot metadata is stored in option bytes (OB), not flash. Forward transitions (Idle→Updating, trial consumption) use 1→0 bit writes without erasing. Verify and app `confirm()` perform full OB erase+rewrite cycles, preserving chip config.

## Versioning

Boot and app versions are stored at the last 2 bytes of their respective flash regions, packed as `(major << 11) | (minor << 6) | patch`. The `tinyboot-ch32-app` crate ships a `tinyboot.x` linker script that automatically places the app version. The bootloader's `link.x` does the same for the boot version. Both versions are derived from `Cargo.toml` via `pkg_version!()`.

## Examples

- [`examples/ch32/system-flash`](examples/ch32/system-flash/) — 1920-byte bootloader in system flash (production)
- [`examples/ch32/user-flash`](examples/ch32/user-flash/) — 4KB bootloader in user flash (development, with defmt)
