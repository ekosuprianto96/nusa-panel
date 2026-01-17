//! # Web Server Model
//!
//! Model dan DTO untuk Web Server configuration (Nginx/Apache).
//! Termasuk Virtual Host, SSL Certificates, dan PHP configurations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Mode Web Server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebServerType {
    Nginx,
    Apache,
    OpenLiteSpeed,
}

impl Default for WebServerType {
    fn default() -> Self {
        Self::Nginx
    }
}

/// PHP Version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhpVersion {
    #[serde(rename = "7.4")]
    Php74,
    #[serde(rename = "8.0")]
    Php80,
    #[serde(rename = "8.1")]
    Php81,
    #[serde(rename = "8.2")]
    Php82,
    #[serde(rename = "8.3")]
    Php83,
}

impl Default for PhpVersion {
    fn default() -> Self {
        Self::Php82
    }
}

impl ToString for PhpVersion {
    fn to_string(&self) -> String {
        match self {
            Self::Php74 => "7.4".to_string(),
            Self::Php80 => "8.0".to_string(),
            Self::Php81 => "8.1".to_string(),
            Self::Php82 => "8.2".to_string(),
            Self::Php83 => "8.3".to_string(),
        }
    }
}

/// Virtual Host entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct VirtualHost {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Domain ID
    pub domain_id: String,

    /// Document root directory
    pub document_root: String,

    /// Admin email
    pub admin_email: String,

    /// PHP version (e.g., "8.2")
    pub php_version: String,

    /// Status SSL
    pub ssl_enabled: bool,

    /// Force HTTPS redirect
    pub force_https: bool,

    /// Status aktif
    pub is_active: bool,

    /// Custom Nginx/Apache config snippet
    pub custom_config: Option<String>,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// Response DTO untuk Virtual Host
#[derive(Debug, Serialize)]
pub struct VirtualHostResponse {
    pub id: String,
    pub user_id: String,
    pub domain_id: String,
    pub domain_name: String, // from join
    pub document_root: String,
    pub php_version: String,
    pub ssl_enabled: bool,
    pub force_https: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    /// Default Package Specs (Temporary placeholder)
    pub package_specs: PackageSpecs,
}

/// Default Package Specs (Placeholder sampai Package Management diimplementasi)
#[derive(Debug, Serialize)]
pub struct PackageSpecs {
    pub max_ram: String,
    pub max_cpu: String,
    pub bandwidth_limit: String,
    pub process_limit: i32,
}

impl Default for PackageSpecs {
    fn default() -> Self {
        Self {
            max_ram: "512MB".to_string(),
            max_cpu: "1 Core".to_string(),
            bandwidth_limit: "Unlimited".to_string(),
            process_limit: 20,
        }
    }
}

/// DTO untuk membuat Virtual Host
#[derive(Debug, Deserialize, Validate)]
pub struct CreateVirtualHostRequest {
    /// Domain ID
    pub domain_id: String,

    /// PHP version (default: 8.2)
    pub php_version: Option<PhpVersion>,

    /// Web Server Type (default: Nginx)
    #[serde(default)]
    pub web_server_type: WebServerType,

    /// Enable SSL (default: false)
    pub ssl_enabled: Option<bool>,
}

/// DTO untuk update Virtual Host
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateVirtualHostRequest {
    /// PHP version
    pub php_version: Option<PhpVersion>,

    /// Enable SSL
    pub ssl_enabled: Option<bool>,

    /// Force HTTPS
    pub force_https: Option<bool>,

    /// Custom config snippet
    #[validate(length(max = 10000, message = "Config terlalu panjang (max 10000 chars)"))]
    pub custom_config: Option<String>,

    /// Status aktif
    pub is_active: Option<bool>,
}

/// SSL Certificate entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SslCertificate {
    /// Unique identifier
    pub id: String,

    /// Virtual Host ID
    pub vhost_id: String,

    /// User ID
    pub user_id: String,

    /// Provider (Let's Encrypt, Custom, Self-Signed)
    pub provider: String,

    /// Certificate content (PEM)
    #[serde(skip_serializing)]
    pub cert_pem: String,

    /// Private key content (PEM)
    #[serde(skip_serializing)]
    pub key_pem: String,

    /// Chain/CA bundle content (PEM)
    #[serde(skip_serializing)]
    pub chain_pem: Option<String>,

    /// Tanggal kadaluarsa
    pub expires_at: DateTime<Utc>,

    /// Auto renew status (Let's Encrypt only)
    pub auto_renew: bool,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,
}

/// Response DTO untuk SSL Certificate
#[derive(Debug, Serialize)]
pub struct SslCertificateResponse {
    pub id: String,
    pub vhost_id: String,
    pub provider: String,
    pub expires_at: DateTime<Utc>,
    pub auto_renew: bool,
    pub days_remaining: i64,
}

impl From<SslCertificate> for SslCertificateResponse {
    fn from(cert: SslCertificate) -> Self {
        let now = Utc::now();
        let duration = cert.expires_at.signed_duration_since(now);

        Self {
            id: cert.id,
            vhost_id: cert.vhost_id,
            provider: cert.provider,
            expires_at: cert.expires_at,
            auto_renew: cert.auto_renew,
            days_remaining: duration.num_days(),
        }
    }
}

/// DTO untuk request SSL (Let's Encrypt)
#[derive(Debug, Deserialize, Validate)]
pub struct RequestSslRequest {
    /// Virtual Host ID
    pub vhost_id: String,

    /// Email untuk notifikasi renewal (default: admin email)
    #[validate(email(message = "Format email tidak valid"))]
    pub email: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_version_string() {
        assert_eq!(PhpVersion::Php82.to_string(), "8.2");
        assert_eq!(PhpVersion::Php74.to_string(), "7.4");
    }

    #[test]
    fn test_ssl_days_remaining() {
        let now = Utc::now();
        let expires = now + chrono::Duration::days(30);

        let cert = SslCertificate {
            id: "1".to_string(),
            vhost_id: "1".to_string(),
            user_id: "1".to_string(),
            provider: "LetsEncrypt".to_string(),
            cert_pem: "".to_string(),
            key_pem: "".to_string(),
            chain_pem: None,
            expires_at: expires,
            auto_renew: true,
            created_at: now,
        };

        let response = SslCertificateResponse::from(cert);
        assert_eq!(response.days_remaining, 30);
    }
}
