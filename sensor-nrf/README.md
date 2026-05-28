# sensor-nrf — Embassy blinky for Seeed Xiao nRF52840

A minimal Embassy async "blinky" intended as the starting template for an
ultra-low-power sensor project. The red user LED on `P0.26` pulses ~500 ms
on / 500 ms off; between pulses the core sleeps in `wfe`, with only the
LFCLK + RTC1 running.

Target board: **Seeed Studio Xiao nRF52840** (no onboard debugger; flashed
via the factory Adafruit nRF52 UF2 bootloader).
Development host: **WSL (Ubuntu) on Windows**.

## Prerequisites

1. **Rust stable** — the `rust-toolchain.toml` in this directory installs
   the `thumbv7em-none-eabihf` target and the `llvm-tools-preview` /
   `rust-src` components automatically the first time you run `cargo`
   here. You only need `rustup` already installed.

2. **`cargo-binutils`** — provides `cargo objcopy`:

   ```bash
   cargo install cargo-binutils
   ```

3. **`uf2conv.py`** — Microsoft's UF2 conversion script. Download once
   into `buildscripts/` (it expects `uf2families.json` to sit next to it):

   ```bash
   cd buildscripts
   curl -sLO https://raw.githubusercontent.com/microsoft/uf2/master/utils/uf2conv.py
   curl -sLO https://raw.githubusercontent.com/microsoft/uf2/master/utils/uf2families.json
   cd ..
   ```

   `python3` (any 3.x) must be on `PATH`.

## Build

```bash
cargo build --release
```

Optional sanity-check on code size:

```bash
cargo size --release
```

`.text` should sit well under 100 KB.

## Flash (UF2 drag-and-drop)

1. **Enter bootloader:** double-tap the Xiao's reset button. A USB
   mass-storage drive named **`XIAO-BOOT`** will appear in Windows
   Explorer (e.g. as `D:`).

2. **Find the WSL path:** Windows drive `D:` is `/mnt/d/` from WSL.
   Confirm with `ls /mnt/d/` — you should see `CURRENT.UF2`,
   `INDEX.HTM`, `INFO_UF2.TXT`.

3. **Build the UF2:**

   ```bash
   ./buildscripts/flash.sh
   ```

   Produces `target/sensor-nrf.uf2`. Drag and drop that file onto the
   `XIAO-BOOT` drive in Windows Explorer. The board reboots into the new
   firmware as soon as the copy completes.

## What you should see

Red LED blinking at 500 ms on / 500 ms off, indefinitely.

## Power note

During the off interval, `embassy_time::Timer::after(..).await`
yields to the Embassy thread executor, which calls `cortex_m::asm::wfe()`.
Only the 32.768 kHz LFCLK and RTC1 stay running, so current at **VBAT**
should sit in the single-digit µA range. (On **VBUS** the onboard USB
regulator dominates and you'll see tens of mA — measure on the battery
rail, not on USB.)

## Troubleshooting

- **`XIAO-BOOT` drive doesn't appear** — the double-tap didn't register.
  Try faster, or hold reset for a moment then tap once.
- **`uf2conv.py: family ... unknown`** — `uf2families.json` isn't next to
  `uf2conv.py`. Re-download both files into `buildscripts/`.
- **Link error mentioning `_stack_start` / `FLASH`** — `memory.x` not
  found. Make sure you run `cargo` from the project root (`sensor-nrf/`).
- **`cargo: command not found: objcopy`** — install `cargo-binutils`
  (see prerequisites).
- **Board flashes but red LED doesn't blink** — wrong family ID. The
  Xiao nRF52840 needs `0xADA52840` (already set in `buildscripts/flash.sh`);
  the generic nRF52840 family `0x1B57745F` is rejected by the Adafruit bootloader.

## File map

| File | Purpose |
| --- | --- |
| [Cargo.toml](Cargo.toml) | Deps (Embassy, cortex-m, panic-halt) and size-optimised release profile |
| [.cargo/config.toml](.cargo/config.toml) | `thumbv7em-none-eabihf` target + linker flags |
| [rust-toolchain.toml](rust-toolchain.toml) | Pins stable + auto-installs target/components |
| [memory.x](memory.x) | Flash starts at `0x27000` (above Adafruit MBR/SD region) |
| [src/main.rs](src/main.rs) | Blinky entry point |
| [buildscripts/flash.sh](buildscripts/flash.sh) | `build → objcopy → uf2conv` pipeline |
| [buildscripts/uf2conv.py](buildscripts/uf2conv.py) | UF2 conversion script (download separately) |
| [buildscripts/uf2families.json](buildscripts/uf2families.json) | Family ID table for uf2conv (download separately) |
