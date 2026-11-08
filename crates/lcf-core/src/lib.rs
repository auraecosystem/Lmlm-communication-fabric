//! Deterministic, transport-neutral primitives for LMLM Communication Fabric.

use std::collections::{HashSet, VecDeque};

/// Stable identifier for a message. Production deployments should use a
/// cryptographically random 128-bit value; this scaffold keeps it opaque.
pub type MessageId = [u8; 16];

/// Minimal transport-neutral message envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub id: MessageId,
    pub source: String,
    pub destination: String,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub hop_limit: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    EmptyIdentity,
    Expired,
    HopLimitExceeded,
    PayloadTooLarge,
    InvalidLifetime,
}

impl Envelope {
    /// Validate bounds before accepting a message into queues or forwarding.
    pub fn validate(&self, now_ms: u64, max_payload_bytes: usize) -> Result<(), RejectReason> {
        if self.source.trim().is_empty() || self.destination.trim().is_empty() {
            return Err(RejectReason::EmptyIdentity);
        }
        if self.expires_at_ms <= self.created_at_ms {
            return Err(RejectReason::InvalidLifetime);
        }
        if now_ms >= self.expires_at_ms {
            return Err(RejectReason::Expired);
        }
        if self.hop_limit == 0 {
            return Err(RejectReason::HopLimitExceeded);
        }
        if self.payload.len() > max_payload_bytes {
            return Err(RejectReason::PayloadTooLarge);
        }
        Ok(())
    }

    /// Decrement hop budget exactly once when forwarding to another peer.
    pub fn forwarded(&self) -> Option<Self> {
        if self.hop_limit <= 1 {
            return None;
        }
        let mut next = self.clone();
        next.hop_limit -= 1;
        Some(next)
    }
}

/// Bounded duplicate cache to prevent loops and repeated delivery.
#[derive(Debug)]
pub struct DuplicateCache {
    capacity: usize,
    order: VecDeque<MessageId>,
    seen: HashSet<MessageId>,
}

impl DuplicateCache {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, order: VecDeque::new(), seen: HashSet::new() }
    }

    /// Returns true only for a newly observed message ID.
    pub fn insert_if_new(&mut self, id: MessageId) -> bool {
        if self.seen.contains(&id) {
            return false;
        }
        if self.capacity == 0 {
            return true;
        }
        self.seen.insert(id);
        self.order.push_back(id);
        while self.order.len() > self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }
        true
    }
}

/// Store-carry-forward queue with explicit byte and item limits.
#[derive(Debug)]
pub struct MessageQueue {
    max_items: usize,
    max_bytes: usize,
    used_bytes: usize,
    queue: VecDeque<Envelope>,
}

impl MessageQueue {
    pub fn new(max_items: usize, max_bytes: usize) -> Self {
        Self { max_items, max_bytes, used_bytes: 0, queue: VecDeque::new() }
    }

    pub fn enqueue(&mut self, message: Envelope) -> Result<(), Envelope> {
        let size = message.payload.len();
        if self.queue.len() >= self.max_items || self.used_bytes.saturating_add(size) > self.max_bytes {
            return Err(message);
        }
        self.used_bytes += size;
        self.queue.push_back(message);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Envelope> {
        let message = self.queue.pop_front()?;
        self.used_bytes = self.used_bytes.saturating_sub(message.payload.len());
        Some(message)
    }

    pub fn len(&self) -> usize { self.queue.len() }
    pub fn is_empty(&self) -> bool { self.queue.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(hop_limit: u8) -> Envelope {
        Envelope {
            id: [7; 16], source: "node-a".into(), destination: "node-c".into(),
            created_at_ms: 10, expires_at_ms: 1_000, hop_limit, payload: vec![1, 2, 3],
        }
    }

    #[test]
    fn validates_and_rejects_expired_messages() {
        assert!(sample(3).validate(100, 64).is_ok());
        assert_eq!(sample(3).validate(1_000, 64), Err(RejectReason::Expired));
    }

    #[test]
    fn forwarding_decrements_hop_limit() {
        assert_eq!(sample(3).forwarded().unwrap().hop_limit, 2);
        assert!(sample(1).forwarded().is_none());
    }

    #[test]
    fn duplicate_cache_suppresses_replays() {
        let mut cache = DuplicateCache::new(2);
        assert!(cache.insert_if_new([1; 16]));
        assert!(!cache.insert_if_new([1; 16]));
    }

    #[test]
    fn queue_enforces_capacity_and_releases_bytes() {
        let mut queue = MessageQueue::new(1, 3);
        assert!(queue.enqueue(sample(2)).is_ok());
        assert!(queue.enqueue(sample(2)).is_err());
        assert!(queue.pop().is_some());
        assert_eq!(queue.len(), 0);
    }
}
