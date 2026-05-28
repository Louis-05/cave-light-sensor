# sensor-ble-test — BLE LED controller for Seeed Xiao nRF52840

A small Embassy + nrf-softdevice firmware that exposes the three on-board user
LEDs (red, green, blue) over BLE. A connected client can independently enable,
disable, and set the blink period of each LED.

The BLE wire format is documented in [`BLE_interface.txt`](BLE_interface.txt) —
hand that file to whoever is writing the controlling app.

## Behaviour at a glance

1. **Reset / power on** — the three LEDs rotate in a chase pattern (red → green
   → blue) for 15 seconds. This is the visible "firmware booted" signal.
2. **After 15 s** — all LEDs go off. The board starts advertising over BLE
   under the name **`XiaoLED`**.
3. **Once a client connects and writes** — each LED follows its own
   `enabled` / `period_ms` independently. When all LEDs are disabled the
   per-LED tasks block on a wake signal, the executor sits in `wfe`, and only
   the SoftDevice's advertising radio and LFCLK/RTC stay active. Expected
   battery-rail current at idle: well under 100 µA.

## Target

- Board : Seeed Studio Xiao nRF52840 (with the factory Adafruit nRF52 UF2
  bootloader, which already includes Nordic SoftDevice S140)
- Host  : WSL (Ubuntu) on Windows

## Prerequisites

Same as [`../sensor-nrf`](../sensor-nrf):

1. **Rust stable** — `rust-toolchain.toml` installs the
   `thumbv7em-none-eabihf` target plus `llvm-tools-preview` and `rust-src`
   automatically the first time you build.
2. **`cargo-binutils`** (provides `cargo objcopy`):
   ```bash
   cargo install cargo-binutils
   ```
3. **`uf2conv.py`** — copy or symlink the two files from
   `../sensor-nrf/buildscripts/` into `buildscripts/` here, or re-download:
   ```bash
   cd buildscripts
   curl -sLO https://raw.githubusercontent.com/microsoft/uf2/master/utils/uf2conv.py
   curl -sLO https://raw.githubusercontent.com/microsoft/uf2/master/utils/uf2families.json
   cd ..
   ```

## Build

```bash
cargo build --release
```

If the firmware boots into the chase animation but then halts silently
when the SoftDevice tries to start, the RAM reservation was too small.
The 16 KB reserved in `memory.x` (RAM `ORIGIN = 0x20004000`) is generous
for our minimal peripheral config (1 conn, ATT MTU 64, no central role);
S140 v7.3.0 typically needs 6–8 KB for this configuration. If you change
`ble::sd_config` to use more connections, a larger ATT MTU, or central
mode, raise `RAM ORIGIN` accordingly. The flash layout itself is fixed by
the Adafruit bootloader (app must start at `0x27000`) — see the comments
in `memory.x` for the full breakdown.

## Flash (UF2 drag-and-drop)

```bash
./buildscripts/flash.sh
```

Produces `target/sensor-ble-test.uf2`. Double-tap reset on the Xiao to enter
the bootloader (a USB drive named `XIAO-BOOT` appears), then drop the UF2 onto
that drive. The board reboots into the new firmware as soon as the copy ends.

## Verify it works

1. After reset, watch the LEDs do the 15-second chase, then all go dark.
2. On a phone install **nRF Connect for Mobile** (Nordic Semiconductor, free).
3. Scan — `XiaoLED` should appear.
4. Connect, expand the "LED Control" service (UUID starts with `a8c1f8a0`),
   and write 5-byte payloads to the red/green/blue characteristics. See
   [`BLE_interface.txt`](BLE_interface.txt) §7 for example payloads.

## Source layout

| File | Purpose |
| --- | --- |
| [src/main.rs](src/main.rs) | Peripheral init, 15 s inline boot chase, SD enable, task spawn, BLE main loop |
| [src/state.rs](src/state.rs) | `LedState` — atomics + Signal, plus three `static` instances (RED/GREEN/BLUE) |
| [src/led_task.rs](src/led_task.rs) | The per-LED async blink task — one spawned instance per colour |
| [src/ble.rs](src/ble.rs) | SoftDevice config, `#[gatt_service]` / `#[gatt_server]`, advertising loop |
| [Cargo.toml](Cargo.toml) | Deps: Embassy 0.10 + nrf-softdevice (git, pinned rev) + s140 |
| [memory.x](memory.x) | FLASH at `0x27000` (above SD), RAM at `0x20002000` (above SD) |
| [BLE_interface.txt](BLE_interface.txt) | The BLE GATT spec for client developers |
| [buildscripts/flash.sh](buildscripts/flash.sh) | `cargo build → objcopy → uf2conv` pipeline |

## Power note

Measure on **VBAT**, not USB. With no client connected, all LEDs off, and the
SoftDevice advertising at its default ~100 ms interval, idle current is
dominated by the radio bursts during each ADV event — expect tens of µA
average. With a client connected and all LEDs off the connection event drives
periodic radio wake-ups instead, at a similar average.

## Troubleshooting

- **`XIAO-BOOT` drive doesn't appear** — the double-tap didn't register;
  try faster or hold reset for a moment then tap once.
- **Linker error about RAM overlap or `_stack_start`** — see the build
  section above; raise `RAM ORIGIN` in `memory.x`.
- **Board boots but never advertises** — wait 16 s after reset. The chase
  blocks advertising. If the chase itself never runs, the wrong UF2 was
  copied or the SoftDevice on the chip got erased (re-flash the bootloader
  from Seeed's docs).
- **Board flashes but UF2 fails to load** — wrong family ID. Check
  `buildscripts/flash.sh` uses `0xADA52840`.
