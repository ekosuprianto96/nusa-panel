//! # Package Model
//!
//! Model dan DTO untuk Hosting Package entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Package entity dari database
///
/// Representasi lengkap hosting package di database.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Package {
    /// Unique identifier
    pub id: String,

    /// Nama package (unique)
    pub name: String,

    /// Deskripsi package
    pub description: Option<String>,

    /// Disk quota dalam MB (0 = unlimited)
    pub disk_quota_mb: i64,

    /// Bandwidth dalam MB (0 = unlimited)
    pub bandwidth_mb: i64,

    /// Maksimum domain (0 = unlimited)
    pub max_domains: i32,

    /// Maksimum subdomain (0 = unlimited)
    pub max_subdomains: i32,

    /// Maksimum database (0 = unlimited)
    pub max_databases: i32,

    /// Maksimum email account (0 = unlimited)
    pub max_email_accounts: i32,

    /// Maksimum FTP account (0 = unlimited)
    pub max_ftp_accounts: i32,

    /// Maksimum cron jobs (0 = unlimited)
    pub max_cron_jobs: i32,

    /// Harga per bulan
    pub price_monthly: f64,

    /// Harga per tahun
    pub price_yearly: f64,

    /// Status aktif
    pub is_active: bool,

    /// Default package untuk user baru
    pub is_default: bool,

    /// Urutan tampilan
    pub sort_order: i32,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// DTO untuk response package
#[derive(Debug, Serialize)]
pub struct PackageResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub disk_quota_mb: i64,
    pub bandwidth_mb: i64,
    pub max_domains: i32,
    pub max_subdomains: i32,
    pub max_databases: i32,
    pub max_email_accounts: i32,
    pub max_ftp_accounts: i32,
    pub max_cron_jobs: i32,
    pub price_monthly: f64,
    pub price_yearly: f64,
    pub is_active: bool,
    pub is_default: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Jumlah user yang menggunakan package ini
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users_count: Option<i64>,
}

impl From<Package> for PackageResponse {
    fn from(pkg: Package) -> Self {
        Self {
            id: pkg.id,
            name: pkg.name,
            description: pkg.description,
            disk_quota_mb: pkg.disk_quota_mb,
            bandwidth_mb: pkg.bandwidth_mb,
            max_domains: pkg.max_domains,
            max_subdomains: pkg.max_subdomains,
            max_databases: pkg.max_databases,
            max_email_accounts: pkg.max_email_accounts,
            max_ftp_accounts: pkg.max_ftp_accounts,
            max_cron_jobs: pkg.max_cron_jobs,
            price_monthly: pkg.price_monthly,
            price_yearly: pkg.price_yearly,
            is_active: pkg.is_active,
            is_default: pkg.is_default,
            sort_order: pkg.sort_order,
            created_at: pkg.created_at,
            updated_at: pkg.updated_at,
            users_count: None,
        }
    }
}

impl PackageResponse {
    /// Set users_count
    pub fn with_users_count(mut self, count: i64) -> Self {
        self.users_count = Some(count);
        self
    }
}

/// DTO untuk membuat package baru
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePackageRequest {
    /// Nama package (3-100 karakter)
    #[validate(length(min = 3, max = 100, message = "Nama package harus 3-100 karakter"))]
    pub name: String,

    /// Deskripsi (opsional, max 500 karakter)
    #[validate(length(max = 500, message = "Deskripsi maksimal 500 karakter"))]
    pub description: Option<String>,

    /// Disk quota dalam MB (default: 1024)
    #[validate(range(min = 0, message = "Disk quota tidak boleh negatif"))]
    pub disk_quota_mb: Option<i64>,

    /// Bandwidth dalam MB (default: 10240)
    #[validate(range(min = 0, message = "Bandwidth tidak boleh negatif"))]
    pub bandwidth_mb: Option<i64>,

    /// Maksimum domain (default: 1)
    #[validate(range(min = 0, message = "Max domains tidak boleh negatif"))]
    pub max_domains: Option<i32>,

    /// Maksimum subdomain (default: 5)
    #[validate(range(min = 0, message = "Max subdomains tidak boleh negatif"))]
    pub max_subdomains: Option<i32>,

    /// Maksimum database (default: 1)
    #[validate(range(min = 0, message = "Max databases tidak boleh negatif"))]
    pub max_databases: Option<i32>,

    /// Maksimum email account (default: 5)
    #[validate(range(min = 0, message = "Max email accounts tidak boleh negatif"))]
    pub max_email_accounts: Option<i32>,

    /// Maksimum FTP account (default: 2)
    #[validate(range(min = 0, message = "Max FTP accounts tidak boleh negatif"))]
    pub max_ftp_accounts: Option<i32>,

    /// Maksimum cron jobs (default: 2)
    #[validate(range(min = 0, message = "Max cron jobs tidak boleh negatif"))]
    pub max_cron_jobs: Option<i32>,

    /// Harga per bulan
    #[validate(range(min = 0.0, message = "Harga tidak boleh negatif"))]
    pub price_monthly: Option<f64>,

    /// Harga per tahun
    #[validate(range(min = 0.0, message = "Harga tidak boleh negatif"))]
    pub price_yearly: Option<f64>,

    /// Status aktif (default: true)
    pub is_active: Option<bool>,

    /// Default package (default: false)
    pub is_default: Option<bool>,

    /// Urutan tampilan
    pub sort_order: Option<i32>,
}

