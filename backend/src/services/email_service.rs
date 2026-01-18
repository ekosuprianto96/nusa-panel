//! # Email Service
//!
//! Business logic untuk email management operations.
//! Includes email accounts, forwarders, dan autoresponders.

use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    Autoresponder, AutoresponderResponse, CreateAutoresponderRequest, CreateEmailAccountRequest,
    CreateEmailForwarderRequest, Domain, EmailAccount, EmailAccountResponse, EmailForwarder,
    EmailForwarderResponse, UpdateAutoresponderRequest, UpdateEmailAccountRequest,
};
use crate::utils::password;

/// Service untuk email operations
pub struct EmailService;

impl EmailService {
    // ==========================================
    // EMAIL ACCOUNT OPERATIONS
    // ==========================================

    /// Get all email accounts untuk user
    pub async fn get_user_accounts(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<EmailAccountResponse>> {
        let accounts = sqlx::query_as::<_, EmailAccount>(
            "SELECT * FROM email_accounts WHERE user_id = ? ORDER BY email_address",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let responses: Vec<EmailAccountResponse> =
            accounts.into_iter().map(EmailAccountResponse::from).collect();

        Ok(responses)
    }

    /// Get email account by ID
    pub async fn get_account_by_id(
        pool: &MySqlPool,
        account_id: &str,
        user_id: &str,
    ) -> ApiResult<EmailAccountResponse> {
        let account = sqlx::query_as::<_, EmailAccount>(
            "SELECT * FROM email_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Email Account".to_string()))?;

        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        Ok(EmailAccountResponse::from(account))
    }

    /// Create email account
    pub async fn create_account(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateEmailAccountRequest,
    ) -> ApiResult<EmailAccountResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(&request.domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build email address
        let email_address = format!("{}@{}", request.username.to_lowercase(), domain.domain_name);

        // Check if email already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_accounts WHERE email_address = ?",
        )
        .bind(&email_address)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Email".to_string()));
        }

        // Hash password
        let password_hash = password::hash_password(&request.password)?;

        // Convert quota from MB to bytes
        let quota_bytes = request.quota_mb.unwrap_or(1024) * 1024 * 1024; // Default 1GB

