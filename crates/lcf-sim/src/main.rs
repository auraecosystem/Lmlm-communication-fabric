use lcf_core::{DuplicateCache, Envelope, MessageQueue};

fn main() {
    // Demonstrate a disconnected route becoming reachable through a relay.
    let mut relay_queue = MessageQueue::new(128, 64 * 1024);
    let mut delivered = DuplicateCache::new(256);
    let message = Envelope {
        id: [42; 16],
        source: "phone-a".into(),
        destination: "edge-c".into(),
        created_at_ms: 1_000,
        expires_at_ms: 86_401_000,
        hop_limit: 5,
        payload: b"hello over an offline relay".to_vec(),
    };

    relay_queue.enqueue(message).expect("queue has capacity");
    let carried = relay_queue.pop().expect("message persisted by relay");
    let forwarded = carried.forwarded().expect("hop budget remains");
    if delivered.insert_if_new(forwarded.id) {
        println!(
            "LCF simulation: {} -> relay -> {} | payload={} bytes | remaining_hops={}",
            forwarded.source,
            forwarded.destination,
            forwarded.payload.len(),
            forwarded.hop_limit
        );
    }
}
