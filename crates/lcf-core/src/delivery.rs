//! Bounded acknowledgement and retry tracking for reliable message delivery.
use lcf_core::MessageId;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingDelivery {
    pub message_id: MessageId,
    pub destination: String,
    pub attempts: u8,
    pub next_attempt_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryError {
    EmptyDestination,
    InvalidLifetime,
    Capacity,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryState {
    Pending(PendingDelivery),
    Acknowledged,
    Exhausted,
}

/// Tracks outbound deliveries. The caller performs transport I/O and calls
/// acknowledge() only after receiving an authenticated acknowledgement.
pub struct DeliveryTracker {
    capacity: usize,
    max_attempts: u8,
    base_backoff_ms: u64,
    pending: HashMap<MessageId, PendingDelivery>,
}

impl DeliveryTracker {
    pub fn new(capacity: usize, max_attempts: u8, base_backoff_ms: u64) -> Self {
        Self { capacity, max_attempts: max_attempts.max(1), base_backoff_ms: base_backoff_ms.max(1), pending: HashMap::new() }
    }

    pub fn register(&mut self, id: MessageId, destination: String, now_ms: u64, expires_at_ms: u64) -> Result<(), DeliveryError> {
        if destination.trim().is_empty() { return Err(DeliveryError::EmptyDestination); }
        if expires_at_ms <= now_ms { return Err(DeliveryError::InvalidLifetime); }
        if self.pending.contains_key(&id) { return Err(DeliveryError::Duplicate); }
        if self.pending.len() >= self.capacity { return Err(DeliveryError::Capacity); }
        self.pending.insert(id, PendingDelivery {
            message_id: id, destination, attempts: 0,
            next_attempt_at_ms: now_ms, expires_at_ms,
        });
        Ok(())
    }

    /// Returns due messages and advances their retry schedule. Exponential
    /// backoff is capped to avoid overflow and never schedules beyond expiry.
    pub fn due(&mut self, now_ms: u64) -> Vec<PendingDelivery> {
        let mut ready = Vec::new();
        let ids: Vec<MessageId> = self.pending.keys().copied().collect();
        for id in ids {
            let mut remove = false;
            if let Some(item) = self.pending.get_mut(&id) {
                if now_ms >= item.expires_at_ms || item.attempts >= self.max_attempts {
                    remove = true;
                } else if now_ms >= item.next_attempt_at_ms {
                    item.attempts = item.attempts.saturating_add(1);
                    ready.push(item.clone());
                    let shift = u32::from(item.attempts.saturating_sub(1)).min(20);
                    let delay = self.base_backoff_ms.saturating_mul(1u64 << shift);
                    item.next_attempt_at_ms = now_ms.saturating_add(delay).min(item.expires_at_ms);
                }
            }
            if remove { self.pending.remove(&id); }
        }
        ready
    }

    pub fn acknowledge(&mut self, id: &MessageId) -> bool {
        self.pending.remove(id).is_some()
    }

    pub fn contains(&self, id: &MessageId) -> bool { self.pending.contains_key(id) }
    pub fn len(&self) -> usize { self.pending.len() }
    pub fn is_empty(&self) -> bool { self.pending.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retries_with_backoff_until_acknowledged() {
        let mut tracker = DeliveryTracker::new(4, 3, 100);
        let id = [5u8; 16];
        tracker.register(id, "peer-b".into(), 0, 10_000).unwrap();
        assert_eq!(tracker.due(0)[0].attempts, 1);
        assert!(tracker.due(99).is_empty());
        assert_eq!(tracker.due(100)[0].attempts, 2);
        assert!(tracker.acknowledge(&id));
        assert!(!tracker.contains(&id));
    }

    #[test]
    fn capacity_and_expiry_are_enforced() {
        let mut tracker = DeliveryTracker::new(1, 2, 10);
        let a = [1u8; 16];
        let b = [2u8; 16];
        tracker.register(a, "a".into(), 0, 100).unwrap();
        assert_eq!(tracker.register(b, "b".into(), 0, 100), Err(DeliveryError::Capacity));
        assert!(tracker.due(100).is_empty());
        assert!(tracker.is_empty());
    }
}
