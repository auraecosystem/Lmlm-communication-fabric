# LMLM Communication Fabric (LCF)

Offline-first, transport-neutral communications for local and regional networks.

## Implemented stages

- Lease-based peer discovery with monotonic advertisement sequences.
- Destination routing table with metric selection, expiry and neighbor invalidation.
- Bounded LCF/1 binary envelope codec.
- UDP datagram transport for devices on the same Wi-Fi/LAN without internet.
- BLE GATT service contract and MTU fragmentation/reassembly primitives.
- Append-only persistent spool with restart recovery.
- Ed25519-signed peer statements and ChaCha20-Poly1305 authenticated packet protection.
- Sliding replay window and bounded acknowledgement/retry tracking with exponential backoff.

## Workspace

- `crates/lcf-core`: envelope, validation, discovery, routing, delivery and storage.
- `crates/lcf-transport`: UDP, wire codec, BLE framing and packet security.
- `crates/lcf-sim`: offline relay demonstration.

## Run

```sh
cargo test --workspace
cargo run -p lcf-sim
```

## Security integration contract

The security primitives use established Ed25519 and ChaCha20-Poly1305 implementations. Session keys must be provisioned by a trusted key-management layer. Each traffic direction must use a distinct, cryptographically random 32-bit nonce prefix for a given key. Never reuse a key/prefix/counter tuple; persist counters safely or rekey before restart. The replay window is updated only after successful authentication.

This stage does **not** implement an authenticated key exchange, secure key storage, authorization policy, signed software updates or automatic key rotation. Until those are integrated and audited, treat session key provisioning as an operator responsibility and do not claim a production-ready secure mesh.

## Platform status

UDP is implemented for an existing local IP interface. BLE framing and UUIDs are implemented, but native CoreBluetooth (iOS), Android BLE and MCU drivers remain platform-specific work. Wi-Fi Direct group formation is OS-managed. LoRa is suitable for compact asynchronous messages and telemetry, not broadband voice/video.
