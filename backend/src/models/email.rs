//! # Email Model
//!
//! Model dan DTO untuk Email Management operations.
//! Includes email accounts, forwarders, dan autoresponders.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Email Account entity dari database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EmailAccount {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Domain ID
    pub domain_id: String,

    /// Alamat email (e.g., info@example.com)
    pub email_address: String,

    /// Password hash
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Quota mailbox dalam bytes (0 = unlimited)
    pub quota_bytes: i64,

    /// Jumlah bytes yang digunakan
    pub used_bytes: i64,

    /// Status aktif
    pub is_active: bool,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,

    /// Login terakhir
    pub last_login: Option<DateTime<Utc>>,
}

/// Response DTO untuk Email Account
#[derive(Debug, Serialize)]
pub struct EmailAccountResponse {
    pub id: String,
    pub user_id: String,
    pub domain_id: String,
    pub email_address: String,
    pub quota_bytes: i64,
    pub quota_mb: f64,
    pub used_bytes: i64,
    pub used_mb: f64,
    pub quota_percentage: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    /// Info untuk webmail
    pub webmail_info: WebmailInfo,
}

impl From<EmailAccount> for EmailAccountResponse {
    fn from(account: EmailAccount) -> Self {
        let quota_percentage = if account.quota_bytes > 0 {
            (account.used_bytes as f64 / account.quota_bytes as f64) * 100.0
        } else {
            0.0
        };

        Self {
            id: account.id.clone(),
            user_id: account.user_id,
            domain_id: account.domain_id,
            email_address: account.email_address.clone(),
            quota_bytes: account.quota_bytes,
            quota_mb: account.quota_bytes as f64 / (1024.0 * 1024.0),
            used_bytes: account.used_bytes,
            used_mb: account.used_bytes as f64 / (1024.0 * 1024.0),
            quota_percentage,
            is_active: account.is_active,
            created_at: account.created_at,
            updated_at: account.updated_at,
            last_login: account.last_login,
            webmail_info: WebmailInfo {
                url: "https://webmail.example.com".to_string(),
                username: account.email_address,
                imap_server: "mail.example.com".to_string(),
                imap_port: 993,
                smtp_server: "mail.example.com".to_string(),
                smtp_port: 587,
            },
        }
    }
}

/// Info untuk akses webmail
#[derive(Debug, Serialize)]
pub struct WebmailInfo {
    /// URL webmail (Roundcube, dll)
    pub url: String,
    /// Username (email address)
    pub username: String,
    /// IMAP server
    pub imap_server: String,
    /// IMAP port (SSL)
    pub imap_port: u16,
    /// SMTP server
    pub smtp_server: String,
    /// SMTP port (TLS)
    pub smtp_port: u16,
}

/// DTO untuk membuat email account
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEmailAccountRequest {
    /// Domain ID
    pub domain_id: String,

    /// Username (bagian sebelum @)
    #[validate(length(min = 1, max = 64, message = "Username harus 1-64 karakter"))]
    #[validate(regex(
        path = "crate::models::email::EMAIL_USERNAME_REGEX",
        message = "Username hanya boleh huruf, angka, titik, dash, dan underscore"
    ))]
    pub username: String,

    /// Password
    #[validate(length(min = 8, max = 128, message = "Password harus 8-128 karakter"))]
    pub password: String,

    /// Quota dalam MB (0 = unlimited)
    pub quota_mb: Option<i64>,
}

/// DTO untuk update email account
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEmailAccountRequest {
    /// Password baru
    #[validate(length(min = 8, max = 128, message = "Password harus 8-128 karakter"))]
    pub password: Option<String>,

    /// Quota baru dalam MB
    pub quota_mb: Option<i64>,

    /// Status aktif
    pub is_active: Option<bool>,
}

/// Email Forwarder entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EmailForwarder {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Domain ID
    pub domain_id: String,

    /// Alamat sumber (e.g., sales@example.com)
    pub source_email: String,

    /// Alamat tujuan (e.g., john@gmail.com)
    pub destination_email: String,

    /// Status aktif
    pub is_active: bool,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,
}

