// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use trust_dns_resolver::{
    TokioAsyncResolver,
    config::{ResolverConfig, ResolverOpts},
};

use super::error::NetError;

/// DNS record types supported by Andromeda
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DnsRecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    PTR,
    SOA,
    SRV,
}

impl FromStr for DnsRecordType {
    type Err = NetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(DnsRecordType::A),
            "AAAA" => Ok(DnsRecordType::AAAA),
            "CNAME" => Ok(DnsRecordType::CNAME),
            "MX" => Ok(DnsRecordType::MX),
            "TXT" => Ok(DnsRecordType::TXT),
            "NS" => Ok(DnsRecordType::NS),
            "PTR" => Ok(DnsRecordType::PTR),
            "SOA" => Ok(DnsRecordType::SOA),
            "SRV" => Ok(DnsRecordType::SRV),
            _ => Err(NetError::not_supported(&format!("DNS record type: {}", s))),
        }
    }
}

impl std::fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsRecordType::A => write!(f, "A"),
            DnsRecordType::AAAA => write!(f, "AAAA"),
            DnsRecordType::CNAME => write!(f, "CNAME"),
            DnsRecordType::MX => write!(f, "MX"),
            DnsRecordType::TXT => write!(f, "TXT"),
            DnsRecordType::NS => write!(f, "NS"),
            DnsRecordType::PTR => write!(f, "PTR"),
            DnsRecordType::SOA => write!(f, "SOA"),
            DnsRecordType::SRV => write!(f, "SRV"),
        }
    }
}

/// DNS resolution result
#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: DnsRecordType,
    pub value: String,
    pub ttl: Option<u32>,
}

impl DnsRecord {
    pub fn new(name: String, record_type: DnsRecordType, value: String, ttl: Option<u32>) -> Self {
        Self {
            name,
            record_type,
            value,
            ttl,
        }
    }

    pub fn a(name: String, ip: Ipv4Addr, ttl: Option<u32>) -> Self {
        Self::new(name, DnsRecordType::A, ip.to_string(), ttl)
    }

    pub fn aaaa(name: String, ip: Ipv6Addr, ttl: Option<u32>) -> Self {
        Self::new(name, DnsRecordType::AAAA, ip.to_string(), ttl)
    }

    pub fn cname(name: String, target: String, ttl: Option<u32>) -> Self {
        Self::new(name, DnsRecordType::CNAME, target, ttl)
    }

    pub fn txt(name: String, text: String, ttl: Option<u32>) -> Self {
        Self::new(name, DnsRecordType::TXT, text, ttl)
    }
}

/// DNS resolution result container
#[derive(Debug, Clone)]
pub struct DnsResponse {
    pub hostname: String,
    pub records: Vec<DnsRecord>,
}

