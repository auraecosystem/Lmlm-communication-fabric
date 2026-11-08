# LCF Architecture

## Runtime layers

1. Application services (messaging, voice signalling, files and local broadcast).
2. Message envelope, validation, duplicate suppression, acknowledgement/retry tracking and durable spool.
3. Signed peer statements, lease-based discovery and destination route table.
4. Session packet protection (ChaCha20-Poly1305), replay window and bounded LCF/1 codec.
5. Transport adapters: UDP on an existing local IP network; BLE GATT framing contract.
6. Platform runtime and device-specific radio drivers.

## Implemented behavior

- Peer advertisements carry a monotonically increasing sequence and a lease.
- Expired peers can be removed; stale advertisements are rejected.
- Routes use positive metrics, expire, and are invalidated when their next-hop neighbor disappears.
- LCF/1 binary frames use bounded strings and payload sizes; decoder rejects malformed or trailing data.
- UDP datagrams work across a local Wi-Fi/LAN without a public internet route.
- BLE GATT UUIDs and MTU fragmentation/reassembly are defined. Native iOS, Android and MCU radio bindings remain to be written.
- Persistent spool uses length-prefixed records, syncs appended data, and recovers complete records after restart.
- Ed25519 signatures authenticate peer statements against a provisioned public key.
- ChaCha20-Poly1305 protects packet confidentiality and integrity; a 64-packet replay window rejects duplicate and stale counters after successful authentication.
- Delivery tracking bounds pending messages and schedules retries with capped exponential backoff until acknowledgement or expiry.

## Security boundaries and operational requirements

Discovery metadata must be verified against a trusted Ed25519 public key before it affects peer trust. Session keys are supplied by an external trusted provisioning mechanism; this repository does not yet implement an authenticated key exchange or secure key store. Use a distinct cryptographically random 32-bit nonce prefix per traffic direction and key, never reuse a key/prefix/counter tuple, persist counters safely or rekey before restart, and authenticate acknowledgements before removing pending deliveries.

The crypto module is a building block, not a complete production security system. Authorization, key rotation, secure storage, signed updates, platform entropy integration and an audited authenticated handshake remain required. UDP and BLE remain untrusted transports. LMLM must not bypass deterministic packet validation.

## Bandwidth policy

Use Wi-Fi/LAN for real-time voice and larger transfers. BLE is suited to proximity and small data. LoRa is for compact asynchronous messages and telemetry, not broadband voice/video.
