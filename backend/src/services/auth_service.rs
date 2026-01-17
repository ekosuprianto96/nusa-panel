//! # Authentication Service
//!
//! Business logic untuk authentication: register, login, token refresh.

use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, RefreshTokenRequest, User,
    UserResponse,
};
use crate::utils::jwt::{create_token_pair, validate_refresh_token, TokenPair, TokenPayload};
use crate::utils::password::{hash_password, validate_password_strength, verify_password};

/// Service untuk operasi authentication
pub struct AuthService;

impl AuthService {
    /// Register user baru
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Data registrasi user
    ///
    /// # Returns
    /// UserResponse jika berhasil
    ///
    /// # Errors
    /// - ValidationError jika input tidak valid
    /// - AlreadyExists jika username/email sudah digunakan
    /// - WeakPassword jika password tidak memenuhi kriteria
    pub async fn register(pool: &MySqlPool, request: CreateUserRequest) -> ApiResult<UserResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Validate password strength
        validate_password_strength(&request.password)?;

        // Check if username already exists
        let existing_user = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = ?",
        )
        .bind(&request.username)
        .fetch_one(pool)
        .await?;

        if existing_user > 0 {
            return Err(ApiError::AlreadyExists("Username".to_string()));
        }

        // Check if email already exists
        let existing_email = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE email = ?",
        )
        .bind(&request.email)
        .fetch_one(pool)
        .await?;

        if existing_email > 0 {
            return Err(ApiError::AlreadyExists("Email".to_string()));
        }

        // Hash password
        let password_hash = hash_password(&request.password)?;

        // Generate UUID
        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Insert user
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, first_name, last_name, role, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, 'user', 'active', ?, ?)
            "#,
        )
        .bind(&user_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.first_name)
        .bind(&request.last_name)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Fetch created user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(pool)
            .await?;

        tracing::info!("New user registered: {}", user.username);

        Ok(UserResponse::from(user))
    }

    /// Login user
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Login credentials
    ///
    /// # Returns
    /// TokenPair (access + refresh token) jika berhasil
    ///
    /// # Errors
    /// - InvalidCredentials jika username/password salah
    /// - Forbidden jika user tidak aktif
    pub async fn login(pool: &MySqlPool, request: LoginRequest) -> ApiResult<TokenPair> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Find user by username or email
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ? OR email = ?",
        )
        .bind(&request.username_or_email)
        .bind(&request.username_or_email)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

        // Check if user is active
        if user.status != "active" {
            tracing::warn!("Login attempt for inactive user: {}", user.username);
            return Err(ApiError::Forbidden);
        }

        // Verify password
        if !verify_password(&request.password, &user.password_hash)? {
            tracing::warn!("Failed login attempt for user: {}", user.username);
            return Err(ApiError::InvalidCredentials);
        }

        // Update last login time
        sqlx::query("UPDATE users SET last_login_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(&user.id)
            .execute(pool)
            .await?;

        // Create token pair
        let payload = TokenPayload {
            user_id: Uuid::parse_str(&user.id).map_err(|_| {
                ApiError::InternalError("Invalid user ID format".to_string())
            })?,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
        };

        let tokens = create_token_pair(&payload)?;

        tracing::info!("User logged in: {}", user.username);

        Ok(tokens)
    }

    /// Refresh access token
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Refresh token
    ///
    /// # Returns
    /// TokenPair baru jika refresh token valid
    ///
    /// # Errors
    /// - InvalidToken jika refresh token tidak valid
    pub async fn refresh_token(
        pool: &MySqlPool,
        request: RefreshTokenRequest,
    ) -> ApiResult<TokenPair> {
        // Validate refresh token
        let claims = validate_refresh_token(&request.refresh_token)?;

        // Verify user still exists and is active
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(&claims.sub)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::InvalidToken)?;

        if user.status != "active" {
            return Err(ApiError::Forbidden);
        }

        // Create new token pair
        let payload = TokenPayload {
            user_id: Uuid::parse_str(&user.id).map_err(|_| {
                ApiError::InternalError("Invalid user ID format".to_string())
            })?,
            username: user.username,
            email: user.email,
            role: user.role,
        };

        create_token_pair(&payload)
    }

    /// Change user password
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user yang sedang login
    /// * `request` - Password change request
    ///
    /// # Returns
    /// Ok(()) jika berhasil
    ///
    /// # Errors
    /// - InvalidCredentials jika password lama salah
    /// - ValidationError jika password baru tidak cocok
    /// - WeakPassword jika password baru tidak memenuhi kriteria
    pub async fn change_password(
        pool: &MySqlPool,
        user_id: &str,
        request: ChangePasswordRequest,
    ) -> ApiResult<()> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Check passwords match
        if !request.passwords_match() {
            return Err(ApiError::ValidationError(
                "Password baru dan konfirmasi tidak cocok".to_string(),
            ));
        }

        // Validate new password strength
        validate_password_strength(&request.new_password)?;

        // Get user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        // Verify current password
        if !verify_password(&request.current_password, &user.password_hash)? {
            return Err(ApiError::InvalidCredentials);
        }

        // Hash new password
        let new_hash = hash_password(&request.new_password)?;

        // Update password
        sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
            .bind(&new_hash)
            .bind(Utc::now())
            .bind(user_id)
            .execute(pool)
            .await?;

        tracing::info!("Password changed for user: {}", user.username);

        Ok(())
    }
}
