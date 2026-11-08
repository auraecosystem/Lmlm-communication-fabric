# LMLM Communication Fabric (LCF)

LCF is an internet-independent, hybrid communications fabric for local and regional connectivity. It is designed to operate across direct device links, local Wi-Fi networks, Bluetooth, long-range low-bandwidth radio, and delay-tolerant relaying.

## Design principles

- **Offline first:** no cloud account, DNS, public internet, or central server is required for local operation.
- **Transport independent:** applications use a common message API while transports are replaceable.
- **Store, carry, forward:** messages can persist on a node and move when a route becomes available.
- **Secure by default:** authenticate peers, encrypt payloads end-to-end, reject replays, and enforce bounded resource use.
- **AI optional:** LMLM may advise routing and diagnostics, but never sits in the trusted forwarding or cryptographic path.

## Repository status

This repository begins with an executable protocol-core scaffold and simulator. Hardware drivers and radio interoperability are staged behind transport interfaces and are not claimed to be implemented yet.

## Workspace

- `crates/lcf-core`: message envelope, validation, duplicate suppression, TTL and forwarding decisions.
- `crates/lcf-sim`: deterministic in-memory multi-hop simulator.
- `docs/architecture.md`: system architecture and trust boundaries.
- `docs/hardware-roadmap.md`: phased device and radio plan.

## Quick start

Requires Rust stable.

```sh
cargo test --workspace
cargo run -p lcf-sim
```

## Initial hardware direction

The first low-cost reference node is an ESP32-S3-class controller paired with an SX1262 LoRa transceiver, with BLE/Wi-Fi for nearby devices. LoRa is for compact asynchronous messages and telemetry, not broadband voice or video. Frequency, power, antenna, and equipment approvals must comply with local spectrum rules before transmission.

## Security note

The initial scaffold does not claim production cryptography or radio interoperability. Before field deployment, integrate audited cryptographic libraries, persistent key storage, signed firmware updates, abuse controls, and hardware-in-the-loop tests.