impl DnsResponse {
    pub fn new(hostname: String) -> Self {
        Self {
            hostname,
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DnsRecord) {
        self.records.push(record);
    }

    pub fn get_a_records(&self) -> Vec<Ipv4Addr> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::A)
            .filter_map(|r| Ipv4Addr::from_str(&r.value).ok())
            .collect()
    }

    pub fn get_aaaa_records(&self) -> Vec<Ipv6Addr> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::AAAA)
            .filter_map(|r| Ipv6Addr::from_str(&r.value).ok())
            .collect()
    }

    pub fn get_all_ips(&self) -> Vec<IpAddr> {
        let mut ips = Vec::new();

        for ipv4 in self.get_a_records() {
            ips.push(IpAddr::V4(ipv4));
        }

        for ipv6 in self.get_aaaa_records() {
            ips.push(IpAddr::V6(ipv6));
        }

        ips
    }

    pub fn get_cname_records(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::CNAME)
            .map(|r| r.value.clone())
            .collect()
    }

    pub fn get_txt_records(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::TXT)
            .map(|r| r.value.clone())
            .collect()
    }

    pub fn get_mx_records(&self) -> Vec<(u16, String)> {
        let mut mx_records = Vec::new();
        for record in &self.records {
            if record.record_type == DnsRecordType::MX {
                // Parse "preference exchange" format
                let parts: Vec<&str> = record.value.splitn(2, ' ').collect();
                if parts.len() == 2 && parts[0].parse::<u16>().is_ok() {
                    let preference = parts[0].parse::<u16>().unwrap();
                    mx_records.push((preference, parts[1].to_string()));
                }
            }
        }
        mx_records.sort_by_key(|&(pref, _)| pref);
        mx_records
    }

    pub fn get_ns_records(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::NS)
            .map(|r| r.value.clone())
            .collect()
    }

    pub fn get_ptr_records(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::PTR)
            .map(|r| r.value.clone())
            .collect()
    }

    pub fn get_soa_records(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::SOA)
            .map(|r| r.value.clone())
            .collect()
    }

    pub fn get_srv_records(&self) -> Vec<String> {
        self.records
            .iter()
            .filter(|r| r.record_type == DnsRecordType::SRV)
            .map(|r| r.value.clone())
            .collect()
    }

    /// Get records by type
    pub fn get_records_by_type(&self, record_type: DnsRecordType) -> Vec<&DnsRecord> {
        self.records
            .iter()
            .filter(|r| r.record_type == record_type)
            .collect()
    }

    /// Check if the response contains any records of a specific type
    pub fn has_record_type(&self, record_type: DnsRecordType) -> bool {
        self.records.iter().any(|r| r.record_type == record_type)
    }

    /// Get the total number of records
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Get a summary of record types and their counts
    pub fn get_record_summary(&self) -> std::collections::HashMap<DnsRecordType, usize> {
        let mut summary = std::collections::HashMap::new();
        for record in &self.records {
            *summary.entry(record.record_type.clone()).or_insert(0) += 1;
        }
        summary
    }
}

/// DNS operations with advanced resolver
pub struct DnsOps {
    resolver: Arc<Mutex<Option<TokioAsyncResolver>>>,
}

impl Default for DnsOps {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsOps {
    /// Create a new DNS operations instance
    pub fn new() -> Self {
        Self {
            resolver: Arc::new(Mutex::new(None)),
        }
    }

    /// Get or create the resolver instance
    async fn get_resolver(&self) -> Result<TokioAsyncResolver, NetError> {
        let mut resolver_guard = self.resolver.lock().await;

        if resolver_guard.is_none() {
            // Create resolver with system configuration
            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

            *resolver_guard = Some(resolver);
        }

        Ok(resolver_guard.as_ref().unwrap().clone())
    }

    /// Resolve hostname to IP addresses using proper DNS resolution
    pub async fn resolve_host(hostname: &str) -> Result<DnsResponse, NetError> {
        let ops = Self::new();
        let resolver = ops.get_resolver().await?;

        match resolver.lookup_ip(hostname).await {
            Ok(lookup) => {
                let mut response = DnsResponse::new(hostname.to_string());

                for ip in lookup.iter() {
                    match ip {
                        IpAddr::V4(ipv4) => {
                            response.add_record(DnsRecord::a(hostname.to_string(), ipv4, None));
                        }
                        IpAddr::V6(ipv6) => {
                            response.add_record(DnsRecord::aaaa(hostname.to_string(), ipv6, None));
                        }
                    }
                }

                if response.records.is_empty() {
                    Err(NetError::dns_failed(hostname, "No IP records found"))
                } else {
                    Ok(response)
                }
            }
            Err(e) => Err(NetError::dns_failed(hostname, &e.to_string())),
        }
    }