/// DTO untuk update package
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePackageRequest {
    /// Nama package (3-100 karakter)
    #[validate(length(min = 3, max = 100, message = "Nama package harus 3-100 karakter"))]
    pub name: Option<String>,

    /// Deskripsi (opsional, max 500 karakter)
    #[validate(length(max = 500, message = "Deskripsi maksimal 500 karakter"))]
    pub description: Option<String>,

    /// Disk quota dalam MB
    #[validate(range(min = 0, message = "Disk quota tidak boleh negatif"))]
    pub disk_quota_mb: Option<i64>,

    /// Bandwidth dalam MB
    #[validate(range(min = 0, message = "Bandwidth tidak boleh negatif"))]
    pub bandwidth_mb: Option<i64>,

    /// Maksimum domain
    #[validate(range(min = 0, message = "Max domains tidak boleh negatif"))]
    pub max_domains: Option<i32>,

    /// Maksimum subdomain
    #[validate(range(min = 0, message = "Max subdomains tidak boleh negatif"))]
    pub max_subdomains: Option<i32>,

    /// Maksimum database
    #[validate(range(min = 0, message = "Max databases tidak boleh negatif"))]
    pub max_databases: Option<i32>,

    /// Maksimum email account
    #[validate(range(min = 0, message = "Max email accounts tidak boleh negatif"))]
    pub max_email_accounts: Option<i32>,

    /// Maksimum FTP account
    #[validate(range(min = 0, message = "Max FTP accounts tidak boleh negatif"))]
    pub max_ftp_accounts: Option<i32>,

    /// Maksimum cron jobs
    #[validate(range(min = 0, message = "Max cron jobs tidak boleh negatif"))]
    pub max_cron_jobs: Option<i32>,

    /// Harga per bulan
    #[validate(range(min = 0.0, message = "Harga tidak boleh negatif"))]
    pub price_monthly: Option<f64>,

    /// Harga per tahun
    #[validate(range(min = 0.0, message = "Harga tidak boleh negatif"))]
    pub price_yearly: Option<f64>,

    /// Status aktif
    pub is_active: Option<bool>,

    /// Default package
    pub is_default: Option<bool>,

    /// Urutan tampilan
    pub sort_order: Option<i32>,
}

/// Helper untuk format resource limit
impl Package {
    /// Format disk quota untuk display
    pub fn disk_quota_display(&self) -> String {
        if self.disk_quota_mb == 0 {
            "Unlimited".to_string()
        } else if self.disk_quota_mb >= 1024 {
            format!("{} GB", self.disk_quota_mb / 1024)
        } else {
            format!("{} MB", self.disk_quota_mb)
        }
    }

    /// Format bandwidth untuk display
    pub fn bandwidth_display(&self) -> String {
        if self.bandwidth_mb == 0 {
            "Unlimited".to_string()
        } else if self.bandwidth_mb >= 1024 {
            format!("{} GB", self.bandwidth_mb / 1024)
        } else {
            format!("{} MB", self.bandwidth_mb)
        }
    }

    /// Format limit untuk display (0 = Unlimited)
    pub fn limit_display(value: i32) -> String {
        if value == 0 {
            "Unlimited".to_string()
        } else {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_quota_display() {
        let pkg = Package {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            disk_quota_mb: 0,
            bandwidth_mb: 0,
            max_domains: 0,
            max_subdomains: 0,
            max_databases: 0,
            max_email_accounts: 0,
            max_ftp_accounts: 0,
            max_cron_jobs: 0,
            price_monthly: 0.0,
            price_yearly: 0.0,
            is_active: true,
            is_default: false,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(pkg.disk_quota_display(), "Unlimited");
    }

    #[test]
    fn test_limit_display() {
        assert_eq!(Package::limit_display(0), "Unlimited");
        assert_eq!(Package::limit_display(5), "5");
    }
}
