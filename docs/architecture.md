# LCF Architecture

## Layers

1. Application services.
2. Messaging and durable spool.
3. Discovery registry and destination route table.
4. Transport adapters (UDP/LAN implemented; BLE framing contract provided).
5. Device runtime and platform radio drivers.

## Current implementation

- Peer advertisements use monotonically increasing sequence numbers and expiring leases.
- Routes are selected by a positive metric; a lower metric replaces a higher one. Neighbor loss invalidates routes through that neighbor.
- LCF/1 binary envelope parsing is bounded and rejects truncation, invalid UTF-8, oversized fields and trailing bytes.
- UDP provides actual datagram exchange over a local IP network.
- BLE uses the documented GATT service/characteristic UUIDs and MTU-aware fragmentation. Native OS/MCU integration remains a platform adapter task.
- Persistent spool stores length-prefixed frames and calls sync_data after append.

## Trust boundaries

Discovery is not authentication. A peer advertisement must not grant access to private messages. UDP and BLE are untrusted transports. The current codec does not encrypt or authenticate frames; production use requires audited identity and session crypto, replay protection and authorization before accepting sensitive traffic.

The intelligence plane remains advisory. LMLM cannot override packet validation, trust policy or forwarding limits.

## Voice and bandwidth

Use Wi-Fi/LAN for real-time voice. BLE is for proximity/control and small transfers. LoRa is for compact asynchronous messages and telemetry, not broadband voice/video.
