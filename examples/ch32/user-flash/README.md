# User Flash Example

Bootloader hosted in user flash alongside the application. The 16KB user flash
is partitioned between the bootloader (4KB) and the application (12KB).

This configuration has more room for features — defmt logging is enabled.

> **Note:** User-flash mode is primarily useful for debugging, since it
> allows enabling defmt and other features that don't fit in system flash.
> For production, prefer hosting the bootloader in system flash so the
> entire user flash is reserved for the application and no custom `memory.x`
> is needed for the app. See the [`system-flash`](../system-flash/) example.

## Memory Layout

```
 User Flash (0x08000000)
 ┌──────────────────────────────┐ 0x08000000
 │  Bootloader code (4KB-66)   │
 ├──────────────────────────────┤ 0x08000FBE
 │  Boot version (2 B)         │
 ├──────────────────────────────┤ 0x08000FC0
 │  Boot metadata (64 B)       │
 ├──────────────────────────────┤ 0x08001000
 │                              │
 │  Application (12KB - 2)      │
 │                              │
 ├──────────────────────────────┤ 0x08003FFE
 │  App version (2 B)           │
 └──────────────────────────────┘ 0x08004000

 Option Bytes (0x1FFFF800)
 ┌──────────────────────────────┐ 0x1FFFF800
 │  Chip config (16 B)         │
 ├──────────────────────────────┤ 0x1FFFF810
 │  Boot metadata (8 B)        │  state, trials, checksum (4 halfwords)
 └──────────────────────────────┘ 0x1FFFF818

 RAM (0x20000000)
 ┌──────────────────────────────┐ 0x20000000
 │  Boot request word (4 B)    │  ← NOLOAD, survives soft reset
 ├──────────────────────────────┤ 0x20000004
 │  Data / BSS / Stack (2KB-4) │
 └──────────────────────────────┘ 0x20000800
```

## Boot Metadata

Boot metadata (state, trials, checksum) is stored in **option bytes** at
`0x1FFFF810`, shared with the system-flash example. This avoids consuming
user flash for metadata.

> **Warning:** The OB bytes at `0x1FFFF810–0x1FFFF817` are spare space in
> the 64-byte OB region — we took the liberty to commandeer them for boot
> metadata. If your application manipulates option bytes directly, preserve
> these halfwords or the bootloader will lose its state.

## Boot Request Mechanism

Without the `system-flash` feature, the hardware `BOOT_MODE` register is not
available. Instead, a **magic word** (`0xB007_CAFE`) is written to a reserved
4-byte region at the start of RAM. This region is placed in a `NOLOAD` linker
section so it is not zeroed on startup, preserving its value across soft resets.

Both the bootloader and the app must link `boot_request.x` to reserve this
word. The bootloader's `link.x` includes it directly; the app links it via
`-Tboot_request.x` in its `build.rs`.

## Building

```sh
cargo build -p boot --release
cargo build -p app --release
```

## Flashing

```sh
# Flash bootloader
wlink flash target/riscv32ec-unknown-none-elf/release/boot

# Flash app via tinyboot-cli
tinyboot flash --reset --port /dev/ttyACM0 target/riscv32ec-unknown-none-elf/release/app
```
