// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

/// Network-specific errors for Andromeda networking operations
#[derive(Debug, Clone)]
pub enum NetError {
    /// Failed to resolve hostname
    DnsResolutionFailed(String),
    /// Connection refused
    ConnectionRefused(String),
    /// Connection timeout
    ConnectionTimeout(String),
    /// Invalid address format
    InvalidAddress(String),
    /// Invalid port number
    InvalidPort(u16),
    /// Socket operation failed
    SocketError(String),
    /// Resource not found
    ResourceNotFound(u32),
    /// Permission denied
    PermissionDenied(String),
    /// Network unreachable
    NetworkUnreachable(String),
    /// Host unreachable
    HostUnreachable(String),
    /// Connection reset by peer
    ConnectionReset(String),
    /// Broken pipe
    BrokenPipe(String),
    /// Address already in use
    AddressInUse(String),
    /// Address not available
    AddressNotAvailable(String),
    /// Operation not supported
    NotSupported(String),
    /// Interrupted system call
    Interrupted(String),
    /// Would block (for non-blocking operations)
    WouldBlock(String),
    /// Generic I/O error
    IoError(String),
}

impl NetError {
    /// Create a DNS resolution error
    pub fn dns_failed(hostname: &str, reason: &str) -> Self {
        NetError::DnsResolutionFailed(format!("Failed to resolve '{}': {}", hostname, reason))
    }

    /// Create a connection refused error
    pub fn connection_refused(addr: &str) -> Self {
        NetError::ConnectionRefused(format!("Connection refused to {}", addr))
    }

    /// Create a connection timeout error
    pub fn connection_timeout(addr: &str) -> Self {
        NetError::ConnectionTimeout(format!("Connection timeout to {}", addr))
    }

    /// Create an invalid address error
    pub fn invalid_address(addr: &str) -> Self {
        NetError::InvalidAddress(format!("Invalid address format: {}", addr))
    }

    /// Create an invalid port error
    pub fn invalid_port(port: u16) -> Self {
        NetError::InvalidPort(port)
    }

    /// Create a socket error
    pub fn socket_error(operation: &str, reason: &str) -> Self {
        NetError::SocketError(format!("Socket {} failed: {}", operation, reason))
    }

    /// Create a resource not found error
    pub fn resource_not_found(rid: u32) -> Self {
        NetError::ResourceNotFound(rid)
    }

    /// Create a permission denied error
    pub fn permission_denied(operation: &str) -> Self {
        NetError::PermissionDenied(format!("Permission denied for {}", operation))
    }

    /// Create a network unreachable error
    pub fn network_unreachable(addr: &str) -> Self {
        NetError::NetworkUnreachable(format!("Network unreachable: {}", addr))
    }

    /// Create a host unreachable error
    pub fn host_unreachable(addr: &str) -> Self {
        NetError::HostUnreachable(format!("Host unreachable: {}", addr))
    }

    /// Create a connection reset error
    pub fn connection_reset(addr: &str) -> Self {
        NetError::ConnectionReset(format!("Connection reset by peer: {}", addr))
    }

    /// Create a broken pipe error
    pub fn broken_pipe() -> Self {
        NetError::BrokenPipe("Broken pipe".to_string())
    }

    /// Create an address already in use error
    pub fn address_in_use(addr: &str) -> Self {
        NetError::AddressInUse(format!("Address already in use: {}", addr))
    }

    /// Create an address not available error
    pub fn address_not_available(addr: &str) -> Self {
        NetError::AddressNotAvailable(format!("Address not available: {}", addr))
    }

    /// Create a not supported error
    pub fn not_supported(operation: &str) -> Self {
        NetError::NotSupported(format!("Operation not supported: {}", operation))
    }

    /// Create an interrupted error
    pub fn interrupted(operation: &str) -> Self {
        NetError::Interrupted(format!("Interrupted: {}", operation))
    }

    /// Create a would block error
    pub fn would_block(operation: &str) -> Self {
        NetError::WouldBlock(format!("Would block: {}", operation))
    }

    /// Create a generic I/O error
    pub fn io_error(reason: &str) -> Self {
        NetError::IoError(reason.to_string())
    }

