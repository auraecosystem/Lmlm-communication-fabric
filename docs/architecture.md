# LCF Architecture

## Layers

1. **Application:** text, voice signalling, file exchange, local broadcast and synchronization.
2. **Messaging service:** identity addressing, message IDs, expiry, priority, acknowledgements and persistence.
3. **Network control:** peer discovery, neighbor state, route selection and topology convergence.
4. **Transport adapters:** BLE, Wi-Fi Direct/local Wi-Fi, Ethernet, LoRa and later licensed radio or satellite links.
5. **Device runtime:** queues, scheduler, storage quotas, power policy and fault recovery.

## Operating modes

- **Connected mesh:** opportunistic multi-hop routing while links are available.
- **Delay-tolerant networking:** store-carry-forward when no end-to-end path exists.
- **Local island:** nodes continue communicating inside a disconnected LAN or radio cluster.
- **Gateway bridge:** an explicitly configured gateway may bridge clusters; internet is optional, never assumed.

## Message lifecycle

Create -> validate -> encrypt/authenticate (production integration) -> enqueue -> discover neighbor -> select route -> forward -> deduplicate -> deliver -> acknowledge.

Each hop checks expiry, hop budget, size limits and duplicate state. Relays should not need plaintext access to end-to-end encrypted payloads.

## Trust boundaries

The transport is untrusted. A discovered peer is not automatically an authorized peer. Production identity should use device keys and explicit trust enrollment. The AI/intelligence adapter is advisory only and cannot authorize peers, alter cryptographic policy, or bypass deterministic packet validation.

## Voice and bandwidth

Low-latency voice belongs on Wi-Fi mesh or other broadband links. LoRa supports short messages and telemetry only. Do not attempt general voice/video over narrowband LoRa.

## Planned protocol modules

LCF-ID (identity), LCF-DISC (discovery), LCF-ROUTE (routing), LCF-LINK (link framing), LCF-MSG (messaging), LCF-SEC (security), LCF-DTN (store-carry-forward), LCF-SYNC (replication), and LCF-CTRL (management).
