// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use super::address::NetAddr;
use super::error::NetError;

/// UDP socket wrapper for resource management
pub struct UdpSocketWrapper {
    pub socket: Arc<Mutex<UdpSocket>>,
    pub local_addr: SocketAddr,
}

impl UdpSocketWrapper {
    pub fn new(socket: UdpSocket) -> Result<Self, NetError> {
        let local_addr = socket.local_addr().map_err(NetError::from)?;

        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
            local_addr,
        })
    }

    pub async fn send_to(&self, buf: &[u8], target: SocketAddr) -> Result<usize, NetError> {
        let socket = self.socket.lock().await;
        socket.send_to(buf, target).await.map_err(NetError::from)
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), NetError> {
        let socket = self.socket.lock().await;
        socket.recv_from(buf).await.map_err(NetError::from)
    }

    pub async fn connect(&self, addr: SocketAddr) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket.connect(addr).await.map_err(NetError::from)
    }

    pub async fn send(&self, buf: &[u8]) -> Result<usize, NetError> {
        let socket = self.socket.lock().await;
        socket.send(buf).await.map_err(NetError::from)
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        let socket = self.socket.lock().await;
        socket.recv(buf).await.map_err(NetError::from)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    // Multicast operations
    pub async fn join_multicast_v4(
        &self,
        multiaddr: Ipv4Addr,
        interface: Ipv4Addr,
    ) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket
            .join_multicast_v4(multiaddr, interface)
            .map_err(NetError::from)
    }

    pub async fn join_multicast_v6(
        &self,
        multiaddr: &Ipv6Addr,
        interface: u32,
    ) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket
            .join_multicast_v6(multiaddr, interface)
            .map_err(NetError::from)
    }

    pub async fn leave_multicast_v4(
        &self,
        multiaddr: Ipv4Addr,
        interface: Ipv4Addr,
    ) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket
            .leave_multicast_v4(multiaddr, interface)
            .map_err(NetError::from)
    }

    pub async fn leave_multicast_v6(
        &self,
        multiaddr: &Ipv6Addr,
        interface: u32,
    ) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket
            .leave_multicast_v6(multiaddr, interface)
            .map_err(NetError::from)
    }

    pub async fn set_multicast_loop_v4(&self, on: bool) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket.set_multicast_loop_v4(on).map_err(NetError::from)
    }

    pub async fn set_multicast_loop_v6(&self, on: bool) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket.set_multicast_loop_v6(on).map_err(NetError::from)
    }

    pub async fn set_multicast_ttl_v4(&self, ttl: u32) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket.set_multicast_ttl_v4(ttl).map_err(NetError::from)
    }

    pub async fn set_broadcast(&self, on: bool) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket.set_broadcast(on).map_err(NetError::from)
    }

    pub async fn set_ttl(&self, ttl: u32) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket.set_ttl(ttl).map_err(NetError::from)
    }

    /// Get the current TTL setting
    pub async fn ttl(&self) -> Result<u32, NetError> {
        let socket = self.socket.lock().await;
        socket.ttl().map_err(NetError::from)
    }

    /// Set socket receive buffer size
    pub async fn set_recv_buffer_size(&self, size: u32) -> Result<(), NetError> {
        use socket2::SockRef;

        let socket = self.socket.lock().await;
        let sock_ref = SockRef::from(&*socket);
        sock_ref
            .set_recv_buffer_size(size as usize)
            .map_err(|e| NetError::socket_error("set_recv_buffer_size", &e.to_string()))
    }

    /// Set socket send buffer size
    pub async fn set_send_buffer_size(&self, size: u32) -> Result<(), NetError> {
        use socket2::SockRef;

        let socket = self.socket.lock().await;
        let sock_ref = SockRef::from(&*socket);
        sock_ref
            .set_send_buffer_size(size as usize)
            .map_err(|e| NetError::socket_error("set_send_buffer_size", &e.to_string()))
    }
}

/// UDP operations
pub struct UdpOps;

