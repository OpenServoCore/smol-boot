# System Flash Example

Bootloader hosted in the CH32V003's 1920-byte system flash region, leaving the
entire 16KB user flash available for applications.

> **Note:** This example targets the CH32V003 which has 1920 bytes of system
> flash. Other variants in the CH32V00x family (CH32V002, V004, V005, V006,
> V007, M007) have **3KB + 256 bytes** of system flash, making them
> significantly roomier for a system-flash bootloader. Newer CH32 families
> may have even more. Adjust `memory.x` accordingly for your target chip.

## Memory Layout

```
 System Flash (0x1FFFF000)
 ┌──────────────────────────────┐ 0x1FFFF000
 │  Bootloader code (≤1918 B)  │
 ├──────────────────────────────┤ 0x1FFFF77E
 │  Boot version (2 B)         │
 └──────────────────────────────┘ 0x1FFFF780

 Option Bytes (0x1FFFF800)
 ┌──────────────────────────────┐ 0x1FFFF800
 │  Chip config (16 B)         │
 ├──────────────────────────────┤ 0x1FFFF810
 │  Boot metadata (8 B)        │  state, trials, checksum (4 halfwords)
 └──────────────────────────────┘ 0x1FFFF818

 User Flash (0x08000000)
 ┌──────────────────────────────┐ 0x08000000
 │                              │
 │  Application (16KB - 2)      │
 │                              │
 ├──────────────────────────────┤ 0x08003FFE
 │  App version (2 B)           │
 └──────────────────────────────┘ 0x08004000

 RAM (0x20000000)
 ┌──────────────────────────────┐ 0x20000000
 │  Data / BSS / Stack (2KB)   │
 └──────────────────────────────┘ 0x20000800
```

## Boot Metadata

Boot metadata (state, trials, checksum) is stored in **option bytes** at
`0x1FFFF810`. This avoids consuming any system flash or user flash for
metadata. OB supports 1→0 bit writes for state transitions without erasing.

The chip config halfwords at `0x1FFFF800–0x1FFFF80F` (RDPR, USER, Data, WRPR)
are preserved across OB erase+rewrite cycles.

> **Warning:** The OB region is 64 bytes, but only the first 16 bytes are
> documented for chip config. The remaining bytes (`0x1FFFF810+`) default to
> `0xFF` and are writable — so we took the liberty to commandeer 8 of them
> for boot metadata. If your application also manipulates option bytes
> (e.g. via the OB erase command), preserve the halfwords at
> `0x1FFFF810–0x1FFFF817` or the bootloader will lose its state.

## Boot State Machine

| State      | Value | Boot Action                                                  |
| ---------- | ----- | ------------------------------------------------------------ |
| Idle       | 0xFF  | Validate app (CRC or blank check) → boot or enter bootloader |
| Updating   | 0x7F  | Enter bootloader (transfer was interrupted)                  |
| Validating | 0x3F  | Consume trial, boot app                                      |

**Lifecycle:**

1. First Erase command → Idle (0xFF) → Updating (0x7F)
2. Verify command → writes Validating (0x3F) + checksum to OB
3. Reset → bootloader sees Validating, consumes trial, boots app
4. App confirms → full OB refresh back to Idle with checksum preserved

## Boot Request Mechanism

Uses the hardware `BOOT_MODE` register in the CH32V003 flash controller
(`STATR` bit 14). The app sets this bit and triggers a soft reset to re-enter
the bootloader. The bootloader clears RMVF and BOOT_MODE before booting the
app so the next reset goes to user flash.

## Building

```sh
cargo build -p boot --release
cargo build -p app --release
```

## Flashing

The bootloader must be flashed with [wlink](https://github.com/ch32-rs/wlink)
because probe-rs does not support writing to system flash, and the bootloader's
VMA (0x0) differs from its LMA (0x1FFFF000) which probe-rs cannot handle.

wlink reads the ELF directly and uses the LMA for placement — no objcopy needed.

```sh
# Flash bootloader to system flash
wlink flash target/riscv32ec-unknown-none-elf/release/boot

# Erase user flash and power cycle, then flash app via tinyboot-cli
wlink erase
# (power cycle)
tinyboot flash --reset --port /dev/ttyACM0 target/riscv32ec-unknown-none-elf/release/app
```