    /// Create a resource management error
    pub fn resource_error(reason: &str) -> Self {
        NetError::IoError(format!("Resource error: {}", reason))
    }

    /// Convert from std::io::Error to NetError
    pub fn from_io_error(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::ConnectionRefused => {
                NetError::IoError("Connection refused".to_string())
            }
            std::io::ErrorKind::ConnectionReset => {
                NetError::IoError("Connection reset".to_string())
            }
            std::io::ErrorKind::ConnectionAborted => {
                NetError::IoError("Connection aborted".to_string())
            }
            std::io::ErrorKind::NotConnected => NetError::IoError("Not connected".to_string()),
            std::io::ErrorKind::AddrInUse => NetError::IoError("Address in use".to_string()),
            std::io::ErrorKind::AddrNotAvailable => {
                NetError::IoError("Address not available".to_string())
            }
            std::io::ErrorKind::NetworkDown => NetError::IoError("Network down".to_string()),
            std::io::ErrorKind::NetworkUnreachable => {
                NetError::IoError("Network unreachable".to_string())
            }
            std::io::ErrorKind::BrokenPipe => NetError::IoError("Broken pipe".to_string()),
            std::io::ErrorKind::AlreadyExists => NetError::IoError("Already exists".to_string()),
            std::io::ErrorKind::WouldBlock => NetError::IoError("Would block".to_string()),
            std::io::ErrorKind::InvalidInput => NetError::IoError("Invalid input".to_string()),
            std::io::ErrorKind::InvalidData => NetError::IoError("Invalid data".to_string()),
            std::io::ErrorKind::TimedOut => NetError::IoError("Timed out".to_string()),
            std::io::ErrorKind::WriteZero => NetError::IoError("Write zero".to_string()),
            std::io::ErrorKind::Interrupted => NetError::IoError("Interrupted".to_string()),
            std::io::ErrorKind::Unsupported => NetError::IoError("Unsupported".to_string()),
            std::io::ErrorKind::UnexpectedEof => NetError::IoError("Unexpected EOF".to_string()),
            std::io::ErrorKind::OutOfMemory => NetError::IoError("Out of memory".to_string()),
            std::io::ErrorKind::PermissionDenied => {
                NetError::IoError("Permission denied".to_string())
            }
            std::io::ErrorKind::NotFound => NetError::IoError("Not found".to_string()),
            _ => NetError::IoError(format!("I/O error: {}", err)),
        }
    }
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::DnsResolutionFailed(msg) => write!(f, "DNS resolution failed: {}", msg),
            NetError::ConnectionRefused(msg) => write!(f, "Connection refused: {}", msg),
            NetError::ConnectionTimeout(msg) => write!(f, "Connection timeout: {}", msg),
            NetError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            NetError::InvalidPort(port) => write!(f, "Invalid port: {}", port),
            NetError::SocketError(msg) => write!(f, "Socket error: {}", msg),
            NetError::ResourceNotFound(rid) => write!(f, "Resource not found: {}", rid),
            NetError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            NetError::NetworkUnreachable(msg) => write!(f, "Network unreachable: {}", msg),
            NetError::HostUnreachable(msg) => write!(f, "Host unreachable: {}", msg),
            NetError::ConnectionReset(msg) => write!(f, "Connection reset: {}", msg),
            NetError::BrokenPipe(msg) => write!(f, "Broken pipe: {}", msg),
            NetError::AddressInUse(msg) => write!(f, "Address in use: {}", msg),
            NetError::AddressNotAvailable(msg) => write!(f, "Address not available: {}", msg),
            NetError::NotSupported(msg) => write!(f, "Not supported: {}", msg),
            NetError::Interrupted(msg) => write!(f, "Interrupted: {}", msg),
            NetError::WouldBlock(msg) => write!(f, "Would block: {}", msg),
            NetError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for NetError {}

impl From<std::io::Error> for NetError {
    fn from(err: std::io::Error) -> Self {
        NetError::from_io_error(err)
    }
}

impl From<std::net::AddrParseError> for NetError {
    fn from(err: std::net::AddrParseError) -> Self {
        NetError::InvalidAddress(err.to_string())
    }
}