impl UdpOps {
    /// Bind to a local address for UDP communication
    pub async fn bind(addr: &NetAddr) -> Result<UdpSocketWrapper, NetError> {
        let socket_addr = addr
            .to_socket_addr()
            .map_err(|_| NetError::invalid_address(&addr.to_string()))?;

        let socket = UdpSocket::bind(socket_addr)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::AddrInUse => NetError::address_in_use(&addr.to_string()),
                std::io::ErrorKind::AddrNotAvailable => {
                    NetError::address_not_available(&addr.to_string())
                }
                std::io::ErrorKind::PermissionDenied => NetError::permission_denied("bind"),
                _ => NetError::from(e),
            })?;

        UdpSocketWrapper::new(socket)
    }

    /// Create a UDP socket and connect it to a remote address
    pub async fn connect(
        local_addr: &NetAddr,
        remote_addr: &NetAddr,
    ) -> Result<UdpSocketWrapper, NetError> {
        let socket = Self::bind(local_addr).await?;

        let remote_socket_addr = match remote_addr.to_socket_addr() {
            Ok(addr) => addr,
            Err(_) => {
                // Try to resolve hostname
                let host_port = format!("{}:{}", remote_addr.hostname, remote_addr.port);
                let mut addresses = tokio::net::lookup_host(&host_port)
                    .await
                    .map_err(|e| NetError::dns_failed(&remote_addr.hostname, &e.to_string()))?;

                addresses.next().ok_or_else(|| {
                    NetError::dns_failed(&remote_addr.hostname, "No addresses found")
                })?
            }
        };

        socket.connect(remote_socket_addr).await?;
        Ok(socket)
    }

    /// Send data to a specific address
    pub async fn send_to(
        socket: &UdpSocketWrapper,
        data: &[u8],
        target: &NetAddr,
    ) -> Result<usize, NetError> {
        let target_addr = match target.to_socket_addr() {
            Ok(addr) => addr,
            Err(_) => {
                // Try to resolve hostname
                let host_port = format!("{}:{}", target.hostname, target.port);
                let mut addresses = tokio::net::lookup_host(&host_port)
                    .await
                    .map_err(|e| NetError::dns_failed(&target.hostname, &e.to_string()))?;

                addresses
                    .next()
                    .ok_or_else(|| NetError::dns_failed(&target.hostname, "No addresses found"))?
            }
        };

        socket.send_to(data, target_addr).await
    }

    /// Receive data from any source
    pub async fn recv_from(
        socket: &UdpSocketWrapper,
        buf: &mut [u8],
    ) -> Result<(usize, NetAddr), NetError> {
        let (size, addr) = socket.recv_from(buf).await?;
        Ok((size, NetAddr::from_socket_addr(addr)))
    }

    /// Send data on a connected socket
    pub async fn send(socket: &UdpSocketWrapper, data: &[u8]) -> Result<usize, NetError> {
        socket.send(data).await
    }

    /// Receive data on a connected socket
    pub async fn recv(socket: &UdpSocketWrapper, buf: &mut [u8]) -> Result<usize, NetError> {
        socket.recv(buf).await
    }

    /// Validate multicast address
    pub fn validate_multicast_addr(addr: &str, interface: &str) -> Result<(), NetError> {
        let multicast_addr: IpAddr = addr.parse().map_err(|_| NetError::invalid_address(addr))?;

        if !multicast_addr.is_multicast() {
            return Err(NetError::invalid_address(&format!(
                "Address {} is not a multicast address",
                addr
            )));
        }

        let _interface_addr: IpAddr = interface
            .parse()
            .map_err(|_| NetError::invalid_address(interface))?;

        match (multicast_addr, _interface_addr) {
            (IpAddr::V4(_), IpAddr::V4(_)) => Ok(()),
            (IpAddr::V6(_), IpAddr::V6(_)) => Ok(()),
            _ => Err(NetError::invalid_address(
                "Multicast address and interface address families do not match",
            )),
        }
    }
}
