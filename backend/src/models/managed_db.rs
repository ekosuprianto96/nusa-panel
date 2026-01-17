//! # Database (Managed) Model
//!
//! Model dan DTO untuk Managed Database operations.
//! Database customer dipisah dari database system NusaPanel.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Managed Database entity dari database
/// Ini adalah database yang dibuat oleh customer, bukan database system NusaPanel
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ManagedDatabase {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik database
    pub user_id: String,

    /// Nama database (format: userid_dbname)
    #[sqlx(rename = "db_name")]
    pub db_name: String,

    /// Deskripsi database
    pub description: Option<String>,

    /// Ukuran database dalam bytes
    pub size_bytes: i64,

    /// Charset (e.g., utf8mb4)
    pub charset: String,

    /// Collation (e.g., utf8mb4_unicode_ci)
    pub collation: String,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// Response DTO untuk Managed Database
#[derive(Debug, Serialize)]
pub struct ManagedDatabaseResponse {
    pub id: String,
    pub user_id: String,
    pub db_name: String,
    pub description: Option<String>,
    pub size_bytes: i64,
    pub size_mb: f64,
    pub charset: String,
    pub collation: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Jumlah database users yang memiliki akses
    pub users_count: i32,
}

impl From<ManagedDatabase> for ManagedDatabaseResponse {
    fn from(db: ManagedDatabase) -> Self {
        Self {
            id: db.id,
            user_id: db.user_id,
            db_name: db.db_name,
            description: db.description,
            size_bytes: db.size_bytes,
            size_mb: db.size_bytes as f64 / (1024.0 * 1024.0),
            charset: db.charset,
            collation: db.collation,
            created_at: db.created_at,
            updated_at: db.updated_at,
            users_count: 0,
        }
    }
}

/// DTO untuk membuat database baru
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDatabaseRequest {
    /// Nama database (akan di-prefix dengan user ID)
    #[validate(length(min = 1, max = 32, message = "Nama database harus 1-32 karakter"))]
    #[validate(regex(
        path = "crate::models::managed_db::DB_NAME_REGEX",
        message = "Nama database hanya boleh huruf, angka, dan underscore"
    ))]
    pub name: String,

    /// Deskripsi database (opsional)
    #[validate(length(max = 255, message = "Deskripsi maksimal 255 karakter"))]
    pub description: Option<String>,

    /// Charset (default: utf8mb4)
    pub charset: Option<String>,

    /// Collation (default: utf8mb4_unicode_ci)
    pub collation: Option<String>,
}

/// DTO untuk update database
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDatabaseRequest {
    /// Deskripsi baru
    #[validate(length(max = 255, message = "Deskripsi maksimal 255 karakter"))]
    pub description: Option<String>,
}

/// Database User entity - user yang dapat login ke phpMyAdmin
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DatabaseUser {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Managed Database ID
    pub database_id: Option<String>,

    /// Username MySQL (format: userid_username)
    pub db_username: String,

    /// Password hash (tidak di-serialize)
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Hostname yang diizinkan (%, localhost, atau IP)
    pub host: String,

    /// Privileges (ALL, SELECT, INSERT, UPDATE, DELETE, dll)
    pub privileges: String,

    /// Status aktif
    pub is_active: bool,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// Response DTO untuk Database User
#[derive(Debug, Serialize)]
pub struct DatabaseUserResponse {
    pub id: String,
    pub user_id: String,
    pub database_id: Option<String>,
    pub db_name: String,
    pub db_username: String,
    pub host: String,
    pub privileges: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// phpMyAdmin login info
    pub phpmyadmin_info: PhpMyAdminInfo,
}

/// Info untuk login ke phpMyAdmin
#[derive(Debug, Serialize)]
pub struct PhpMyAdminInfo {
    /// URL phpMyAdmin
    pub url: String,
    /// Username untuk login
    pub username: String,
    /// Database yang akan dipilih
    pub database: String,
    /// Host MySQL
    pub mysql_host: String,
}

/// DTO untuk membuat database user
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDatabaseUserRequest {
    /// Username (akan di-prefix dengan user ID)
    #[validate(length(min = 1, max = 16, message = "Username harus 1-16 karakter"))]
    #[validate(regex(
        path = "crate::models::managed_db::DB_USERNAME_REGEX",
        message = "Username hanya boleh huruf, angka, dan underscore"
    ))]
    pub username: String,

    /// Password untuk MySQL user
    #[validate(length(min = 8, max = 128, message = "Password harus 8-128 karakter"))]
    pub password: String,

    /// ID database yang akan diakses
    pub database_id: Option<String>,

    /// Host yang diizinkan (default: %)
    pub host: Option<String>,

    /// Privileges (default: ALL)
    pub privileges: Option<String>,
}

/// DTO untuk update database user
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDatabaseUserRequest {
    /// ID database baru (untuk assign/pindah database)
    pub database_id: Option<String>,

    /// Password baru
    #[validate(length(min = 8, max = 128, message = "Password harus 8-128 karakter"))]
    pub password: Option<String>,

    /// Host baru
    pub host: Option<String>,

    /// Privileges baru
    pub privileges: Option<String>,

    /// Status aktif
    pub is_active: Option<bool>,
}

/// Privilege levels yang tersedia
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatabasePrivilege {
    /// Full access
    All,
    /// Read only
    Select,
    /// Read and write
    SelectInsertUpdateDelete,
    /// Custom (specify in privileges string)
    Custom,
}

impl DatabasePrivilege {
    /// Convert ke MySQL GRANT statement privileges
    pub fn to_mysql_privileges(&self) -> &str {
        match self {
            Self::All => "ALL PRIVILEGES",
            Self::Select => "SELECT",
            Self::SelectInsertUpdateDelete => "SELECT, INSERT, UPDATE, DELETE",
            Self::Custom => "", // Will use the custom privileges string
        }
    }
}

/// Regex untuk validasi nama database
pub static DB_NAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap());

/// Regex untuk validasi username database
pub static DB_USERNAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap());

/// Daftar charsets yang didukung
pub const SUPPORTED_CHARSETS: &[&str] = &[
    "utf8mb4",
    "utf8",
    "latin1",
    "ascii",
    "binary",
];

/// Daftar collations yang didukung
pub const SUPPORTED_COLLATIONS: &[&str] = &[
    "utf8mb4_unicode_ci",
    "utf8mb4_general_ci",
    "utf8_general_ci",
    "latin1_swedish_ci",
    "ascii_general_ci",
    "binary",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_name_regex() {
        assert!(DB_NAME_REGEX.is_match("mydb"));
        assert!(DB_NAME_REGEX.is_match("my_database"));
        assert!(DB_NAME_REGEX.is_match("db123"));
        assert!(!DB_NAME_REGEX.is_match("123db")); // Can't start with number
        assert!(!DB_NAME_REGEX.is_match("my-db")); // No dash
        assert!(!DB_NAME_REGEX.is_match("my.db")); // No dot
    }

    #[test]
    fn test_db_username_regex() {
        assert!(DB_USERNAME_REGEX.is_match("admin"));
        assert!(DB_USERNAME_REGEX.is_match("user_1"));
        assert!(!DB_USERNAME_REGEX.is_match("1user"));
    }

    #[test]
    fn test_privilege_conversion() {
        assert_eq!(
            DatabasePrivilege::All.to_mysql_privileges(),
            "ALL PRIVILEGES"
        );
        assert_eq!(DatabasePrivilege::Select.to_mysql_privileges(), "SELECT");
    }
}
