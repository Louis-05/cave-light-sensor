#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo build --release
cargo objcopy --release -- -O binary target/firmware.bin
python3 "$SCRIPT_DIR/uf2conv.py" target/firmware.bin \
  --base 0x27000 --family 0xADA52840 \
  --output target/sensor-ble-test.uf2

echo "Done: target/sensor-ble-test.uf2"
