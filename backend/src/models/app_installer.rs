//! # App Installer Model
//!
//! Model dan DTO untuk module App Installer (WordPress, Laravel, dll).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Supported Application Types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    WordPress,
    Laravel,
    Joomla,
    // Add more supported apps here
}

/// Helper struct for installed app metadata
#[derive(Debug, Clone, Serialize)]
pub struct InstalledApp {
    pub id: String,
    pub user_id: String,
    pub domain_id: String,
    pub app_type: AppType,
    pub version: String,
    pub installation_path: String,
    pub db_name: String,
    pub db_user: String,
    pub created_at: DateTime<Utc>,
}

/// DTO Request untuk Install Aplikasi
#[derive(Debug, Deserialize, Validate)]
pub struct InstallAppRequest {
    /// Domain t target instalasi
    #[validate(length(min = 1))]
    pub domain_id: String,

    /// Tipe Aplikasi (wordpress, laravel)
    pub app_type: AppType,

    /// Path instalasi (opsional, default: public_html root)
    /// Contoh: "blog" -> domain.com/blog
    pub install_path: Option<String>,

    /// Judul Website (untuk WP config)
    #[validate(length(min = 1))]
    pub site_title: String,

    /// Admin Username (untuk WP login)
    #[validate(length(min = 3))]
    pub admin_username: String,

    /// Admin Password
    #[validate(length(min = 6))]
    pub admin_password: String,

    /// Admin Email
    #[validate(email)]
    pub admin_email: String,
}

/// Response setelah install berhasil
#[derive(Debug, Serialize)]
pub struct InstallAppResponse {
    pub success: bool,
    pub message: String,
    pub app_url: String,
    pub admin_url: String,
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
}
