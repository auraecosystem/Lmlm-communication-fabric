//! Small deterministic route table; route authorization is handled separately.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub destination: String,
    pub next_hop: String,
    pub metric: u32,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError { EmptyAddress, ZeroMetric }

#[derive(Debug, Default)]
pub struct RouteTable { routes: HashMap<String, Route> }

impl RouteTable {
    pub fn new() -> Self { Self::default() }

    /// Prefer lower-cost routes. Equal-cost advertisements refresh the lease.
    pub fn advertise(&mut self, route: Route) -> Result<(), RouteError> {
        if route.destination.trim().is_empty() || route.next_hop.trim().is_empty() {
            return Err(RouteError::EmptyAddress);
        }
        if route.metric == 0 { return Err(RouteError::ZeroMetric); }
        let replace = self.routes.get(&route.destination)
            .map(|old| route.metric <= old.metric || old.expires_at_ms == 0)
            .unwrap_or(true);
        if replace { self.routes.insert(route.destination.clone(), route); }
        Ok(())
    }

    pub fn lookup(&self, destination: &str, now_ms: u64) -> Option<&Route> {
        self.routes.get(destination).filter(|r| now_ms < r.expires_at_ms)
    }

    pub fn remove_neighbor(&mut self, neighbor: &str) -> usize {
        let before=self.routes.len();
        self.routes.retain(|_,r| r.next_hop != neighbor);
        before-self.routes.len()
    }

    pub fn expire(&mut self, now_ms:u64) -> usize {
        let before=self.routes.len();
        self.routes.retain(|_,r| now_ms < r.expires_at_ms);
        before-self.routes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn route(metric:u32, hop:&str)->Route {
        Route { destination:"node-z".into(), next_hop:hop.into(), metric, expires_at_ms:900 }
    }
    #[test] fn lower_metric_replaces_route() {
        let mut t=RouteTable::new(); t.advertise(route(5,"b")).unwrap(); t.advertise(route(2,"c")).unwrap();
        assert_eq!(t.lookup("node-z",100).unwrap().next_hop,"c");
    }
    #[test] fn neighbor_loss_invalidates_routes() {
        let mut t=RouteTable::new(); t.advertise(route(1,"relay")).unwrap();
        assert_eq!(t.remove_neighbor("relay"),1); assert!(t.lookup("node-z",100).is_none());
    }
}
