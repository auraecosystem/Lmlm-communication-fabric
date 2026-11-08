# Hardware Roadmap

## Software stage

Run `cargo test --workspace` and the simulator on a host. Use two local processes or devices on the same Wi-Fi hotspot/LAN to exercise UDP without internet.

## BLE stage

The repository defines LCF GATT UUIDs and MTU fragmentation/reassembly. Implement native CoreBluetooth on iOS, Android BLE, and the chosen embedded BLE stack behind `BleFrameIo`. iOS background execution and peripheral-role restrictions need explicit testing.

## Radio node

Reference direction: ESP32-S3 + SX1262 LoRa + antenna + battery management + optional GNSS/storage. LoRa driver and radio framing are not included in this commit.

## Compliance

Before transmitting, verify Nigerian frequency allocation, duty cycle, power, antenna, equipment approval and licensing requirements with the Nigerian Communications Commission. This repository does not authorize radio transmission.
