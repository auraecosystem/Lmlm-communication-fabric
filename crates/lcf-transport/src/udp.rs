//! UDP adapter for local Wi-Fi/LAN operation; no internet route is required.
use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

pub struct UdpTransport {
    socket: UdpSocket,
    peers: HashMap<String, SocketAddr>,
}

impl UdpTransport {
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket, peers: HashMap::new() })
    }
    pub fn local_addr(&self) -> io::Result<SocketAddr> { self.socket.local_addr() }
    pub fn register_peer(&mut self, node_id: impl Into<String>, addr: SocketAddr) {
        self.peers.insert(node_id.into(), addr);
    }
    pub fn send(&self, node_id: &str, frame: &[u8]) -> io::Result<usize> {
        let addr = self.peers.get(node_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "peer not registered"))?;
        self.socket.send_to(frame, addr)
    }
    pub fn receive(&self, buffer: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buffer)
    }
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_nonblocking(false)?;
        self.socket.set_read_timeout(timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_udp_nodes_exchange_datagrams() {
        let a = UdpTransport::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let mut b = UdpTransport::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        a.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        b.register_peer("a", a.local_addr().unwrap());
        b.send("a", b"offline-packet").unwrap();
        let mut buf = [0u8; 64];
        let (n, _) = a.receive(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"offline-packet");
    }
}
