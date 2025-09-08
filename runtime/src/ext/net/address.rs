// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;

/// Network address representation for Andromeda
#[derive(Debug, Clone, PartialEq)]
pub struct NetAddr {
    pub hostname: String,
    pub port: u16,
}

impl NetAddr {
    pub fn new(hostname: String, port: u16) -> Self {
        Self { hostname, port }
    }

    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        Self {
            hostname: addr.ip().to_string(),
            port: addr.port(),
        }
    }

    pub fn to_socket_addr(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        let ip = IpAddr::from_str(&self.hostname)?;
        Ok(SocketAddr::new(ip, self.port))
    }

    pub fn is_ipv4(&self) -> bool {
        Ipv4Addr::from_str(&self.hostname).is_ok()
    }

    pub fn is_ipv6(&self) -> bool {
        Ipv6Addr::from_str(&self.hostname).is_ok()
    }
}

impl std::fmt::Display for NetAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_ipv6() {
            write!(f, "[{}]:{}", self.hostname, self.port)
        } else {
            write!(f, "{}:{}", self.hostname, self.port)
        }
    }
}

impl FromStr for NetAddr {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('[') {
            let end_bracket = s.find(']').ok_or("Invalid IPv6 address format")?;
            let hostname = s[1..end_bracket].to_string();
            let port_part = &s[end_bracket + 1..];

            if let Some(stripped) = port_part.strip_prefix(':') {
                let port = stripped.parse::<u16>()?;
                Ok(NetAddr::new(hostname, port))
            } else {
                Err("Missing port in IPv6 address".into())
            }
        } else {
            let parts: Vec<&str> = s.rsplitn(2, ':').collect();
            if parts.len() != 2 {
                return Err("Invalid address format".into());
            }

            let port = parts[0].parse::<u16>()?;
            let hostname = parts[1].to_string();

            Ok(NetAddr::new(hostname, port))
        }
    }
}
