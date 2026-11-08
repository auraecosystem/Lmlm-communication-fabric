# LMLM Communication Fabric (LCF)

LCF is an internet-independent, hybrid communications fabric for local and regional connectivity. Its core is transport-neutral and supports direct links, local IP networks and delay-tolerant relaying.

## Implemented in this stage

- Peer lease registry with advertisement sequence handling and expiry.
- Destination route table with metric-based replacement and neighbor invalidation.
- Versioned, bounded binary envelope codec.
- Real UDP/IP transport adapter for offline LAN/Wi-Fi networks.
- BLE GATT framing contract and MTU-aware fragmentation/reassembly primitives.
- Append-only persistent message spool with restart recovery.

## Workspace

- `crates/lcf-core`: envelope, validation, discovery, routing and persistence.
- `crates/lcf-transport`: UDP transport, binary wire codec and BLE framing primitives.
- `crates/lcf-sim`: runnable offline relay demonstration.
- `docs/architecture.md`: layers and trust boundaries.
- `docs/hardware-roadmap.md`: staged device plan.

## Run

Requires Rust stable:

```sh
cargo test --workspace
cargo run -p lcf-sim
```

## Transport status

UDP is implemented for nodes that share an IP LAN/hotspot; it does not require internet access. BLE has a defined GATT service contract and tested frame fragmentation/reassembly, but platform-specific CoreBluetooth (iOS), Android BLE, and embedded BLE drivers are not yet included. Wi-Fi Direct group formation is OS-specific; the UDP adapter uses an already-established local IP interface.

## Security status

This stage does **not** claim authenticated peer identity or production encryption. Treat UDP/BLE links as untrusted. Do not send sensitive plaintext across an untrusted network until the audited LCF-SEC implementation is integrated. Discovery is not authorization. Production identity, replay protection, encrypted sessions, key storage and signed updates remain mandatory before field deployment.

LoRa remains reserved for compact asynchronous messages and telemetry, not broadband voice/video. Verify Nigerian spectrum, power, equipment approval and licensing requirements before radio transmission.