    /// Resolve specific record type with proper DNS queries
    pub async fn resolve_record(
        hostname: &str,
        record_type: DnsRecordType,
    ) -> Result<DnsResponse, NetError> {
        let ops = Self::new();
        let resolver = ops.get_resolver().await?;
        let mut response = DnsResponse::new(hostname.to_string());

        match record_type {
            DnsRecordType::A => match resolver.ipv4_lookup(hostname).await {
                Ok(lookup) => {
                    for ip in lookup.iter() {
                        response.add_record(DnsRecord::a(hostname.to_string(), ip.0, None));
                    }
                }
                Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
            },
            DnsRecordType::AAAA => match resolver.ipv6_lookup(hostname).await {
                Ok(lookup) => {
                    for ip in lookup.iter() {
                        response.add_record(DnsRecord::aaaa(hostname.to_string(), ip.0, None));
                    }
                }
                Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
            },
            DnsRecordType::CNAME => {
                // TODO: Find a better way to get CNAME records
                match resolver.txt_lookup(hostname).await {
                    Ok(lookup) => {
                        for txt in lookup.iter() {
                            let txt_data = txt.to_string();
                            if txt_data.contains("CNAME=") {
                                let cname = txt_data.replace("CNAME=", "");
                                response.add_record(DnsRecord::cname(
                                    hostname.to_string(),
                                    cname,
                                    None,
                                ));
                            }
                        }
                    }
                    Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
                }
            }
            DnsRecordType::TXT => match resolver.txt_lookup(hostname).await {
                Ok(lookup) => {
                    for txt in lookup.iter() {
                        response.add_record(DnsRecord::txt(
                            hostname.to_string(),
                            txt.to_string(),
                            None,
                        ));
                    }
                }
                Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
            },
            DnsRecordType::MX => match resolver.mx_lookup(hostname).await {
                Ok(lookup) => {
                    for mx in lookup.iter() {
                        let mx_record = format!("{} {}", mx.preference(), mx.exchange());
                        response.add_record(DnsRecord::new(
                            hostname.to_string(),
                            DnsRecordType::MX,
                            mx_record,
                            None,
                        ));
                    }
                }
                Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
            },
            DnsRecordType::NS => match resolver.ns_lookup(hostname).await {
                Ok(lookup) => {
                    for ns in lookup.iter() {
                        response.add_record(DnsRecord::new(
                            hostname.to_string(),
                            DnsRecordType::NS,
                            ns.to_string(),
                            None,
                        ));
                    }
                }
                Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
            },
            DnsRecordType::PTR => {
                // PTR records require reverse DNS lookup
                if let Ok(ip) = hostname.parse::<IpAddr>() {
                    match resolver.reverse_lookup(ip).await {
                        Ok(lookup) => {
                            for ptr in lookup.iter() {
                                response.add_record(DnsRecord::new(
                                    hostname.to_string(),
                                    DnsRecordType::PTR,
                                    ptr.to_string(),
                                    None,
                                ));
                            }
                        }
                        Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
                    }
                } else {
                    return Err(NetError::invalid_address(hostname));
                }
            }
            DnsRecordType::SOA => match resolver.soa_lookup(hostname).await {
                Ok(lookup) => {
                    for soa in lookup.iter() {
                        let soa_record = format!(
                            "{} {} {} {} {} {} {}",
                            soa.mname(),
                            soa.rname(),
                            soa.serial(),
                            soa.refresh(),
                            soa.retry(),
                            soa.expire(),
                            soa.minimum()
                        );
                        response.add_record(DnsRecord::new(
                            hostname.to_string(),
                            DnsRecordType::SOA,
                            soa_record,
                            None,
                        ));
                    }
                }
                Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
            },
            DnsRecordType::SRV => match resolver.srv_lookup(hostname).await {
                Ok(lookup) => {
                    for srv in lookup.iter() {
                        let srv_record = format!(
                            "{} {} {} {}",
                            srv.priority(),
                            srv.weight(),
                            srv.port(),
                            srv.target()
                        );
                        response.add_record(DnsRecord::new(
                            hostname.to_string(),
                            DnsRecordType::SRV,
                            srv_record,
                            None,
                        ));
                    }
                }
                Err(e) => return Err(NetError::dns_failed(hostname, &e.to_string())),
            },
        }

        if response.records.is_empty() {
            Err(NetError::dns_failed(
                hostname,
                &format!("No {} records found", record_type),
            ))
        } else {
            Ok(response)
        }
    }

