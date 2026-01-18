//! # Security & Monitoring Model
//!
//! Model dan DTO untuk fitur keamanan (IP Blocking, SSH) dan monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Blocked IP entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BlockedIp {
    /// Unique identifier
    pub id: String,

    /// IP Address (v4 or v6)
    pub ip_address: String,

    /// Alasan blokir
    pub reason: Option<String>,

    /// Waktu blokir dibuat
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// DTO untuk blokir IP baru
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBlockedIpRequest {
    /// IP Address
    #[validate(length(min = 1, message = "IP Address tidak boleh kosong"))]
    pub ip_address: String,

    /// Alasan (opsional)
    #[validate(length(max = 255, message = "Reason max 255 karakter"))]
    pub reason: Option<String>,
}

/// SSH Access entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SshAccess {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// SSH Public Key
    pub public_key: String,

    /// Label/Nama kunci
    pub label: String,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,
}

/// Response DTO untuk SSH Access
#[derive(Debug, Serialize)]
pub struct SshAccessResponse {
    pub id: String,
    pub label: String,
    pub fingerprint: String, // SHA256/MD5 fingerprint simulation
    pub created_at: DateTime<Utc>,
}

/// DTO untuk tambah SSH Key
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSshKeyRequest {
    /// Label kunci
    #[validate(length(min = 1, max = 50, message = "Label harus 1-50 karakter"))]
    pub label: String,

    /// Public Key content (starts with ssh-rsa, etc)
    #[validate(length(min = 1, message = "Public Key tidak boleh kosong"))]
    pub public_key: String,
}

/// Resource Usage Stats (Simulasi)
#[derive(Debug, Serialize)]
pub struct ResourceUsageStats {
    /// User ID
    pub user_id: String,

    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f32,

    /// RAM usage in bytes
    pub ram_usage_bytes: i64,
    /// RAM limit in bytes
    pub ram_limit_bytes: i64,

    /// Disk usage in bytes
    pub disk_usage_bytes: i64,
    /// Disk limit in bytes
    pub disk_limit_bytes: i64,

    /// Waktu pengambilan data
    pub timestamp: DateTime<Utc>,
}

/// Access Log Entry (Simulasi)
#[derive(Debug, Serialize)]
pub struct AccessLogEntry {
    pub ip_address: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
}
