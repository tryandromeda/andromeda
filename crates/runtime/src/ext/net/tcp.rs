// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use super::address::NetAddr;
use super::error::NetError;

/// TCP stream wrapper for resource management
pub struct TcpStreamWrapper {
    pub stream: Arc<Mutex<TcpStream>>,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
}

impl TcpStreamWrapper {
    pub fn new(stream: TcpStream) -> Result<Self, NetError> {
        let local_addr = stream.local_addr().map_err(NetError::from)?;
        let remote_addr = stream.peer_addr().map_err(NetError::from)?;

        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
            local_addr,
            remote_addr,
        })
    }

    pub async fn read(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        let mut stream = self.stream.lock().await;
        stream.read(buf).await.map_err(NetError::from)
    }

    pub async fn write(&self, buf: &[u8]) -> Result<usize, NetError> {
        let mut stream = self.stream.lock().await;
        stream.write(buf).await.map_err(NetError::from)
    }

    pub async fn write_all(&self, buf: &[u8]) -> Result<(), NetError> {
        let mut stream = self.stream.lock().await;
        stream.write_all(buf).await.map_err(NetError::from)
    }

    pub async fn flush(&self) -> Result<(), NetError> {
        let mut stream = self.stream.lock().await;
        stream.flush().await.map_err(NetError::from)
    }

    pub async fn shutdown(&self) -> Result<(), NetError> {
        let mut stream = self.stream.lock().await;
        stream.shutdown().await.map_err(NetError::from)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Set TCP_NODELAY option
    pub async fn set_nodelay(&self, nodelay: bool) -> Result<(), NetError> {
        let stream = self.stream.lock().await;
        stream.set_nodelay(nodelay).map_err(NetError::from)
    }

    /// Set SO_KEEPALIVE option
    pub async fn set_keepalive(&self, keepalive: bool) -> Result<(), NetError> {
        use socket2::SockRef;

        let stream = self.stream.lock().await;
        let sock_ref = SockRef::from(&*stream);
        sock_ref
            .set_keepalive(keepalive)
            .map_err(|e| NetError::socket_error("set_keepalive", &e.to_string()))
    }

    /// Get the current TCP_NODELAY setting
    pub async fn nodelay(&self) -> Result<bool, NetError> {
        let stream = self.stream.lock().await;
        stream.nodelay().map_err(NetError::from)
    }

    /// Get the current SO_KEEPALIVE setting
    pub async fn keepalive(&self) -> Result<bool, NetError> {
        use socket2::SockRef;

        let stream = self.stream.lock().await;
        let sock_ref = SockRef::from(&*stream);
        sock_ref
            .keepalive()
            .map_err(|e| NetError::socket_error("get_keepalive", &e.to_string()))
    }
}

/// TCP listener wrapper for resource management
pub struct TcpListenerWrapper {
    pub listener: Arc<Mutex<TcpListener>>,
    pub local_addr: SocketAddr,
}

impl TcpListenerWrapper {
    pub fn new(listener: TcpListener) -> Result<Self, NetError> {
        let local_addr = listener.local_addr().map_err(NetError::from)?;

        Ok(Self {
            listener: Arc::new(Mutex::new(listener)),
            local_addr,
        })
    }

    pub async fn accept(&self) -> Result<TcpStreamWrapper, NetError> {
        let listener = self.listener.lock().await;
        let (stream, _addr) = listener.accept().await.map_err(NetError::from)?;
        TcpStreamWrapper::new(stream)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

/// TCP operations
pub struct TcpOps;

impl TcpOps {
    /// Connect to a remote TCP address
    pub async fn connect(addr: &NetAddr) -> Result<TcpStreamWrapper, NetError> {
        // Resolve address immediately to avoid holding non-Send errors across await points
        let socket_addr = if let Ok(sock_addr) = addr.to_socket_addr() {
            sock_addr
        } else {
            // Try to resolve hostname
            let host_port = format!("{}:{}", addr.hostname, addr.port);
            let mut addresses = tokio::net::lookup_host(&host_port)
                .await
                .map_err(|e| NetError::dns_failed(&addr.hostname, &e.to_string()))?;

            addresses
                .next()
                .ok_or_else(|| NetError::dns_failed(&addr.hostname, "No addresses found"))?
        };

        let stream = TcpStream::connect(socket_addr)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::ConnectionRefused => {
                    NetError::connection_refused(&format!("{}:{}", addr.hostname, addr.port))
                }
                std::io::ErrorKind::TimedOut => {
                    NetError::connection_timeout(&format!("{}:{}", addr.hostname, addr.port))
                }
                _ => NetError::from(e),
            })?;

        TcpStreamWrapper::new(stream)
    }

    /// Bind to a local address and listen for connections
    pub async fn listen(addr: &NetAddr) -> Result<TcpListenerWrapper, NetError> {
        let socket_addr = addr
            .to_socket_addr()
            .map_err(|_| NetError::invalid_address(&addr.to_string()))?;

        let listener = TcpListener::bind(socket_addr)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::AddrInUse => NetError::address_in_use(&addr.to_string()),
                std::io::ErrorKind::AddrNotAvailable => {
                    NetError::address_not_available(&addr.to_string())
                }
                std::io::ErrorKind::PermissionDenied => NetError::permission_denied("bind"),
                _ => NetError::from(e),
            })?;

        TcpListenerWrapper::new(listener)
    }
}
