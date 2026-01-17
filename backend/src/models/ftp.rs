//! # FTP Model
//!
//! Model dan DTO untuk FTP Account management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// FTP Account entity dari database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct FtpAccount {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Username FTP (biasanya format: user@domain)
    pub ftp_username: String,

    /// Password hash (tidak di-serialize ke response)
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Home directory untuk FTP user
    pub home_directory: String,

    /// Status aktif
    pub is_active: bool,

    /// Quota dalam bytes (0 = unlimited)
    pub quota_bytes: i64,

    /// Jumlah bytes yang sudah digunakan
    pub used_bytes: i64,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,

    /// Login terakhir
    pub last_login: Option<DateTime<Utc>>,
}

/// Response DTO untuk FTP Account
#[derive(Debug, Serialize)]
pub struct FtpAccountResponse {
    pub id: String,
    pub user_id: String,
    pub ftp_username: String,
    pub home_directory: String,
    pub is_active: bool,
    pub quota_bytes: i64,
    pub used_bytes: i64,
    pub quota_percentage: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<FtpAccount> for FtpAccountResponse {
    fn from(account: FtpAccount) -> Self {
        let quota_percentage = if account.quota_bytes > 0 {
            (account.used_bytes as f64 / account.quota_bytes as f64) * 100.0
        } else {
            0.0
        };

        Self {
            id: account.id,
            user_id: account.user_id,
            ftp_username: account.ftp_username,
            home_directory: account.home_directory,
            is_active: account.is_active,
            quota_bytes: account.quota_bytes,
            used_bytes: account.used_bytes,
            quota_percentage,
            created_at: account.created_at,
            updated_at: account.updated_at,
            last_login: account.last_login,
        }
    }
}

/// DTO untuk membuat FTP account baru
#[derive(Debug, Deserialize, Validate)]
pub struct CreateFtpAccountRequest {
    /// Username FTP (tanpa @domain, akan di-generate otomatis)
    #[validate(length(min = 3, max = 32, message = "Username harus 3-32 karakter"))]
    #[validate(regex(
        path = "crate::models::ftp::FTP_USERNAME_REGEX",
        message = "Username hanya boleh huruf, angka, underscore, dan dash"
    ))]
    pub username: String,

    /// Password untuk FTP account
    #[validate(length(min = 8, max = 128, message = "Password harus 8-128 karakter"))]
    pub password: String,

    /// Home directory (relatif dari user home, opsional)
    #[validate(length(max = 500, message = "Home directory maksimal 500 karakter"))]
    pub home_directory: Option<String>,

    /// Quota dalam MB (0 = unlimited)
    pub quota_mb: Option<i64>,
}

/// DTO untuk update FTP account
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateFtpAccountRequest {
    /// Password baru (opsional)
    #[validate(length(min = 8, max = 128, message = "Password harus 8-128 karakter"))]
    pub password: Option<String>,

    /// Home directory baru (opsional)
    #[validate(length(max = 500, message = "Home directory maksimal 500 karakter"))]
    pub home_directory: Option<String>,

    /// Status aktif
    pub is_active: Option<bool>,

    /// Quota baru dalam MB
    pub quota_mb: Option<i64>,
}

/// DTO untuk update password FTP
#[derive(Debug, Deserialize, Validate)]
pub struct ChangeFtpPasswordRequest {
    /// Password baru
    #[validate(length(min = 8, max = 128, message = "Password harus 8-128 karakter"))]
    pub new_password: String,
}

/// Regex untuk validasi FTP username
pub static FTP_USERNAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap()
    });

/// FTP Server configuration
#[derive(Debug, Clone, Serialize)]
pub struct FtpServerInfo {
    /// FTP server hostname
    pub hostname: String,
    /// FTP port (usually 21)
    pub port: u16,
    /// FTPS (FTP over TLS) port
    pub ftps_port: u16,
    /// SFTP port (usually 22)
    pub sftp_port: u16,
    /// Passive mode port range
    pub passive_ports: String,
}

impl Default for FtpServerInfo {
    fn default() -> Self {
        Self {
            hostname: "ftp.example.com".to_string(),
            port: 21,
            ftps_port: 990,
            sftp_port: 22,
            passive_ports: "49152-65534".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ftp_username_regex() {
        assert!(FTP_USERNAME_REGEX.is_match("user123"));
        assert!(FTP_USERNAME_REGEX.is_match("my_user"));
        assert!(FTP_USERNAME_REGEX.is_match("my-user"));
        assert!(!FTP_USERNAME_REGEX.is_match("123user")); // Can't start with number
        assert!(!FTP_USERNAME_REGEX.is_match("user@domain")); // No @ allowed
        assert!(!FTP_USERNAME_REGEX.is_match("user.name")); // No dot allowed
    }

    #[test]
    fn test_quota_percentage() {
        let account = FtpAccount {
            id: "1".to_string(),
            user_id: "1".to_string(),
            ftp_username: "test".to_string(),
            password_hash: "hash".to_string(),
            home_directory: "/home/user".to_string(),
            is_active: true,
            quota_bytes: 1024 * 1024 * 100, // 100MB
            used_bytes: 1024 * 1024 * 50,    // 50MB
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login: None,
        };

        let response = FtpAccountResponse::from(account);
        assert!((response.quota_percentage - 50.0).abs() < 0.01);
    }
}
