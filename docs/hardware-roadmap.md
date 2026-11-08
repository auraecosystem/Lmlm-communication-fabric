# Hardware Roadmap

## Phase 0 — software first

Run the Rust core and deterministic simulator on a laptop or Linux host. Validate offline queues, expiry, hop limits, duplicate suppression, partition recovery and resource limits before buying radios.

## Phase 1 — phone-to-phone

Prototype nearby exchange over BLE and local Wi-Fi. Mobile applications must expose transport capability and permissions clearly and must tolerate background-execution limits imposed by mobile operating systems.

## Phase 2 — first radio node

Reference direction: ESP32-S3-class MCU + SX1262 LoRa transceiver + antenna + battery management + optional GNSS and removable storage. Use BLE/Wi-Fi for local device access and LoRa for compact asynchronous packets.

## Phase 3 — gateway and mesh

Add a Linux/Raspberry Pi gateway, persistent message store, radio interface abstraction, monitoring and controlled bridging between local clusters.

## Phase 4 — voice and advanced links

Add voice over Wi-Fi mesh; evaluate DMR, amateur radio and satellite only where lawful, licensed and technically appropriate.

## Compliance and safety

Before transmitting, verify the permitted frequency bands, duty-cycle limits, output power, antenna requirements, equipment type approval and licensing requirements with the relevant Nigerian authorities. This repository does not authorize radio transmission.
