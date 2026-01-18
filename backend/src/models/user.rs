//! # User Model
//!
//! Model dan DTO untuk User entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Role user dalam sistem
///
/// Menentukan level akses dan permissions user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Administrator dengan akses penuh
    Admin,
    /// Reseller yang dapat membuat dan mengelola user
    Reseller,
    /// User biasa dengan akses terbatas
    User,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Reseller => write!(f, "reseller"),
            Self::User => write!(f, "user"),
        }
    }
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Self::Admin,
            "reseller" => Self::Reseller,
            _ => Self::User,
        }
    }
}

/// Status user
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// User aktif
    Active,
    /// User tidak aktif/suspended
    Inactive,
    /// User menunggu verifikasi email
    Pending,
    /// User diblokir
    Blocked,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// User entity dari database
///
/// Representasi lengkap user di database.
/// Password hash tidak di-serialize untuk keamanan.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    /// Unique identifier
    pub id: String,

    /// Username unik
    pub username: String,

    /// Email unik
    pub email: String,

    /// Password hash (Argon2)
    /// Tidak di-serialize untuk keamanan
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Nama depan
    pub first_name: Option<String>,

    /// Nama belakang
    pub last_name: Option<String>,

    /// Role user
    pub role: String,

    /// Status user
    pub status: String,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,

    /// Waktu login terakhir
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    /// Mendapatkan full name user
    pub fn full_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self.username.clone(),
        }
    }

    /// Mendapatkan role sebagai enum
    pub fn get_role(&self) -> UserRole {
        UserRole::from(self.role.clone())
    }

    /// Cek apakah user adalah admin
    pub fn is_admin(&self) -> bool {
        self.get_role() == UserRole::Admin
    }

    /// Cek apakah user adalah reseller atau admin
    pub fn is_reseller_or_above(&self) -> bool {
        matches!(self.get_role(), UserRole::Admin | UserRole::Reseller)
    }
}

/// DTO untuk response user (tanpa informasi sensitif)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            full_name: user.full_name(),
            id: user.id,
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
            status: user.status,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

/// DTO untuk registrasi user baru
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    /// Username (3-30 karakter, alphanumeric dan underscore)
    #[validate(length(min = 3, max = 30, message = "Username harus 3-30 karakter"))]
    #[validate(regex(
        path = "crate::models::user::USERNAME_REGEX",
        message = "Username hanya boleh berisi huruf, angka, dan underscore"
    ))]
    pub username: String,

    /// Email valid
    #[validate(email(message = "Format email tidak valid"))]
    pub email: String,

    /// Password (akan divalidasi terpisah untuk strength)
    #[validate(length(min = 8, message = "Password minimal 8 karakter"))]
    pub password: String,

    /// Nama depan (opsional)
    #[validate(length(max = 50, message = "Nama depan maksimal 50 karakter"))]
    pub first_name: Option<String>,

    /// Nama belakang (opsional)
    #[validate(length(max = 50, message = "Nama belakang maksimal 50 karakter"))]
    pub last_name: Option<String>,
}

/// DTO untuk login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// Username atau email
    #[validate(length(min = 1, message = "Username/email tidak boleh kosong"))]
    pub username_or_email: String,

    /// Password
    #[validate(length(min = 1, message = "Password tidak boleh kosong"))]
    pub password: String,
}

/// DTO untuk update user
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    /// Email baru (opsional)
    #[validate(email(message = "Format email tidak valid"))]
    pub email: Option<String>,

    /// Nama depan (opsional)
    #[validate(length(max = 50, message = "Nama depan maksimal 50 karakter"))]
    pub first_name: Option<String>,

    /// Nama belakang (opsional)
    #[validate(length(max = 50, message = "Nama belakang maksimal 50 karakter"))]
    pub last_name: Option<String>,
}

/// DTO untuk change password
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    /// Password lama untuk verifikasi
    #[validate(length(min = 1, message = "Password lama tidak boleh kosong"))]
    pub current_password: String,

    /// Password baru
    #[validate(length(min = 8, message = "Password baru minimal 8 karakter"))]
    pub new_password: String,

    /// Konfirmasi password baru
    #[validate(length(min = 1, message = "Konfirmasi password tidak boleh kosong"))]
    pub confirm_password: String,
}

impl ChangePasswordRequest {
    /// Validasi apakah password baru dan konfirmasi cocok
    pub fn passwords_match(&self) -> bool {
        self.new_password == self.confirm_password
    }
}

/// DTO untuk refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Regex untuk validasi username
pub static USERNAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_display() {
        assert_eq!(UserRole::Admin.to_string(), "admin");
        assert_eq!(UserRole::Reseller.to_string(), "reseller");
        assert_eq!(UserRole::User.to_string(), "user");
    }

    #[test]
    fn test_user_role_from_string() {
        assert_eq!(UserRole::from("admin".to_string()), UserRole::Admin);
        assert_eq!(UserRole::from("ADMIN".to_string()), UserRole::Admin);
        assert_eq!(UserRole::from("unknown".to_string()), UserRole::User);
    }

    #[test]
    fn test_change_password_match() {
        let req = ChangePasswordRequest {
            current_password: "old".to_string(),
            new_password: "newpass123".to_string(),
            confirm_password: "newpass123".to_string(),
        };
        assert!(req.passwords_match());

        let req2 = ChangePasswordRequest {
            current_password: "old".to_string(),
            new_password: "newpass123".to_string(),
            confirm_password: "different".to_string(),
        };
        assert!(!req2.passwords_match());
    }
}
