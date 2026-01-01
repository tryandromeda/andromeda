// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
use tokio::net::{UnixDatagram, UnixListener, UnixStream};
#[cfg(unix)]
use tokio::sync::Mutex;

use super::error::NetError;

/// Unix domain stream socket wrapper for resource management
#[cfg(unix)]
pub struct UnixStreamWrapper {
    pub stream: Arc<Mutex<UnixStream>>,
    pub local_addr: Option<String>,
    pub remote_addr: Option<String>,
}

#[cfg(unix)]
impl UnixStreamWrapper {
    pub fn new(stream: UnixStream) -> Result<Self, NetError> {
        let local_addr = stream
            .local_addr()
            .ok()
            .and_then(|addr| addr.as_pathname().map(|p| p.to_string_lossy().to_string()));
        let remote_addr = stream
            .peer_addr()
            .ok()
            .and_then(|addr| addr.as_pathname().map(|p| p.to_string_lossy().to_string()));

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

    pub fn local_addr(&self) -> Option<&str> {
        self.local_addr.as_deref()
    }

    pub fn remote_addr(&self) -> Option<&str> {
        self.remote_addr.as_deref()
    }
}

/// Unix domain listener wrapper for resource management
#[cfg(unix)]
pub struct UnixListenerWrapper {
    pub listener: Arc<Mutex<UnixListener>>,
    pub local_addr: Option<String>,
}

#[cfg(unix)]
impl UnixListenerWrapper {
    pub fn new(listener: UnixListener) -> Result<Self, NetError> {
        let local_addr = listener
            .local_addr()
            .ok()
            .and_then(|addr| addr.as_pathname().map(|p| p.to_string_lossy().to_string()));

        Ok(Self {
            listener: Arc::new(Mutex::new(listener)),
            local_addr,
        })
    }

    pub async fn accept(&self) -> Result<UnixStreamWrapper, NetError> {
        let listener = self.listener.lock().await;
        let (stream, _addr) = listener.accept().await.map_err(NetError::from)?;
        UnixStreamWrapper::new(stream)
    }

    pub fn local_addr(&self) -> Option<&str> {
        self.local_addr.as_deref()
    }
}

/// Unix domain datagram socket wrapper for resource management
#[cfg(unix)]
pub struct UnixDatagramWrapper {
    pub socket: Arc<Mutex<UnixDatagram>>,
    pub local_addr: Option<String>,
}

#[cfg(unix)]
impl UnixDatagramWrapper {
    pub fn new(socket: UnixDatagram) -> Result<Self, NetError> {
        let local_addr = socket
            .local_addr()
            .ok()
            .and_then(|addr| addr.as_pathname().map(|p| p.to_string_lossy().to_string()));

        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
            local_addr,
        })
    }

    pub async fn send_to(&self, buf: &[u8], target: &str) -> Result<usize, NetError> {
        let socket = self.socket.lock().await;
        socket.send_to(buf, target).await.map_err(NetError::from)
    }

    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, Option<String>), NetError> {
        let socket = self.socket.lock().await;
        let (size, addr) = socket.recv_from(buf).await.map_err(NetError::from)?;
        let addr_str = addr.as_pathname().map(|p| p.to_string_lossy().to_string());
        Ok((size, addr_str))
    }

    pub async fn connect(&self, addr: &str) -> Result<(), NetError> {
        let socket = self.socket.lock().await;
        socket.connect(addr).map_err(NetError::from)
    }

    pub async fn send(&self, buf: &[u8]) -> Result<usize, NetError> {
        let socket = self.socket.lock().await;
        socket.send(buf).await.map_err(NetError::from)
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize, NetError> {
        let socket = self.socket.lock().await;
        socket.recv(buf).await.map_err(NetError::from)
    }

    pub fn local_addr(&self) -> Option<&str> {
        self.local_addr.as_deref()
    }
}

/// Unix domain socket operations
pub struct UnixOps;

