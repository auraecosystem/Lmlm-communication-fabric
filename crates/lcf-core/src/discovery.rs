//! Lease-based peer discovery registry. Advertisements are hints, not trust proofs.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerAdvertisement {
    pub node_id: String,
    pub sequence: u64,
    pub observed_at_ms: u64,
    pub lease_ms: u64,
    pub transports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryError { EmptyNodeId, InvalidLease, StaleSequence }

#[derive(Debug, Default)]
pub struct PeerRegistry { peers: HashMap<String, PeerAdvertisement> }

impl PeerRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn observe(&mut self, ad: PeerAdvertisement) -> Result<(), DiscoveryError> {
        if ad.node_id.trim().is_empty() { return Err(DiscoveryError::EmptyNodeId); }
        if ad.lease_ms == 0 { return Err(DiscoveryError::InvalidLease); }
        if let Some(old) = self.peers.get(&ad.node_id) {
            if ad.sequence <= old.sequence { return Err(DiscoveryError::StaleSequence); }
        }
        self.peers.insert(ad.node_id.clone(), ad);
        Ok(())
    }

    pub fn is_live(&self, node_id: &str, now_ms: u64) -> bool {
        self.peers.get(node_id).map(|p| now_ms < p.observed_at_ms.saturating_add(p.lease_ms)).unwrap_or(false)
    }

    pub fn get(&self, node_id: &str, now_ms: u64) -> Option<&PeerAdvertisement> {
        self.peers.get(node_id).filter(|p| now_ms < p.observed_at_ms.saturating_add(p.lease_ms))
    }

    pub fn expire(&mut self, now_ms: u64) -> usize {
        let before = self.peers.len();
        self.peers.retain(|_, p| now_ms < p.observed_at_ms.saturating_add(p.lease_ms));
        before - self.peers.len()
    }

    pub fn len(&self) -> usize { self.peers.len() }
    pub fn is_empty(&self) -> bool { self.peers.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ad(seq:u64) -> PeerAdvertisement {
        PeerAdvertisement { node_id:"peer-1".into(), sequence:seq, observed_at_ms:100,
            lease_ms:500, transports:vec!["udp".into(),"ble".into()] }
    }
    #[test] fn peer_lease_expires() {
        let mut r=PeerRegistry::new(); r.observe(ad(1)).unwrap();
        assert!(r.is_live("peer-1",599)); assert!(!r.is_live("peer-1",600));
        assert_eq!(r.expire(600),1);
    }
    #[test] fn stale_advertisements_are_rejected() {
        let mut r=PeerRegistry::new(); r.observe(ad(4)).unwrap();
        assert_eq!(r.observe(ad(4)),Err(DiscoveryError::StaleSequence));
    }
}
