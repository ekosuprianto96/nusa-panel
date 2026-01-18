//! # Domain Model
//!
//! Model dan DTO untuk Domain, Subdomain, dan DNS Records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Tipe DNS Record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsRecordType {
    /// A record - IPv4 address
    A,
    /// AAAA record - IPv6 address
    AAAA,
    /// CNAME record - Canonical name
    CNAME,
    /// MX record - Mail exchange
    MX,
    /// TXT record - Text record
    TXT,
    /// NS record - Name server
    NS,
    /// SOA record - Start of authority
    SOA,
    /// SRV record - Service
    SRV,
    /// CAA record - Certificate Authority Authorization
    CAA,
}

impl std::fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::AAAA => write!(f, "AAAA"),
            Self::CNAME => write!(f, "CNAME"),
            Self::MX => write!(f, "MX"),
            Self::TXT => write!(f, "TXT"),
            Self::NS => write!(f, "NS"),
            Self::SOA => write!(f, "SOA"),
            Self::SRV => write!(f, "SRV"),
            Self::CAA => write!(f, "CAA"),
        }
    }
}

impl From<String> for DnsRecordType {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "A" => Self::A,
            "AAAA" => Self::AAAA,
            "CNAME" => Self::CNAME,
            "MX" => Self::MX,
            "TXT" => Self::TXT,
            "NS" => Self::NS,
            "SOA" => Self::SOA,
            "SRV" => Self::SRV,
            "CAA" => Self::CAA,
            _ => Self::A, // Default
        }
    }
}

/// Domain entity dari database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Domain {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik domain
    pub user_id: String,

    /// Nama domain (e.g., example.com)
    pub domain_name: String,

    /// Path document root
    pub document_root: String,

    /// Status aktif domain
    pub is_active: bool,

    /// SSL enabled
    pub ssl_enabled: bool,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// Response DTO untuk Domain
#[derive(Debug, Serialize)]
pub struct DomainResponse {
    pub id: String,
    pub user_id: String,
    pub domain_name: String,
    pub document_root: String,
    pub is_active: bool,
    pub ssl_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Jumlah subdomain
    pub subdomains_count: i32,
    /// Jumlah DNS records
    pub dns_records_count: i32,
}

impl From<Domain> for DomainResponse {
    fn from(domain: Domain) -> Self {
        Self {
            id: domain.id,
            user_id: domain.user_id,
            domain_name: domain.domain_name,
            document_root: domain.document_root,
            is_active: domain.is_active,
            ssl_enabled: domain.ssl_enabled,
            created_at: domain.created_at,
            updated_at: domain.updated_at,
            subdomains_count: 0,
            dns_records_count: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedDomains {
    pub items: Vec<DomainResponse>,
    pub total: i64,
}

/// DTO untuk membuat domain baru
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDomainRequest {
    /// Nama domain (e.g., example.com)
    #[validate(length(min = 4, max = 255, message = "Nama domain harus 4-255 karakter"))]
    #[validate(regex(
        path = "crate::models::domain::DOMAIN_REGEX",
        message = "Format nama domain tidak valid"
    ))]
    pub domain_name: String,

    /// Path document root (opsional, akan di-generate jika kosong)
    #[validate(length(max = 500, message = "Document root maksimal 500 karakter"))]
    pub document_root: Option<String>,
}

/// DTO untuk update domain
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDomainRequest {
    /// Path document root baru
    #[validate(length(max = 500, message = "Document root maksimal 500 karakter"))]
    pub document_root: Option<String>,

    /// Status aktif
    pub is_active: Option<bool>,
}

/// Subdomain entity dari database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Subdomain {
    /// Unique identifier
    pub id: String,

    /// Domain ID parent
    pub domain_id: String,

    /// Nama subdomain (e.g., "blog" untuk blog.example.com)
    pub subdomain_name: String,

    /// Path document root
    pub document_root: String,

    /// Status aktif
    pub is_active: bool,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,
}

/// Response DTO untuk Subdomain
#[derive(Debug, Serialize)]
pub struct SubdomainResponse {
    pub id: String,
    pub domain_id: String,
    pub subdomain_name: String,
    pub full_name: String,
    pub document_root: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// DTO untuk membuat subdomain
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubdomainRequest {
    /// Nama subdomain (e.g., "blog", "api", "www")
    #[validate(length(min = 1, max = 63, message = "Nama subdomain harus 1-63 karakter"))]
    #[validate(regex(
        path = "crate::models::domain::SUBDOMAIN_REGEX",
        message = "Format subdomain tidak valid (hanya huruf, angka, dan dash)"
    ))]
    pub subdomain_name: String,

    /// Path document root (opsional)
    #[validate(length(max = 500, message = "Document root maksimal 500 karakter"))]
    pub document_root: Option<String>,
}

