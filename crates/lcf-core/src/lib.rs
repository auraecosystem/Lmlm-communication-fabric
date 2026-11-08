//! Deterministic transport-neutral LCF primitives.
pub mod delivery;
pub mod discovery;
pub mod routing;
pub mod storage;
use std::collections::{HashSet,VecDeque};
pub type MessageId=[u8;16];
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Envelope{pub id:MessageId,pub source:String,pub destination:String,pub created_at_ms:u64,pub expires_at_ms:u64,pub hop_limit:u8,pub payload:Vec<u8>}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum RejectReason{EmptyIdentity,Expired,HopLimitExceeded,PayloadTooLarge,InvalidLifetime}
impl Envelope{
 pub fn validate(&self,now:u64,max_payload:usize)->Result<(),RejectReason>{
  if self.source.trim().is_empty()||self.destination.trim().is_empty(){return Err(RejectReason::EmptyIdentity);}
  if self.expires_at_ms<=self.created_at_ms{return Err(RejectReason::InvalidLifetime);}
  if now>=self.expires_at_ms{return Err(RejectReason::Expired);}
  if self.hop_limit==0{return Err(RejectReason::HopLimitExceeded);}
  if self.payload.len()>max_payload{return Err(RejectReason::PayloadTooLarge);}Ok(())
 }
 pub fn forwarded(&self)->Option<Self>{if self.hop_limit<=1{return None;}let mut n=self.clone();n.hop_limit-=1;Some(n)}
}
#[derive(Debug)]pub struct DuplicateCache{capacity:usize,order:VecDeque<MessageId>,seen:HashSet<MessageId>}
impl DuplicateCache{
 pub fn new(capacity:usize)->Self{Self{capacity,order:VecDeque::new(),seen:HashSet::new()}}
 pub fn insert_if_new(&mut self,id:MessageId)->bool{if self.seen.contains(&id){return false;}if self.capacity==0{return true;}self.seen.insert(id);self.order.push_back(id);while self.order.len()>self.capacity{if let Some(old)=self.order.pop_front(){self.seen.remove(&old);}}true}
}
#[derive(Debug)]pub struct MessageQueue{max_items:usize,max_bytes:usize,used_bytes:usize,queue:VecDeque<Envelope>}
impl MessageQueue{
 pub fn new(max_items:usize,max_bytes:usize)->Self{Self{max_items,max_bytes,used_bytes:0,queue:VecDeque::new()}}
 pub fn enqueue(&mut self,m:Envelope)->Result<(),Envelope>{let n=m.payload.len();if self.queue.len()>=self.max_items||self.used_bytes.saturating_add(n)>self.max_bytes{return Err(m);}self.used_bytes+=n;self.queue.push_back(m);Ok(())}
 pub fn pop(&mut self)->Option<Envelope>{let m=self.queue.pop_front()?;self.used_bytes=self.used_bytes.saturating_sub(m.payload.len());Some(m)}
 pub fn len(&self)->usize{self.queue.len()}pub fn is_empty(&self)->bool{self.queue.is_empty()}
}
#[cfg(test)]mod tests{use super::*;fn sample(h:u8)->Envelope{Envelope{id:[7;16],source:"a".into(),destination:"c".into(),created_at_ms:1,expires_at_ms:100,hop_limit:h,payload:vec![1,2,3]}}
#[test]fn expiry(){assert!(sample(2).validate(2,10).is_ok());assert_eq!(sample(2).validate(100,10),Err(RejectReason::Expired));}
#[test]fn hop_limit(){assert_eq!(sample(2).forwarded().unwrap().hop_limit,1);assert!(sample(1).forwarded().is_none());}
#[test]fn dedup(){let mut c=DuplicateCache::new(2);assert!(c.insert_if_new([1;16]));assert!(!c.insert_if_new([1;16]));}
#[test]fn queue_bounds(){let mut q=MessageQueue::new(1,3);assert!(q.enqueue(sample(2)).is_ok());assert!(q.enqueue(sample(2)).is_err());assert!(q.pop().is_some());}}
