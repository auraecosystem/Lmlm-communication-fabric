//! Deterministic route table. Peer authorization is a separate concern.
use std::collections::HashMap;
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Route{pub destination:String,pub next_hop:String,pub metric:u32,pub expires_at_ms:u64}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum RouteError{EmptyAddress,ZeroMetric}
#[derive(Debug,Default)]pub struct RouteTable{routes:HashMap<String,Route>}
impl RouteTable{
 pub fn new()->Self{Self::default()}
 pub fn advertise(&mut self,r:Route)->Result<(),RouteError>{if r.destination.trim().is_empty()||r.next_hop.trim().is_empty(){return Err(RouteError::EmptyAddress);}if r.metric==0{return Err(RouteError::ZeroMetric);}let replace=self.routes.get(&r.destination).map(|old|r.metric<=old.metric||old.expires_at_ms==0).unwrap_or(true);if replace{self.routes.insert(r.destination.clone(),r);}Ok(())}
 pub fn lookup(&self,d:&str,now:u64)->Option<&Route>{self.routes.get(d).filter(|r|now<r.expires_at_ms)}
 pub fn remove_neighbor(&mut self,n:&str)->usize{let before=self.routes.len();self.routes.retain(|_,r|r.next_hop!=n);before-self.routes.len()}
 pub fn expire(&mut self,now:u64)->usize{let before=self.routes.len();self.routes.retain(|_,r|now<r.expires_at_ms);before-self.routes.len()}
}
#[cfg(test)]mod tests{use super::*;fn r(m:u32,n:&str)->Route{Route{destination:"z".into(),next_hop:n.into(),metric:m,expires_at_ms:100}}
#[test]fn lower_metric_wins(){let mut t=RouteTable::new();t.advertise(r(4,"b")).unwrap();t.advertise(r(2,"c")).unwrap();assert_eq!(t.lookup("z",1).unwrap().next_hop,"c");}
#[test]fn neighbor_loss_removes_routes(){let mut t=RouteTable::new();t.advertise(r(1,"relay")).unwrap();assert_eq!(t.remove_neighbor("relay"),1);}}