/// Response DTO untuk Email Forwarder
#[derive(Debug, Serialize)]
pub struct EmailForwarderResponse {
    pub id: String,
    pub user_id: String,
    pub domain_id: String,
    pub source_email: String,
    pub destination_email: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<EmailForwarder> for EmailForwarderResponse {
    fn from(forwarder: EmailForwarder) -> Self {
        Self {
            id: forwarder.id,
            user_id: forwarder.user_id,
            domain_id: forwarder.domain_id,
            source_email: forwarder.source_email,
            destination_email: forwarder.destination_email,
            is_active: forwarder.is_active,
            created_at: forwarder.created_at,
        }
    }
}

/// DTO untuk membuat email forwarder
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEmailForwarderRequest {
    /// Domain ID
    pub domain_id: String,

    /// Username sumber (bagian sebelum @)
    #[validate(length(min = 1, max = 64, message = "Username harus 1-64 karakter"))]
    pub source_username: String,

    /// Email tujuan (alamat lengkap)
    #[validate(email(message = "Format email tujuan tidak valid"))]
    pub destination_email: String,
}

/// Autoresponder entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Autoresponder {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Email Account ID
    pub email_account_id: String,

    /// Subject auto-reply
    pub subject: String,

    /// Body auto-reply message
    pub body: String,

    /// Tanggal mulai aktif
    pub start_date: Option<DateTime<Utc>>,

    /// Tanggal berakhir
    pub end_date: Option<DateTime<Utc>>,

    /// Status aktif
    pub is_active: bool,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// Response DTO untuk Autoresponder
#[derive(Debug, Serialize)]
pub struct AutoresponderResponse {
    pub id: String,
    pub user_id: String,
    pub email_account_id: String,
    pub email_address: String,
    pub subject: String,
    pub body: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO untuk membuat autoresponder
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAutoresponderRequest {
    /// Email Account ID
    pub email_account_id: String,

    /// Subject auto-reply
    #[validate(length(min = 1, max = 255, message = "Subject harus 1-255 karakter"))]
    pub subject: String,

    /// Body message
    #[validate(length(min = 1, max = 10000, message = "Body harus 1-10000 karakter"))]
    pub body: String,

    /// Tanggal mulai (opsional)
    pub start_date: Option<DateTime<Utc>>,

    /// Tanggal berakhir (opsional)
    pub end_date: Option<DateTime<Utc>>,
}

/// DTO untuk update autoresponder
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAutoresponderRequest {
    /// Subject baru
    #[validate(length(min = 1, max = 255, message = "Subject harus 1-255 karakter"))]
    pub subject: Option<String>,

    /// Body baru
    #[validate(length(min = 1, max = 10000, message = "Body harus 1-10000 karakter"))]
    pub body: Option<String>,

    /// Tanggal mulai baru
    pub start_date: Option<DateTime<Utc>>,

    /// Tanggal berakhir baru
    pub end_date: Option<DateTime<Utc>>,

    /// Status aktif
    pub is_active: Option<bool>,
}

/// Regex untuk validasi email username
pub static EMAIL_USERNAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9._-]*$").unwrap()
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_username_regex() {
        assert!(EMAIL_USERNAME_REGEX.is_match("info"));
        assert!(EMAIL_USERNAME_REGEX.is_match("john.doe"));
        assert!(EMAIL_USERNAME_REGEX.is_match("support-team"));
        assert!(EMAIL_USERNAME_REGEX.is_match("admin_1"));
        assert!(!EMAIL_USERNAME_REGEX.is_match(".info")); // Can't start with dot
        assert!(!EMAIL_USERNAME_REGEX.is_match("-info")); // Can't start with dash
    }

    #[test]
    fn test_quota_percentage() {
        let account = EmailAccount {
            id: "1".to_string(),
            user_id: "1".to_string(),
            domain_id: "1".to_string(),
            email_address: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            quota_bytes: 1024 * 1024 * 100, // 100MB
            used_bytes: 1024 * 1024 * 25,    // 25MB
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
        };

        let response = EmailAccountResponse::from(account);
        assert!((response.quota_percentage - 25.0).abs() < 0.01);
    }
}