#[cfg(unix)]
impl UnixOps {
    /// Connect to a Unix domain socket
    pub async fn connect(path: &str) -> Result<UnixStreamWrapper, NetError> {
        let stream = UnixStream::connect(path).await.map_err(|e| {
            match e.kind() {
                std::io::ErrorKind::NotFound => NetError::resource_not_found(0), // No specific RID available
                std::io::ErrorKind::ConnectionRefused => NetError::connection_refused(path),
                std::io::ErrorKind::PermissionDenied => NetError::permission_denied("connect"),
                _ => NetError::from(e),
            }
        })?;

        UnixStreamWrapper::new(stream)
    }

    /// Bind to a Unix domain socket path and listen for connections
    pub async fn listen(path: &str) -> Result<UnixListenerWrapper, NetError> {
        if Path::new(path).exists() {
            std::fs::remove_file(path).map_err(|e| NetError::io_error(&e.to_string()))?;
        }

        let listener = UnixListener::bind(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::AddrInUse => NetError::address_in_use(path),
            std::io::ErrorKind::PermissionDenied => NetError::permission_denied("bind"),
            _ => NetError::from(e),
        })?;

        UnixListenerWrapper::new(listener)
    }

    /// Bind to a Unix domain datagram socket
    pub async fn bind_datagram(path: &str) -> Result<UnixDatagramWrapper, NetError> {
        // Remove existing socket file if it exists
        if Path::new(path).exists() {
            std::fs::remove_file(path).map_err(|e| NetError::io_error(&e.to_string()))?;
        }

        let socket = UnixDatagram::bind(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::AddrInUse => NetError::address_in_use(path),
            std::io::ErrorKind::PermissionDenied => NetError::permission_denied("bind"),
            _ => NetError::from(e),
        })?;

        UnixDatagramWrapper::new(socket)
    }

    /// Create an unbound Unix datagram socket
    pub async fn datagram_unbound() -> Result<UnixDatagramWrapper, NetError> {
        let socket = UnixDatagram::unbound().map_err(NetError::from)?;
        UnixDatagramWrapper::new(socket)
    }

    /// Send data to a specific Unix domain socket path
    pub async fn send_to(
        socket: &UnixDatagramWrapper,
        data: &[u8],
        target: &str,
    ) -> Result<usize, NetError> {
        socket.send_to(data, target).await
    }

    /// Receive data from any source
    pub async fn recv_from(
        socket: &UnixDatagramWrapper,
        buf: &mut [u8],
    ) -> Result<(usize, Option<String>), NetError> {
        socket.recv_from(buf).await
    }

    /// Send data on a connected socket
    pub async fn send(socket: &UnixDatagramWrapper, data: &[u8]) -> Result<usize, NetError> {
        socket.send(data).await
    }

    /// Receive data on a connected socket
    pub async fn recv(socket: &UnixDatagramWrapper, buf: &mut [u8]) -> Result<usize, NetError> {
        socket.recv(buf).await
    }
}

#[cfg(not(unix))]
impl UnixOps {
    pub async fn connect(_path: &str) -> Result<(), NetError> {
        Err(NetError::not_supported(
            "Unix domain sockets on non-Unix platforms",
        ))
    }

    pub async fn listen(_path: &str) -> Result<(), NetError> {
        Err(NetError::not_supported(
            "Unix domain sockets on non-Unix platforms",
        ))
    }

    pub async fn bind_datagram(_path: &str) -> Result<(), NetError> {
        Err(NetError::not_supported(
            "Unix domain sockets on non-Unix platforms",
        ))
    }

    pub async fn datagram_unbound() -> Result<(), NetError> {
        Err(NetError::not_supported(
            "Unix domain sockets on non-Unix platforms",
        ))
    }
}

#[cfg(not(unix))]
pub struct UnixStreamWrapper;

#[cfg(not(unix))]
pub struct UnixListenerWrapper;

#[cfg(not(unix))]
pub struct UnixDatagramWrapper;
