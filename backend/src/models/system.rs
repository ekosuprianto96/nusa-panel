//! # System Tools Model
//!
//! Model dan DTO untuk System Tools (Cron, Backup, Service Status).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Cron Job entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CronJob {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Command cron schedule (e.g., "0 * * * *")
    pub schedule: String,

    /// Command to execute
    pub command: String,

    /// Deskripsi/Label
    pub description: Option<String>,

    /// Status aktif
    pub is_active: bool,

    /// Email notification (opsional)
    pub email_notification: Option<String>,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// DTO untuk membuat cron job
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCronJobRequest {
    /// Schedule (crontab format)
    #[validate(length(min = 5, message = "Schedule cron tidak valid"))]
    pub schedule: String,

    /// Command to execute
    #[validate(length(min = 1, message = "Command tidak boleh kosong"))]
    pub command: String,

    /// Description
    #[validate(length(max = 255))]
    pub description: Option<String>,

    /// Email notification
    #[validate(email(message = "Format email tidak valid"))]
    pub email_notification: Option<String>,
}

/// DTO untuk update cron job
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCronJobRequest {
    pub schedule: Option<String>,
    pub command: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub email_notification: Option<String>,
}

/// System Backup entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SystemBackup {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Nama file backup
    pub filename: String,

    /// Ukuran file dalam bytes
    pub size_bytes: i64,

    /// Tipe backup (Full, Database, Homedir)
    pub backup_type: String,

    /// Status (Pending, Completed, Failed)
    pub status: String,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu selesai
    pub completed_at: Option<DateTime<Utc>>,
}

/// DTO untuk membuat backup
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBackupRequest {
    /// Tipe backup
    #[validate(custom = "validate_backup_type")]
    pub backup_type: String,

    /// Deskripsi (opsional)
    pub description: Option<String>,
}

fn validate_backup_type(backup_type: &str) -> Result<(), validator::ValidationError> {
    match backup_type {
        "full" | "database" | "homedir" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_backup_type")),
    }
}

/// Service Status
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String, // "running", "stopped", "failed"
    pub uptime: String,
    pub memory_usage: String,
}
