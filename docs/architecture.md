# LCF Architecture

## Runtime layers
1. Application services (messaging, voice signalling, files and local broadcast).
2. Message envelope, validation, duplicate suppression and durable spool.
3. Peer discovery registry and destination route table.
4. Transport adapters: UDP on an existing local IP network; BLE GATT framing contract.
5. Platform runtime and device-specific radio drivers.

## Implemented behavior
- Peer advertisements carry a monotonically increasing sequence and a lease.
- Expired peers can be removed; stale advertisements are rejected.
- Routes use positive metrics, expire, and are invalidated when their next-hop neighbor disappears.
- LCF/1 binary frames use bounded strings and payload sizes; decoder rejects malformed or trailing data.
- UDP datagrams work across a local Wi-Fi/LAN without a public internet route.
- BLE GATT UUIDs and MTU fragmentation/reassembly are defined. Native iOS, Android and MCU radio bindings remain to be written.
- Persistent spool uses length-prefixed records, syncs appended data, and recovers complete records after restart.

## Trust boundaries
Discovery is not authentication. UDP and BLE are untrusted links. The current frame codec does not encrypt or authenticate payloads. Production deployment requires audited device identity, authenticated encryption, replay protection, authorization, key rotation and signed updates. LMLM remains advisory and cannot bypass deterministic packet checks.

## Bandwidth policy
Use Wi-Fi/LAN for real-time voice and larger transfers. BLE is suited to proximity and small data. LoRa is for compact asynchronous messages and telemetry, not broadband voice/video.
