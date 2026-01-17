//! # FTP Service
//!
//! Business logic untuk FTP account management.

use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::config::CONFIG;
use crate::errors::{ApiError, ApiResult};
use crate::models::{
    ChangeFtpPasswordRequest, CreateFtpAccountRequest, FtpAccount, FtpAccountResponse,
    FtpServerInfo, UpdateFtpAccountRequest,
};
use crate::utils::password;

/// Service untuk FTP account operations
pub struct FtpService;

impl FtpService {
    /// Get all FTP accounts untuk user
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user pemilik accounts
    ///
    /// # Returns
    /// Vector of FtpAccountResponse
    pub async fn get_user_accounts(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<FtpAccountResponse>> {
        let accounts = sqlx::query_as::<_, FtpAccount>(
            "SELECT * FROM ftp_accounts WHERE user_id = ? ORDER BY ftp_username",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let responses: Vec<FtpAccountResponse> =
            accounts.into_iter().map(FtpAccountResponse::from).collect();

        Ok(responses)
    }

    /// Get FTP account by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `account_id` - ID FTP account
    /// * `user_id` - ID user untuk authorization check
    pub async fn get_by_id(
        pool: &MySqlPool,
        account_id: &str,
        user_id: &str,
    ) -> ApiResult<FtpAccountResponse> {
        let account = sqlx::query_as::<_, FtpAccount>(
            "SELECT * FROM ftp_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("FTP Account".to_string()))?;

        // Check authorization
        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        Ok(FtpAccountResponse::from(account))
    }

    /// Create new FTP account
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user pemilik
    /// * `request` - Data FTP account baru
    pub async fn create(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateFtpAccountRequest,
    ) -> ApiResult<FtpAccountResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Generate FTP username (format: username_userid)
        let ftp_username = format!("{}_{}", request.username.to_lowercase(), &user_id[..8]);

        // Check if username already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ftp_accounts WHERE ftp_username = ?",
        )
        .bind(&ftp_username)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("FTP Username".to_string()));
        }

        // Hash password
        let password_hash = password::hash_password(&request.password)?;

        // Generate home directory
        let home_directory = request.home_directory.unwrap_or_else(|| {
            format!("{}/{}", CONFIG.file.user_home_base, user_id)
        });

        // Convert quota from MB to bytes
        let quota_bytes = request.quota_mb.unwrap_or(0) * 1024 * 1024;

        // Generate UUID
        let account_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Insert FTP account
        sqlx::query(
            r#"
            INSERT INTO ftp_accounts (id, user_id, ftp_username, password_hash, home_directory, is_active, quota_bytes, used_bytes, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, TRUE, ?, 0, ?, ?)
            "#,
        )
        .bind(&account_id)
        .bind(user_id)
        .bind(&ftp_username)
        .bind(&password_hash)
        .bind(&home_directory)
        .bind(quota_bytes)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!(
            "FTP account created: {} for user {}",
            ftp_username,
            user_id
        );

