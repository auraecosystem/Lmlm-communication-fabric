# LCF Architecture

## Runtime layers

1. Application services (messaging, voice signalling, files and local broadcast).
2. Message envelope, validation, duplicate suppression, acknowledgement/retry tracking and durable spool.
3. Signed peer statements, lease-based discovery and destination route table.
4. Deterministic relay engine: validate, resolve next hop, suppress duplicates and decrement hop limit.
5. Identity-bound X25519 handshake, HKDF-SHA256 directional session keys, ChaCha20-Poly1305 and replay window.
6. Bounded LCF/1 codec and transport adapters: UDP on an existing local IP network; BLE GATT framing contract.
7. Platform runtime and device-specific radio drivers.

## Implemented behavior

- Peer advertisements carry a monotonically increasing sequence and a lease.
- Expired peers can be removed; stale advertisements are rejected.
- Routes use positive metrics, expire, and are invalidated when their next-hop neighbor disappears.
- Relay decisions validate envelope lifetime and payload bounds, deliver local destinations, resolve remote next hops, suppress duplicate IDs and decrement hop limits exactly once.
- LCF/1 binary frames use bounded strings and payload sizes; decoder rejects malformed or trailing data.
- UDP datagrams work across a local Wi-Fi/LAN without a public internet route.
- BLE GATT UUIDs and MTU fragmentation/reassembly are defined. Native iOS, Android and MCU radio bindings remain to be written.
- Persistent spool uses length-prefixed records, syncs appended data, and recovers complete records after restart.
- Ed25519 signatures authenticate peer statements and the handshake transcript against caller-supplied trusted public keys.
- Ephemeral X25519 keys plus fresh nonces are mutually authenticated; HKDF-SHA256 derives separate directional session keys and nonce prefixes.
- ChaCha20-Poly1305 protects packet confidentiality and integrity; a 64-packet replay window rejects duplicate and stale counters after successful authentication.
- Delivery tracking bounds pending messages and schedules retries with capped exponential backoff until acknowledgement or expiry.

## Security boundaries and operational requirements

The caller must verify that each peer's Ed25519 public key belongs to the claimed identity before accepting a handshake. Protect long-term identity keys in platform secure storage, implement authorization and revocation, and define key rotation and session restart policy. Never reuse a directional key with the same nonce prefix and counter sequence. Authenticate acknowledgements before removing pending deliveries.

The handshake and packet security code are protocol building blocks, not a completed production security certification. Independent cryptographic/protocol review, secure storage, signed updates and native platform entropy integration remain required. UDP and BLE remain untrusted transports. LMLM must not bypass deterministic packet validation.

## Bandwidth policy

Use Wi-Fi/LAN for real-time voice and larger transfers. BLE is suited to proximity and small data. LoRa is for compact asynchronous messages and telemetry, not broadband voice/video.