        // Insert email account
        let account_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO email_accounts (id, user_id, domain_id, email_address, password_hash, quota_bytes, used_bytes, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, 0, TRUE, ?, ?)
            "#,
        )
        .bind(&account_id)
        .bind(user_id)
        .bind(&request.domain_id)
        .bind(&email_address)
        .bind(&password_hash)
        .bind(quota_bytes)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!("Email account created: {} for user {}", email_address, user_id);

        Self::get_account_by_id(pool, &account_id, user_id).await
    }

    /// Update email account
    pub async fn update_account(
        pool: &MySqlPool,
        account_id: &str,
        user_id: &str,
        request: UpdateEmailAccountRequest,
    ) -> ApiResult<EmailAccountResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let account = sqlx::query_as::<_, EmailAccount>(
            "SELECT * FROM email_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Email Account".to_string()))?;

        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build update values
        let password_hash = if let Some(ref new_password) = request.password {
            password::hash_password(new_password)?
        } else {
            account.password_hash
        };

        let quota_bytes = request
            .quota_mb
            .map(|mb| mb * 1024 * 1024)
            .unwrap_or(account.quota_bytes);

        let is_active = request.is_active.unwrap_or(account.is_active);

        // Update account
        sqlx::query(
            "UPDATE email_accounts SET password_hash = ?, quota_bytes = ?, is_active = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&password_hash)
        .bind(quota_bytes)
        .bind(is_active)
        .bind(Utc::now())
        .bind(account_id)
        .execute(pool)
        .await?;

        tracing::info!("Email account updated: {}", account.email_address);

        Self::get_account_by_id(pool, account_id, user_id).await
    }

    /// Delete email account
    pub async fn delete_account(
        pool: &MySqlPool,
        account_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let account = sqlx::query_as::<_, EmailAccount>(
            "SELECT * FROM email_accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Email Account".to_string()))?;

        if account.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Delete autoresponders first
        sqlx::query("DELETE FROM autoresponders WHERE email_account_id = ?")
            .bind(account_id)
            .execute(pool)
            .await?;

        // Delete email account
        sqlx::query("DELETE FROM email_accounts WHERE id = ?")
            .bind(account_id)
            .execute(pool)
            .await?;

        tracing::info!("Email account deleted: {}", account.email_address);

        Ok(())
    }

    // ==========================================
    // EMAIL FORWARDER OPERATIONS
    // ==========================================

    /// Get all forwarders untuk user
    pub async fn get_user_forwarders(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<EmailForwarderResponse>> {
        let forwarders = sqlx::query_as::<_, EmailForwarder>(
            "SELECT * FROM email_forwarders WHERE user_id = ? ORDER BY source_email",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let responses: Vec<EmailForwarderResponse> =
            forwarders.into_iter().map(EmailForwarderResponse::from).collect();

        Ok(responses)
    }

    /// Create forwarder
    pub async fn create_forwarder(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateEmailForwarderRequest,
    ) -> ApiResult<EmailForwarderResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(&request.domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build source email
        let source_email = format!(
            "{}@{}",
            request.source_username.to_lowercase(),
            domain.domain_name
        );

        // Check if forwarder already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_forwarders WHERE source_email = ? AND destination_email = ?",
        )
        .bind(&source_email)
        .bind(&request.destination_email)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Forwarder".to_string()));
        }

        // Insert forwarder
        let forwarder_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO email_forwarders (id, user_id, domain_id, source_email, destination_email, is_active, created_at)
            VALUES (?, ?, ?, ?, ?, TRUE, ?)
            "#,
        )
        .bind(&forwarder_id)
        .bind(user_id)
        .bind(&request.domain_id)
        .bind(&source_email)
        .bind(&request.destination_email)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!(
            "Email forwarder created: {} -> {}",
            source_email,
            request.destination_email
        );

        let forwarder = EmailForwarder {
            id: forwarder_id,
            user_id: user_id.to_string(),
            domain_id: request.domain_id,
            source_email,
            destination_email: request.destination_email,
            is_active: true,
            created_at: now,
        };

        Ok(EmailForwarderResponse::from(forwarder))
    }

    /// Delete forwarder
    pub async fn delete_forwarder(
        pool: &MySqlPool,
        forwarder_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let forwarder = sqlx::query_as::<_, EmailForwarder>(
            "SELECT * FROM email_forwarders WHERE id = ?",
        )
        .bind(forwarder_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Forwarder".to_string()))?;

        if forwarder.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        sqlx::query("DELETE FROM email_forwarders WHERE id = ?")
            .bind(forwarder_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "Email forwarder deleted: {} -> {}",
            forwarder.source_email,
            forwarder.destination_email
        );

        Ok(())
    }

    // ==========================================
    // AUTORESPONDER OPERATIONS
    // ==========================================

    /// Get all autoresponders untuk user
    pub async fn get_user_autoresponders(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<AutoresponderResponse>> {
        let autoresponders = sqlx::query_as::<_, Autoresponder>(
            "SELECT * FROM autoresponders WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut responses = Vec::with_capacity(autoresponders.len());
        for ar in autoresponders {
            // Get email address
            let email = sqlx::query_as::<_, EmailAccount>(
                "SELECT * FROM email_accounts WHERE id = ?",
            )
            .bind(&ar.email_account_id)
            .fetch_optional(pool)
            .await?;

            let email_address = email
                .map(|e| e.email_address)
                .unwrap_or_else(|| "unknown".to_string());

            responses.push(AutoresponderResponse {
                id: ar.id,
                user_id: ar.user_id,
                email_account_id: ar.email_account_id,
                email_address,
                subject: ar.subject,
                body: ar.body,
                start_date: ar.start_date,
                end_date: ar.end_date,
                is_active: ar.is_active,
                created_at: ar.created_at,
                updated_at: ar.updated_at,
            });
        }

        Ok(responses)
    }

    /// Create autoresponder
    pub async fn create_autoresponder(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateAutoresponderRequest,
    ) -> ApiResult<AutoresponderResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Verify email account ownership
        let email = sqlx::query_as::<_, EmailAccount>(
            "SELECT * FROM email_accounts WHERE id = ?",
        )
        .bind(&request.email_account_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Email Account".to_string()))?;

        if email.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Check if autoresponder already exists for this email
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM autoresponders WHERE email_account_id = ?",
        )
        .bind(&request.email_account_id)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists(
                "Autoresponder (hapus yang lama terlebih dahulu)".to_string(),
            ));
        }

        // Insert autoresponder
        let autoresponder_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO autoresponders (id, user_id, email_account_id, subject, body, start_date, end_date, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, TRUE, ?, ?)
            "#,
        )
        .bind(&autoresponder_id)
        .bind(user_id)
        .bind(&request.email_account_id)
        .bind(&request.subject)
        .bind(&request.body)
        .bind(request.start_date)
        .bind(request.end_date)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!(
            "Autoresponder created for email: {}",
            email.email_address
        );

        Ok(AutoresponderResponse {
            id: autoresponder_id,
            user_id: user_id.to_string(),
            email_account_id: request.email_account_id,
            email_address: email.email_address,
            subject: request.subject,
            body: request.body,
            start_date: request.start_date,
            end_date: request.end_date,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update autoresponder
    pub async fn update_autoresponder(
        pool: &MySqlPool,
        autoresponder_id: &str,
        user_id: &str,
        request: UpdateAutoresponderRequest,
    ) -> ApiResult<AutoresponderResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let ar = sqlx::query_as::<_, Autoresponder>(
            "SELECT * FROM autoresponders WHERE id = ?",
        )
        .bind(autoresponder_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Autoresponder".to_string()))?;

        if ar.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build update values
        let subject = request.subject.unwrap_or(ar.subject);
        let body = request.body.unwrap_or(ar.body);
        let start_date = request.start_date.or(ar.start_date);
        let end_date = request.end_date.or(ar.end_date);
        let is_active = request.is_active.unwrap_or(ar.is_active);
        let now = Utc::now();

        // Update autoresponder
        sqlx::query(
            "UPDATE autoresponders SET subject = ?, body = ?, start_date = ?, end_date = ?, is_active = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&subject)
        .bind(&body)
        .bind(start_date)
        .bind(end_date)
        .bind(is_active)
        .bind(now)
        .bind(autoresponder_id)
        .execute(pool)
        .await?;

        // Get email address
        let email = sqlx::query_as::<_, EmailAccount>(
            "SELECT * FROM email_accounts WHERE id = ?",
        )
        .bind(&ar.email_account_id)
        .fetch_optional(pool)
        .await?;

        let email_address = email
            .map(|e| e.email_address)
            .unwrap_or_else(|| "unknown".to_string());

        tracing::info!("Autoresponder updated for email: {}", email_address);

        Ok(AutoresponderResponse {
            id: autoresponder_id.to_string(),
            user_id: user_id.to_string(),
            email_account_id: ar.email_account_id,
            email_address,
            subject,
            body,
            start_date,
            end_date,
            is_active,
            created_at: ar.created_at,
            updated_at: now,
        })
    }

    /// Delete autoresponder
    pub async fn delete_autoresponder(
        pool: &MySqlPool,
        autoresponder_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let ar = sqlx::query_as::<_, Autoresponder>(
            "SELECT * FROM autoresponders WHERE id = ?",
        )
        .bind(autoresponder_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Autoresponder".to_string()))?;

        if ar.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        sqlx::query("DELETE FROM autoresponders WHERE id = ?")
            .bind(autoresponder_id)
            .execute(pool)
            .await?;

        tracing::info!("Autoresponder deleted");

        Ok(())
    }

    /// Get email account count untuk user
    pub async fn get_account_count(pool: &MySqlPool, user_id: &str) -> ApiResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_accounts WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }
}
