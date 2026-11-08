# LMLM Communication Fabric (LCF)

Offline-first, transport-neutral communications for local and regional networks.

## Stage 2 implementation
- Lease-based peer discovery with monotonic advertisement sequences.
- Destination routing table with metric selection, expiry and neighbor invalidation.
- Bounded LCF/1 binary envelope codec.
- UDP datagram transport for devices on the same Wi-Fi/LAN without internet.
- BLE GATT service contract and MTU fragmentation/reassembly primitives.
- Append-only persistent spool with restart recovery.

## Workspace
- `crates/lcf-core`: envelope, validation, discovery, routing and storage.
- `crates/lcf-transport`: UDP, wire codec and BLE framing.
- `crates/lcf-sim`: offline relay demonstration.

## Run
```sh
cargo test --workspace
cargo run -p lcf-sim
```

## Platform status
UDP is implemented. BLE framing and UUIDs are implemented, but native CoreBluetooth (iOS), Android BLE and MCU drivers remain platform-specific work. Wi-Fi Direct group formation is OS-managed; UDP uses an existing local IP interface.

## Security limitations
The current codec is not encrypted or authenticated. Discovery is not trust. Do not use this stage for sensitive traffic over untrusted links until audited identity, authenticated encryption, replay protection and authorization are implemented. LoRa is for compact asynchronous messages, not broadband voice/video.