        // Fetch and return created account
        Self::get_by_id(pool, &account_id, user_id).await
    }

    /// Update FTP account
    pub async fn update(
        pool: &MySqlPool,
        account_id: &str,
        user_id: &str,
        request: UpdateFtpAccountRequest,
    ) -> ApiResult<FtpAccountResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Get existing account
        let account = sqlx::query_as::<_, FtpAccount>(
            "SELECT * FROM ftp_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("FTP Account".to_string()))?;

        // Check authorization
        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build update values
        let password_hash = if let Some(ref new_password) = request.password {
            password::hash_password(new_password)?
        } else {
            account.password_hash
        };

        let home_directory = request.home_directory.unwrap_or(account.home_directory);
        let is_active = request.is_active.unwrap_or(account.is_active);
        let quota_bytes = request
            .quota_mb
            .map(|mb| mb * 1024 * 1024)
            .unwrap_or(account.quota_bytes);

        // Update account
        sqlx::query(
            r#"
            UPDATE ftp_accounts 
            SET password_hash = ?, home_directory = ?, is_active = ?, quota_bytes = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&password_hash)
        .bind(&home_directory)
        .bind(is_active)
        .bind(quota_bytes)
        .bind(Utc::now())
        .bind(account_id)
        .execute(pool)
        .await?;

        tracing::info!("FTP account updated: {}", account.ftp_username);

        Self::get_by_id(pool, account_id, user_id).await
    }

    /// Change FTP password
    pub async fn change_password(
        pool: &MySqlPool,
        account_id: &str,
        user_id: &str,
        request: ChangeFtpPasswordRequest,
    ) -> ApiResult<()> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Get existing account
        let account = sqlx::query_as::<_, FtpAccount>(
            "SELECT * FROM ftp_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("FTP Account".to_string()))?;

        // Check authorization
        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Hash new password
        let password_hash = password::hash_password(&request.new_password)?;

        // Update password
        sqlx::query(
            "UPDATE ftp_accounts SET password_hash = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&password_hash)
        .bind(Utc::now())
        .bind(account_id)
        .execute(pool)
        .await?;

        tracing::info!("FTP password changed: {}", account.ftp_username);

        Ok(())
    }

    /// Delete FTP account
    pub async fn delete(pool: &MySqlPool, account_id: &str, user_id: &str) -> ApiResult<()> {
        // Get account
        let account = sqlx::query_as::<_, FtpAccount>(
            "SELECT * FROM ftp_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("FTP Account".to_string()))?;

        // Check authorization
        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Delete account
        sqlx::query("DELETE FROM ftp_accounts WHERE id = ?")
            .bind(account_id)
            .execute(pool)
            .await?;

        tracing::info!("FTP account deleted: {}", account.ftp_username);

        Ok(())
    }

    /// Toggle FTP account status
    pub async fn toggle_status(
        pool: &MySqlPool,
        account_id: &str,
        user_id: &str,
    ) -> ApiResult<FtpAccountResponse> {
        // Get existing account
        let account = sqlx::query_as::<_, FtpAccount>(
            "SELECT * FROM ftp_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("FTP Account".to_string()))?;

        // Check authorization
        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Toggle status
        let new_status = !account.is_active;

        sqlx::query(
            "UPDATE ftp_accounts SET is_active = ?, updated_at = ? WHERE id = ?",
        )
        .bind(new_status)
        .bind(Utc::now())
        .bind(account_id)
        .execute(pool)
        .await?;

        tracing::info!(
            "FTP account status toggled: {} -> {}",
            account.ftp_username,
            if new_status { "active" } else { "inactive" }
        );

        Self::get_by_id(pool, account_id, user_id).await
    }

    /// Get FTP server info
    pub fn get_server_info() -> FtpServerInfo {
        // TODO: Load from config
        FtpServerInfo::default()
    }

    /// Get FTP account count untuk user
    pub async fn get_account_count(pool: &MySqlPool, user_id: &str) -> ApiResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ftp_accounts WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Update used bytes untuk FTP account (dipanggil oleh FTP server)
    pub async fn update_used_bytes(
        pool: &MySqlPool,
        ftp_username: &str,
        used_bytes: i64,
    ) -> ApiResult<()> {
        sqlx::query(
            "UPDATE ftp_accounts SET used_bytes = ?, updated_at = ? WHERE ftp_username = ?",
        )
        .bind(used_bytes)
        .bind(Utc::now())
        .bind(ftp_username)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record FTP login
    pub async fn record_login(pool: &MySqlPool, ftp_username: &str) -> ApiResult<()> {
        sqlx::query(
            "UPDATE ftp_accounts SET last_login = ? WHERE ftp_username = ?",
        )
        .bind(Utc::now())
        .bind(ftp_username)
        .execute(pool)
        .await?;

        Ok(())
    }
}
