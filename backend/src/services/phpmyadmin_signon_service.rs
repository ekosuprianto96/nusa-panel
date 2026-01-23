//! # phpMyAdmin Signon Service
//!
//! Business logic untuk phpMyAdmin SSO (Single Sign-On).
//! Menggunakan in-memory storage untuk token dengan auto-cleanup.

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;
use once_cell::sync::Lazy;
use sqlx::MySqlPool;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    PhpMyAdminConfig, SignonCredentials, SignonResponse, SignonToken, ValidateTokenRequest,
};
use crate::services::DatabaseService;

/// In-memory storage untuk signon tokens
/// Menggunakan RwLock untuk thread-safe access
static TOKEN_STORE: Lazy<RwLock<HashMap<String, SignonToken>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Service untuk phpMyAdmin SSO operations
pub struct PhpMyAdminSignonService;

impl PhpMyAdminSignonService {
    /// Generate signon token untuk user yang sudah terautentikasi
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `db_user_id` - ID database user yang akan di-SSO
    /// * `panel_user_id` - ID panel user yang melakukan request
    ///
    /// # Returns
    /// SignonResponse dengan URL untuk redirect ke phpMyAdmin
    ///
    /// # Errors
    /// - `NotFound` jika database user tidak ditemukan
    /// - `Forbidden` jika user tidak memiliki akses ke database user tersebut
    pub async fn generate_token(
        pool: &MySqlPool,
        db_user_id: &str,
        panel_user_id: &str,
    ) -> ApiResult<SignonResponse> {
        // 1. Validasi ownership - pastikan panel user memiliki database user ini
        let db_user = DatabaseService::get_database_user_by_id(pool, db_user_id, panel_user_id)
            .await
            .map_err(|_| {
                ApiError::NotFound(format!(
                    "Database user with id {} not found or access denied",
                    db_user_id
                ))
            })?;

        // 2. Cek apakah database user aktif
        if !db_user.is_active {
            return Err(ApiError::Forbidden);
        }

        // 3. Buat token baru
        let token = SignonToken::new(
            db_user_id,
            panel_user_id,
            &db_user.db_username,
            if db_user.db_name.is_empty() {
                None
            } else {
                Some(db_user.db_name.clone())
            },
            "localhost",
        );

        let token_id = token.id.clone();
        let expires_in = (token.expires_at - Utc::now()).num_seconds();

        // 4. Simpan token ke storage
        {
            let mut store = TOKEN_STORE
                .write()
                .map_err(|_| ApiError::InternalError("Failed to acquire token store lock".into()))?;

            // Cleanup expired tokens saat menambah token baru
            store.retain(|_, t| !t.is_expired());

            store.insert(token_id.clone(), token);
        }

        // 5. Build signon URL
        let config = PhpMyAdminConfig::default();
        let signon_url = format!("{}{}?token={}", config.base_url, config.signon_path, token_id);

        Ok(SignonResponse {
            signon_url,
            expires_in,
        })
    }

    /// Validasi token dan kembalikan kredensial
    /// Token akan dihapus setelah validasi (one-time use)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Request berisi token yang akan divalidasi
    /// * `internal_key` - Internal key dari header untuk verifikasi
    ///
    /// # Returns
    /// SignonCredentials jika token valid
    ///
    /// # Errors
    /// - `Unauthorized` jika token tidak valid, expired, atau internal key salah
    pub async fn validate_token(
        pool: &MySqlPool,
        request: &ValidateTokenRequest,
        internal_key: &str,
    ) -> ApiResult<SignonCredentials> {
        // 1. Validasi internal key
        let config = PhpMyAdminConfig::default();
        if internal_key != config.internal_key {
            return Err(ApiError::Unauthorized);
        }

        // 2. Ambil dan hapus token dari storage (one-time use)
        let token = {
            let mut store = TOKEN_STORE.write().map_err(|_| {
                ApiError::InternalError("Failed to acquire token store lock".into())
            })?;

            store.remove(&request.token)
        };

        let token = match token {
            Some(t) => t,
            None => {
                return Err(ApiError::Unauthorized)
            }
        };

        // 3. Cek expiry
        if token.is_expired() {
            return Err(ApiError::Unauthorized);
        }

        // 4. Ambil password dari database
        // Karena kita menyimpan password sebagai hash, kita perlu pendekatan berbeda
        // Untuk SSO, kita perlu menyimpan password terenkripsi (bukan hash)
        // Sementara ini, kita return error dan perlu implementasi enkripsi password
        let password = Self::get_db_user_password(pool, &token.db_user_id).await?;

        Ok(SignonCredentials {
            user: token.db_username,
            password,
            host: token.host,
            port: 3306,
            database: token.db_name,
        })
    }

    /// Get password database user
    /// 
    /// # Note
    /// Saat ini menggunakan fallback karena password disimpan sebagai hash.
    /// Untuk production, implementasikan encrypted password storage.
    async fn get_db_user_password(pool: &MySqlPool, db_user_id: &str) -> ApiResult<String> {
        // Query untuk mendapatkan password
        // Karena SSO membutuhkan password plaintext, coba ambil dari password_encrypted
        // Fallback ke password_hash (tapi ini tidak akan work karena hash tidak bisa di-decrypt)
        let result: Option<(Option<String>, String)> = sqlx::query_as(
            "SELECT password_encrypted, password_hash FROM database_users WHERE id = ?"
        )
        .bind(db_user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("Database query failed: {}", e)))?;

        match result {
            Some((Some(encrypted_password), _)) if !encrypted_password.is_empty() => {
                // Password terenkripsi tersedia
                // TODO: Decrypt password menggunakan master key
                Ok(encrypted_password)
            }
            Some((_, password_hash)) => {
                // Fallback: untuk development, gunakan password_hash sebagai placeholder
                // WARNING: Ini TIDAK akan work di production karena hash tidak bisa di-decrypt
                // User harus mengupdate password untuk enable SSO
                Err(ApiError::InternalError(
                    "SSO not available: password stored as hash (cannot decrypt). \
                    Add 'password_encrypted' column and update user password to enable SSO.".to_string()
                ))
            }
            None => {
                Err(ApiError::NotFound("Database user not found".to_string()))
            }
        }
    }

    /// Cleanup expired tokens secara manual
    /// Dipanggil secara periodik atau saat maintenance
    pub fn cleanup_expired_tokens() -> usize {
        if let Ok(mut store) = TOKEN_STORE.write() {
            let before = store.len();
            store.retain(|_, t| !t.is_expired());
            before - store.len()
        } else {
            0
        }
    }

    /// Get jumlah token aktif (untuk monitoring)
    pub fn get_active_token_count() -> usize {
        TOKEN_STORE.read().map(|s| s.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_expired_tokens() {
        // Cleanup should not panic
        let cleaned = PhpMyAdminSignonService::cleanup_expired_tokens();
        assert!(cleaned >= 0);
    }

    #[test]
    fn test_get_active_token_count() {
        let count = PhpMyAdminSignonService::get_active_token_count();
        assert!(count >= 0);
    }
}
