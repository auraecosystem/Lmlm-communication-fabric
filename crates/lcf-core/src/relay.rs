//! Deterministic relay decision engine joining envelope validation, dedup and routes.
use crate::{DuplicateCache, Envelope, RejectReason};
use crate::routing::{Route, RouteError, RouteTable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayError {
    InvalidEnvelope(RejectReason),
    Duplicate,
    NoRoute,
    HopLimitExceeded,
    RouteRejected(RouteError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayAction {
    DeliverLocal(Envelope),
    Forward { next_hop: String, envelope: Envelope },
}

pub struct RelayNode {
    id: String,
    max_payload: usize,
    duplicates: DuplicateCache,
    routes: RouteTable,
}

impl RelayNode {
    pub fn new(id: String, max_payload: usize, duplicate_capacity: usize) -> Self {
        Self { id, max_payload, duplicates: DuplicateCache::new(duplicate_capacity), routes: RouteTable::new() }
    }

    pub fn advertise_route(&mut self, route: Route) -> Result<(), RelayError> {
        self.routes.advertise(route).map_err(RelayError::RouteRejected)
    }

    pub fn remove_neighbor(&mut self, peer_id: &str) -> usize { self.routes.remove_neighbor(peer_id) }

    pub fn process(&mut self, envelope: Envelope, now_ms: u64) -> Result<RelayAction, RelayError> {
        envelope.validate(now_ms, self.max_payload).map_err(RelayError::InvalidEnvelope)?;
        if envelope.destination == self.id {
            if !self.duplicates.insert_if_new(envelope.id) { return Err(RelayError::Duplicate); }
            return Ok(RelayAction::DeliverLocal(envelope));
        }
        let route = self.routes.lookup(&envelope.destination, now_ms).ok_or(RelayError::NoRoute)?;
        let next_hop = route.next_hop.clone();
        let forwarded = envelope.forwarded().ok_or(RelayError::HopLimitExceeded)?;
        if !self.duplicates.insert_if_new(envelope.id) { return Err(RelayError::Duplicate); }
        Ok(RelayAction::Forward { next_hop, envelope: forwarded })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MessageId;

    fn message(id: MessageId, destination: &str, hops: u8) -> Envelope {
        Envelope { id, source: "origin".into(), destination: destination.into(), created_at_ms: 1, expires_at_ms: 1000, hop_limit: hops, payload: b"mesh".to_vec() }
    }

    #[test]
    fn forwards_using_route_and_decrements_hop_limit_once() {
        let mut node = RelayNode::new("relay-a".into(), 128, 32);
        node.advertise_route(Route { destination: "node-c".into(), next_hop: "relay-b".into(), metric: 2, expires_at_ms: 900 }).unwrap();
        match node.process(message([1; 16], "node-c", 3), 10).unwrap() {
            RelayAction::Forward { next_hop, envelope } => {
                assert_eq!(next_hop, "relay-b");
                assert_eq!(envelope.hop_limit, 2);
            }
            _ => panic!("expected forward action"),
        }
    }

    #[test]
    fn rejects_duplicate_and_expired_route() {
        let mut node = RelayNode::new("relay-a".into(), 128, 32);
        node.advertise_route(Route { destination: "node-c".into(), next_hop: "relay-b".into(), metric: 2, expires_at_ms: 20 }).unwrap();
        assert!(node.process(message([2; 16], "node-c", 2), 10).is_ok());
        assert_eq!(node.process(message([2; 16], "node-c", 2), 10), Err(RelayError::Duplicate));
        assert_eq!(node.process(message([3; 16], "node-c", 2), 20), Err(RelayError::NoRoute));
    }
}