/// DNS Record entity dari database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DnsRecord {
    /// Unique identifier
    pub id: String,

    /// Domain ID
    pub domain_id: String,

    /// Tipe record (A, AAAA, CNAME, MX, TXT, dll)
    pub record_type: String,

    /// Nama record (e.g., "@", "www", "mail")
    pub name: String,

    /// Nilai record (e.g., IP address, hostname)
    pub value: String,

    /// Time to live dalam detik
    pub ttl: i32,

    /// Priority (untuk MX records)
    pub priority: Option<i32>,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// Response DTO untuk DNS Record
#[derive(Debug, Serialize)]
pub struct DnsRecordResponse {
    pub id: String,
    pub domain_id: String,
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: i32,
    pub priority: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DnsRecord> for DnsRecordResponse {
    fn from(record: DnsRecord) -> Self {
        Self {
            id: record.id,
            domain_id: record.domain_id,
            record_type: record.record_type,
            name: record.name,
            value: record.value,
            ttl: record.ttl,
            priority: record.priority,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// DTO untuk membuat DNS record
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDnsRecordRequest {
    /// Tipe record
    pub record_type: String,

    /// Nama record (e.g., "@" untuk root, "www", "mail")
    #[validate(length(min = 1, max = 255, message = "Nama record harus 1-255 karakter"))]
    pub name: String,

    /// Nilai record
    #[validate(length(min = 1, max = 500, message = "Nilai record harus 1-500 karakter"))]
    pub value: String,

    /// TTL dalam detik (default: 3600)
    pub ttl: Option<i32>,

    /// Priority untuk MX records
    pub priority: Option<i32>,
}

/// DTO untuk update DNS record
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDnsRecordRequest {
    /// Nilai record baru
    #[validate(length(min = 1, max = 500, message = "Nilai record harus 1-500 karakter"))]
    pub value: Option<String>,

    /// TTL baru
    pub ttl: Option<i32>,

    /// Priority baru
    pub priority: Option<i32>,
}

/// Redirect entity dari database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Redirect {
    pub id: String,
    pub domain_id: String,
    pub source_path: String,
    pub destination_url: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub type_: String,
    pub created_at: DateTime<Utc>,
}

/// DTO untuk membuat redirect
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRedirectRequest {
    #[validate(length(min = 1, max = 500, message = "Source path maksimal 500 karakter"))]
    pub source_path: String,
    
    #[validate(url(message = "Destination URL tidak valid"))]
    pub destination_url: String,
    
    pub type_: String,
}

/// Domain Alias entity dari database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DomainAlias {
    pub id: String,
    pub domain_id: String,
    pub alias_domain: String,
    pub created_at: DateTime<Utc>,
}

/// DTO untuk membuat alias
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAliasRequest {
    #[validate(length(min = 4, max = 255, message = "Nama domain harus 4-255 karakter"))]
    #[validate(regex(
        path = "crate::models::domain::DOMAIN_REGEX",
        message = "Format nama domain tidak valid"
    ))]
    pub alias_domain: String,
}

/// Regex untuk validasi domain name
pub static DOMAIN_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$")
            .unwrap()
    });

/// Regex untuk validasi subdomain
pub static SUBDOMAIN_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$").unwrap()
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_regex() {
        assert!(DOMAIN_REGEX.is_match("example.com"));
        assert!(DOMAIN_REGEX.is_match("sub.example.com"));
        assert!(DOMAIN_REGEX.is_match("my-domain.co.id"));
        assert!(!DOMAIN_REGEX.is_match("invalid"));
        assert!(!DOMAIN_REGEX.is_match("-invalid.com"));
    }

    #[test]
    fn test_subdomain_regex() {
        assert!(SUBDOMAIN_REGEX.is_match("www"));
        assert!(SUBDOMAIN_REGEX.is_match("api"));
        assert!(SUBDOMAIN_REGEX.is_match("my-api"));
        assert!(!SUBDOMAIN_REGEX.is_match("-invalid"));
        assert!(!SUBDOMAIN_REGEX.is_match("invalid-"));
    }

    #[test]
    fn test_dns_record_type() {
        assert_eq!(DnsRecordType::from("A".to_string()), DnsRecordType::A);
        assert_eq!(DnsRecordType::from("mx".to_string()), DnsRecordType::MX);
        assert_eq!(DnsRecordType::A.to_string(), "A");
    }
}
