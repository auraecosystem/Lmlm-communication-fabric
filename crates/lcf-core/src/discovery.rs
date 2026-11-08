//! Lease-based discovery. Advertisements are hints, not identity proofs.
use std::collections::HashMap;
#[derive(Debug,Clone,PartialEq,Eq)]pub struct PeerAdvertisement{pub node_id:String,pub sequence:u64,pub observed_at_ms:u64,pub lease_ms:u64,pub transports:Vec<String>}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum DiscoveryError{EmptyNodeId,InvalidLease,StaleSequence}
#[derive(Debug,Default)]pub struct PeerRegistry{peers:HashMap<String,PeerAdvertisement>}
impl PeerRegistry{
 pub fn new()->Self{Self::default()}
 pub fn observe(&mut self,a:PeerAdvertisement)->Result<(),DiscoveryError>{if a.node_id.trim().is_empty(){return Err(DiscoveryError::EmptyNodeId);}if a.lease_ms==0{return Err(DiscoveryError::InvalidLease);}if let Some(old)=self.peers.get(&a.node_id){if a.sequence<=old.sequence{return Err(DiscoveryError::StaleSequence);}}self.peers.insert(a.node_id.clone(),a);Ok(())}
 pub fn get(&self,id:&str,now:u64)->Option<&PeerAdvertisement>{self.peers.get(id).filter(|p|now<p.observed_at_ms.saturating_add(p.lease_ms))}
 pub fn is_live(&self,id:&str,now:u64)->bool{self.get(id,now).is_some()}
 pub fn expire(&mut self,now:u64)->usize{let n=self.peers.len();self.peers.retain(|_,p|now<p.observed_at_ms.saturating_add(p.lease_ms));n-self.peers.len()}
 pub fn len(&self)->usize{self.peers.len()}pub fn is_empty(&self)->bool{self.peers.is_empty()}
}
#[cfg(test)]mod tests{use super::*;fn ad(s:u64)->PeerAdvertisement{PeerAdvertisement{node_id:"p".into(),sequence:s,observed_at_ms:10,lease_ms:20,transports:vec!["udp".into()]}}
#[test]fn expires_lease(){let mut r=PeerRegistry::new();r.observe(ad(1)).unwrap();assert!(r.is_live("p",29));assert_eq!(r.expire(30),1);}
#[test]fn rejects_stale_sequence(){let mut r=PeerRegistry::new();r.observe(ad(2)).unwrap();assert_eq!(r.observe(ad(2)),Err(DiscoveryError::StaleSequence));}}
