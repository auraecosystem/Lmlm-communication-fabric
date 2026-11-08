# LMLM Communication Fabric (LCF)

Offline-first, transport-neutral communications for local and regional networks.

## Implemented stages

- Lease-based peer discovery with monotonic advertisement sequences.
- Destination routing table with metric selection, expiry and neighbor invalidation.
- Deterministic relay engine with route lookup, duplicate suppression, payload/lifetime validation and hop-limit decrement.
- Bounded LCF/1 binary envelope codec.
- UDP datagram transport for devices on the same Wi-Fi/LAN without internet.
- BLE GATT service contract and MTU fragmentation/reassembly primitives.
- Append-only persistent spool with restart recovery.
- Ed25519-signed peer statements and ChaCha20-Poly1305 authenticated packet protection.
- Mutually authenticated ephemeral X25519 handshake, signed with trusted Ed25519 identity keys.
- HKDF-SHA256 directional session-key derivation and replay-window protection.
- Bounded acknowledgement/retry tracking with capped exponential backoff.

## Workspace

- `crates/lcf-core`: envelope, validation, discovery, routing, relay, delivery and storage.
- `crates/lcf-transport`: UDP, wire codec, BLE framing, identity handshake and packet security.
- `crates/lcf-sim`: offline relay demonstration.

## Run

```sh
cargo test --workspace
cargo run -p lcf-sim
```

## Security integration contract

The handshake authenticates both ephemeral X25519 public keys by signing the complete identity-bound transcript with pre-trusted Ed25519 keys. HKDF-SHA256 derives separate directional AEAD keys and nonce prefixes. Use `SessionMaterial::into_sessions` to obtain independent send/receive cipher states. The caller remains responsible for secure long-term identity-key storage, trusted public-key distribution, authorization, revocation, key rotation and safe session-state lifecycle. A compromised or incorrectly trusted identity key defeats peer authentication.

Never reuse a directional session key with a prior nonce prefix/counter state. Persist counters safely or discard the session and establish a fresh handshake after restart. Acknowledgements must be authenticated before pending deliveries are removed. The crypto code is a protocol implementation requiring independent review before production deployment.

## Platform status

UDP is implemented for an existing local IP interface. BLE framing and UUIDs are implemented, but native CoreBluetooth (iOS), Android BLE and MCU drivers remain platform-specific work. Wi-Fi Direct group formation is OS-managed. LoRa is suitable for compact asynchronous messages and telemetry, not broadband voice/video.
