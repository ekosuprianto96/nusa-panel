//! # Security & Monitoring Model
//!
//! Model dan DTO untuk fitur keamanan (IP Blocking, SSH) dan monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use rocket::form::FromForm;

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
    pub public_key: String,
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

    /// Disk usage percent (0-100)
    pub disk_percent: f32,

    /// Firewall status (simple indicator)
    pub firewall_enabled: bool,

    /// Total blocked IP count
    pub blocked_ips_count: i64,

    /// Total SSH key count for user
    pub ssh_keys_count: i64,

    /// Waktu pengambilan data
    pub timestamp: DateTime<Utc>,
}

/// Access Log Entry (Simulasi)
#[derive(Debug, Serialize)]
pub struct AccessLogEntry {
    pub id: String,
    pub event_type: String,
    pub ip_address: String,
    pub target: Option<String>,
    pub status: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// 2FA status response
#[derive(Debug, Serialize)]
pub struct TwoFactorStatusResponse {
    pub enabled: bool,
    pub enabled_at: Option<DateTime<Utc>>,
}

/// 2FA setup response
#[derive(Debug, Serialize)]
pub struct TwoFactorSetupResponse {
    pub qr_code: String,
    pub secret: String,
    pub backup_codes: Vec<String>,
}

/// 2FA verify request
#[derive(Debug, Deserialize, Validate)]
pub struct TwoFactorVerifyRequest {
    #[validate(length(min = 6, max = 6, message = "Kode harus 6 digit"))]
    pub code: String,
}

/// ModSecurity settings
#[derive(Debug, Serialize)]
pub struct ModSecuritySettings {
    pub main_engine: bool,
    pub paranoia_level: i32,
    pub anomaly_threshold: i32,
}

/// ModSecurity rule set
#[derive(Debug, Serialize)]
pub struct ModSecurityRuleSet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

/// ModSecurity custom rule
#[derive(Debug, Serialize, FromRow)]
pub struct ModSecurityCustomRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_content: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create custom rule request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateModSecurityCustomRuleRequest {
    #[validate(length(min = 1, max = 120, message = "Nama rule wajib diisi"))]
    pub name: String,
    #[validate(length(max = 255, message = "Deskripsi maksimal 255 karakter"))]
    pub description: Option<String>,
    #[validate(length(min = 5, message = "Rule content wajib diisi"))]
    pub rule_content: String,
    pub enabled: Option<bool>,
}

/// Update custom rule request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateModSecurityCustomRuleRequest {
    #[validate(length(min = 1, max = 120, message = "Nama rule wajib diisi"))]
    pub name: Option<String>,
    #[validate(length(max = 255, message = "Deskripsi maksimal 255 karakter"))]
    pub description: Option<String>,
    #[validate(length(min = 5, message = "Rule content wajib diisi"))]
    pub rule_content: Option<String>,
    pub enabled: Option<bool>,
}

/// ModSecurity audit log entry
#[derive(Debug, Serialize, FromRow)]
pub struct ModSecurityAuditLog {
    pub id: String,
    pub user_id: Option<String>,
    pub domain_id: Option<String>,
    pub rule_id: Option<String>,
    pub custom_rule_id: Option<String>,
    pub severity: String,
    pub message: String,
    pub ip_address: Option<String>,
    pub uri: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, FromForm)]
pub struct ModSecurityAuditLogQuery {
    pub domain_id: Option<String>,
    pub limit: Option<i64>,
}

/// Ingest ModSecurity audit log request
#[derive(Debug, Deserialize, Validate)]
pub struct IngestModSecurityAuditLogRequest {
    /// Optional path to log file (fallback to config)
    #[validate(length(min = 1, max = 500, message = "Path log wajib 1-500 karakter"))]
    pub path: Option<String>,
    /// Maximum entries to ingest (default 1000, max 5000)
    #[validate(range(min = 1, max = 5000, message = "Max entries 1-5000"))]
    pub max_entries: Option<i64>,
}

/// Ingest ModSecurity audit log response
#[derive(Debug, Serialize)]
pub struct IngestModSecurityAuditLogResponse {
    pub source_path: String,
    pub parsed: i64,
    pub inserted: i64,
    pub skipped: i64,
}

/// ModSecurity overview response
#[derive(Debug, Serialize)]
pub struct ModSecurityOverview {
    pub rules_triggered: i64,
    pub trend_percent: i32,
    pub chart_labels: Vec<String>,
    pub chart_data: Vec<i32>,
    pub settings: ModSecuritySettings,
    pub rules: Vec<ModSecurityRuleSet>,
}

/// Update ModSecurity settings request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateModSecuritySettingsRequest {
    pub main_engine: Option<bool>,
    #[validate(range(min = 1, max = 4, message = "Paranoia level 1-4"))]
    pub paranoia_level: Option<i32>,
    #[validate(range(min = 1, max = 20, message = "Anomaly threshold 1-20"))]
    pub anomaly_threshold: Option<i32>,
}

/// Update ModSecurity rule request
#[derive(Debug, Deserialize)]
pub struct UpdateModSecurityRuleRequest {
    pub enabled: bool,
}

/// Update ModSecurity domain request
#[derive(Debug, Deserialize)]
pub struct UpdateModSecurityDomainRequest {
    pub enabled: bool,
}
