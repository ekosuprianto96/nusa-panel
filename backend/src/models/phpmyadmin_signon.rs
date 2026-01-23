//! # phpMyAdmin Signon Model
//!
//! Model dan DTO untuk phpMyAdmin SSO (Single Sign-On) operations.
//! Memungkinkan user untuk login ke phpMyAdmin tanpa memasukkan kredensial.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Token untuk SSO phpMyAdmin
/// Token ini memiliki masa hidup singkat (30 detik) dan hanya bisa digunakan sekali
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignonToken {
    /// Unique identifier token
    pub id: String,

    /// ID database user yang akan di-login
    pub db_user_id: String,

    /// ID panel user yang melakukan request
    pub panel_user_id: String,

    /// MySQL username
    pub db_username: String,

    /// Nama database yang akan diakses
    pub db_name: Option<String>,

    /// MySQL host
    pub host: String,

    /// Waktu pembuatan token
    pub created_at: DateTime<Utc>,

    /// Waktu kadaluarsa token
    pub expires_at: DateTime<Utc>,
}

impl SignonToken {
    /// Membuat token baru dengan TTL 30 detik
    ///
    /// # Arguments
    /// * `db_user_id` - ID database user yang akan di-SSO
    /// * `panel_user_id` - ID panel user yang melakukan request
    /// * `db_username` - MySQL username
    /// * `db_name` - Nama database (opsional)
    /// * `host` - MySQL host
    ///
    /// # Returns
    /// Token baru yang valid selama 30 detik
    pub fn new(
        db_user_id: &str,
        panel_user_id: &str,
        db_username: &str,
        db_name: Option<String>,
        host: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            db_user_id: db_user_id.to_string(),
            panel_user_id: panel_user_id.to_string(),
            db_username: db_username.to_string(),
            db_name,
            host: host.to_string(),
            created_at: now,
            expires_at: now + Duration::seconds(30),
        }
    }

    /// Cek apakah token sudah kadaluarsa
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Response DTO untuk generate signon token
#[derive(Debug, Serialize)]
pub struct SignonResponse {
    /// URL lengkap untuk SSO ke phpMyAdmin
    pub signon_url: String,

    /// Detik tersisa sampai token kadaluarsa
    pub expires_in: i64,
}

/// Request DTO untuk validasi token (dari phpMyAdmin signon script)
#[derive(Debug, Deserialize)]
pub struct ValidateTokenRequest {
    /// Token yang akan divalidasi
    pub token: String,
}

/// Response DTO berisi kredensial untuk phpMyAdmin
/// Hanya dikembalikan jika token valid
#[derive(Debug, Serialize)]
pub struct SignonCredentials {
    /// MySQL username
    pub user: String,

    /// MySQL password (plaintext untuk signon)
    pub password: String,

    /// MySQL host
    pub host: String,

    /// MySQL port
    pub port: u16,

    /// Database yang akan dipilih (opsional)
    pub database: Option<String>,
}

/// Konfigurasi phpMyAdmin
#[derive(Debug, Clone)]
pub struct PhpMyAdminConfig {
    /// Base URL phpMyAdmin (e.g., "https://panel.example.com/phpmyadmin")
    pub base_url: String,

    /// Path ke signon script
    pub signon_path: String,

    /// Internal API key untuk validasi dari signon script
    pub internal_key: String,
}

impl Default for PhpMyAdminConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("PHPMYADMIN_URL")
                .unwrap_or_else(|_| "http://localhost/phpmyadmin".to_string()),
            signon_path: "/signon.php".to_string(),
            internal_key: std::env::var("PMA_INTERNAL_KEY")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signon_token_creation() {
        let token = SignonToken::new(
            "db-user-123",
            "panel-user-456",
            "myuser",
            Some("mydb".to_string()),
            "localhost",
        );

        assert!(!token.id.is_empty());
        assert_eq!(token.db_user_id, "db-user-123");
        assert_eq!(token.panel_user_id, "panel-user-456");
        assert!(!token.is_expired());
    }

    #[test]
    fn test_signon_token_expiry() {
        let mut token = SignonToken::new(
            "db-user-123",
            "panel-user-456",
            "myuser",
            None,
            "localhost",
        );

        // Token baru tidak expired
        assert!(!token.is_expired());

        // Set expires_at ke masa lalu
        token.expires_at = Utc::now() - Duration::seconds(1);
        assert!(token.is_expired());
    }
}
