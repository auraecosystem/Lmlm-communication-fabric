# Hardware Roadmap

## Current software stage
Run `cargo test --workspace` and `cargo run -p lcf-sim` on a Rust host. Two devices on the same Wi-Fi hotspot/LAN can exchange UDP datagrams with no internet access.

## BLE integration
Implement `BleFrameIo` using CoreBluetooth on iOS, Android BLE APIs, and the selected embedded BLE stack. iOS background execution, connection lifecycle and peripheral-role restrictions require device testing.

## First radio node
Reference direction: ESP32-S3 + SX1262 LoRa transceiver + suitable antenna + battery management, with optional GNSS and removable storage. Firmware and LoRa radio framing are not implemented in this stage.

## Compliance
Before transmitting, verify Nigerian frequency allocation, duty-cycle limits, output power, antenna requirements, equipment approval and licensing with the Nigerian Communications Commission. This code does not authorize radio transmission.