    /// Get all IP addresses for a hostname
    pub async fn get_ips(hostname: &str) -> Result<Vec<IpAddr>, NetError> {
        let response = Self::resolve_host(hostname).await?;
        Ok(response.get_all_ips())
    }

    /// Get only IPv4 addresses for a hostname
    pub async fn get_ipv4_addresses(hostname: &str) -> Result<Vec<Ipv4Addr>, NetError> {
        let response = Self::resolve_record(hostname, DnsRecordType::A).await?;
        Ok(response.get_a_records())
    }

    /// Get only IPv6 addresses for a hostname
    pub async fn get_ipv6_addresses(hostname: &str) -> Result<Vec<Ipv6Addr>, NetError> {
        let response = Self::resolve_record(hostname, DnsRecordType::AAAA).await?;
        Ok(response.get_aaaa_records())
    }

    /// Reverse DNS lookup (IP to hostname) with proper implementation
    pub async fn reverse_lookup(ip: IpAddr) -> Result<String, NetError> {
        let ops = Self::new();
        let resolver = ops.get_resolver().await?;

        match resolver.reverse_lookup(ip).await {
            Ok(lookup) => {
                if let Some(name) = lookup.iter().next() {
                    Ok(name.to_string())
                } else {
                    Err(NetError::dns_failed(&ip.to_string(), "No PTR record found"))
                }
            }
            Err(e) => Err(NetError::dns_failed(&ip.to_string(), &e.to_string())),
        }
    }

    /// Get MX records for a domain
    pub async fn get_mx_records(domain: &str) -> Result<Vec<(u16, String)>, NetError> {
        let response = Self::resolve_record(domain, DnsRecordType::MX).await?;
        let mut mx_records = Vec::new();

        for record in response.records {
            if record.record_type == DnsRecordType::MX {
                // Parse "preference exchange" format
                let parts: Vec<&str> = record.value.splitn(2, ' ').collect();
                if parts.len() == 2 && parts[0].parse::<u16>().is_ok() {
                    let preference = parts[0].parse::<u16>().unwrap();
                    mx_records.push((preference, parts[1].to_string()));
                }
            }
        }

        // Sort by preference (lower values have higher priority)
        mx_records.sort_by_key(|&(pref, _)| pref);
        Ok(mx_records)
    }

    /// Get TXT records for a domain
    pub async fn get_txt_records(domain: &str) -> Result<Vec<String>, NetError> {
        let response = Self::resolve_record(domain, DnsRecordType::TXT).await?;
        Ok(response.get_txt_records())
    }

    /// Get NS records for a domain
    pub async fn get_ns_records(domain: &str) -> Result<Vec<String>, NetError> {
        let response = Self::resolve_record(domain, DnsRecordType::NS).await?;
        Ok(response
            .records
            .into_iter()
            .filter(|r| r.record_type == DnsRecordType::NS)
            .map(|r| r.value)
            .collect())
    }

    /// Perform a comprehensive DNS lookup (all record types)
    pub async fn comprehensive_lookup(hostname: &str) -> Result<DnsResponse, NetError> {
        let mut response = DnsResponse::new(hostname.to_string());

        let record_types = [
            DnsRecordType::A,
            DnsRecordType::AAAA,
            DnsRecordType::TXT,
            DnsRecordType::MX,
            DnsRecordType::NS,
            DnsRecordType::SOA,
        ];

        for record_type in &record_types {
            if let Ok(type_response) = Self::resolve_record(hostname, record_type.clone()).await {
                for record in type_response.records {
                    response.add_record(record);
                }
            }
        }

        if response.records.is_empty() {
            Err(NetError::dns_failed(hostname, "No DNS records found"))
        } else {
            Ok(response)
        }
    }
}
