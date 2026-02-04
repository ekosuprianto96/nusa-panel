//! # Configuration Module
//!
//! Modul untuk manajemen konfigurasi aplikasi.
//! Membaca konfigurasi dari environment variables dan menyediakan
//! nilai default yang aman.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;

/// Konfigurasi aplikasi global
///
/// Singleton instance yang dapat diakses dari mana saja dalam aplikasi.
pub static CONFIG: Lazy<AppConfig> = Lazy::new(AppConfig::from_env);

/// Struktur konfigurasi aplikasi
///
/// Berisi semua pengaturan yang diperlukan untuk menjalankan aplikasi.
/// Nilai-nilai diambil dari environment variables dengan fallback ke default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// URL koneksi database MySQL
    pub database_url: String,

    /// Konfigurasi JWT
    pub jwt: JwtConfig,

    /// Mode environment (development, staging, production)
    pub environment: Environment,

    /// Konfigurasi keamanan
    pub security: SecurityConfig,

    /// Konfigurasi file management
    pub file: FileConfig,

    /// URL phpMyAdmin
    pub phpmyadmin_url: String,
}

/// Konfigurasi JWT (JSON Web Token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Secret key untuk signing JWT
    /// PENTING: Harus diganti dengan value yang aman di production!
    pub secret: String,

    /// Durasi token dalam detik (default: 24 jam)
    pub expiration: u64,

    /// Durasi refresh token dalam detik (default: 7 hari)
    pub refresh_expiration: u64,

    /// Issuer JWT
    pub issuer: String,
}

/// Konfigurasi keamanan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Origins yang diizinkan untuk CORS
    pub cors_allowed_origins: Vec<String>,

    /// Rate limit per menit
    pub rate_limit_per_minute: u32,

    /// Panjang minimum password
    pub password_min_length: usize,

    /// Memerlukan huruf besar dalam password
    pub password_require_uppercase: bool,

    /// Memerlukan angka dalam password
    pub password_require_number: bool,

    /// Memerlukan karakter spesial dalam password
    pub password_require_special: bool,

    /// Master key untuk enkripsi password (32 bytes, base64 encoded)
    /// Digunakan untuk SSO phpMyAdmin
    pub encryption_master_key: String,

    /// Path default untuk ModSecurity audit log
    pub modsecurity_audit_log_path: String,
}

/// Konfigurasi file management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Base path untuk home directory user
    pub user_home_base: String,

    /// Ukuran maksimum upload dalam bytes
    pub max_upload_size: u64,

    /// Ekstensi file yang dilarang untuk upload
    pub forbidden_extensions: Vec<String>,
}

/// Mode environment aplikasi
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

impl From<String> for Environment {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "staging" | "stage" => Self::Staging,
            _ => Self::Development,
        }
    }
}

impl AppConfig {
    /// Membuat konfigurasi dari environment variables
    ///
    /// # Returns
    /// AppConfig dengan nilai dari env vars atau default yang aman
    ///
    /// # Example
    /// ```rust
    /// let config = AppConfig::from_env();
    /// println!("Database URL: {}", config.database_url);
    /// ```
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:password@localhost:3306/nusa_panel".to_string()),

            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                    // WARNING: Default secret hanya untuk development!
                    if cfg!(debug_assertions) {
                        "dev-secret-key-not-for-production".to_string()
                    } else {
                        panic!("JWT_SECRET must be set in production!");
                    }
                }),
                expiration: env::var("JWT_EXPIRATION")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(86400), // 24 jam
                refresh_expiration: env::var("JWT_REFRESH_EXPIRATION")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(604800), // 7 hari
                issuer: "nusa-panel".to_string(),
            },

            environment: env::var("RUST_ENV")
                .unwrap_or_else(|_| "development".to_string())
                .into(),

            security: SecurityConfig {
                cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
                password_min_length: env::var("PASSWORD_MIN_LENGTH")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(8),
                password_require_uppercase: true,
                password_require_number: true,
                password_require_special: true,
                encryption_master_key: env::var("ENCRYPTION_MASTER_KEY").unwrap_or_else(|_| {
                    // Default key hanya untuk development
                    if cfg!(debug_assertions) {
                        "ZGV2LWtleS1ub3QtZm9yLXByb2R1Y3Rpb24tMzI=".to_string() // 32 bytes base64
                    } else {
                        panic!("ENCRYPTION_MASTER_KEY must be set in production!");
                    }
                }),
                modsecurity_audit_log_path: env::var("MODSEC_AUDIT_LOG_PATH")
                    .unwrap_or_else(|_| "/var/log/modsec_audit.log".to_string()),
            },

            file: FileConfig {
                user_home_base: env::var("USER_HOME_BASE")
                    .unwrap_or_else(|_| "/tmp/nusa-panel-users".to_string()),
                max_upload_size: env::var("MAX_UPLOAD_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(104857600), // 100MB
                forbidden_extensions: vec![
                    "exe".to_string(),
                    "sh".to_string(),
                    "bat".to_string(),
                    "cmd".to_string(),
                    "ps1".to_string(),
                    "vbs".to_string(),
                ],
            },

            phpmyadmin_url: env::var("PHPMYADMIN_URL")
                .unwrap_or_else(|_| "http://localhost/phpmyadmin".to_string()),
        }
    }

    /// Cek apakah aplikasi berjalan di mode production
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Cek apakah aplikasi berjalan di mode development
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_environment() {
        let env = Environment::default();
        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn test_environment_from_string() {
        assert_eq!(Environment::from("production".to_string()), Environment::Production);
        assert_eq!(Environment::from("staging".to_string()), Environment::Staging);
        assert_eq!(Environment::from("development".to_string()), Environment::Development);
        assert_eq!(Environment::from("unknown".to_string()), Environment::Development);
    }
}
