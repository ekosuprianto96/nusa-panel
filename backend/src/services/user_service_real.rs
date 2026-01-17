//! # User Service (REAL IMPLEMENTATION)
//!
//! Implementasi nyata untuk User Management dengan integrasi Linux System User.
//! Gunakan file ini menggantikan `user_service.rs` saat deploy ke production server.
//!
//! Syarat Sistem:
//! - Root privileges (sudo) untuk menjalankan `useradd`, `passwd`, `chown`.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use chrono::Utc;
use sqlx::MySqlPool;
use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{CreateUserRequest, User, UserResponse};

pub struct UserServiceReal;

impl UserServiceReal {
    /// Create new user (DB + System User)
    pub async fn create(pool: &MySqlPool, request: CreateUserRequest) -> ApiResult<UserResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 1. Check if email exists
        let email_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(&request.email)
            .fetch_one(pool)
            .await?;

        if email_exists > 0 {
            return Err(ApiError::AlreadyExists("Email".to_string()));
        }

        // 2. Hash Password (untuk DB Login)
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(request.password.as_bytes(), &salt)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .to_string();

        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let system_username = format!("user_{}", user_id.replace("-", "").chars().take(8).collect::<String>());

        // 3. Create Linux System User
        // useradd -m -s /bin/bash -d /home/username username
        let output = Command::new("useradd")
            .arg("-m") // Create home dir
            .arg("-s").arg("/bin/bash")
            .arg("-d").arg(format!("/home/{}", system_username))
            .arg(&system_username)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute useradd: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("System User Creation Failed: {}", error_msg)));
        }

        // 4. Set System Password
        // echo "username:password" | chpasswd
        let mut child = Command::new("chpasswd")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ApiError::InternalError(format!("Failed to spawn chpasswd: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            let input = format!("{}:{}", system_username, request.password);
            stdin.write_all(input.as_bytes())
                .map_err(|e| ApiError::InternalError(format!("Failed to write to chpasswd: {}", e)))?;
        }

        if !child.wait().map_err(|_| ApiError::InternalError("Wait failed".to_string()))?.success() {
             return Err(ApiError::InternalError("Failed to set system password".to_string()));
        }

        // 5. Setup Web Directory Structure
        let dirs = vec!["public_html", "backups", "logs"];
        for dir in dirs {
            let path = format!("/home/{}/{}", system_username, dir);
            fs::create_dir_all(&path).map_err(|e| ApiError::InternalError(format!("Failed to create dir {}: {}", dir, e)))?;
            
            Command::new("chown")
                .arg("-R")
                .arg(format!("{}:{}", system_username, system_username))
                .arg(&path)
                .output()?;
        }
        
        // 6. Insert Into Database
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'user', 'active', ?, ?)
            "#,
        )
        .bind(&user_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!("User created: {} (system: {})", request.email, system_username);

        Ok(UserResponse {
            id: user_id,
            username: request.username,
            email: request.email,
            full_name: format!("{} {}", request.first_name.as_deref().unwrap_or_default(), request.last_name.as_deref().unwrap_or_default()),
            first_name: request.first_name,
            last_name: request.last_name,
            role: "user".to_string(),
            status: "active".to_string(),
            created_at: now,
            last_login_at: None,
        })
    }

    /// Get user by ID
    pub async fn get_by_id(pool: &MySqlPool, user_id: &str) -> ApiResult<UserResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        Ok(UserResponse::from(user))
    }

    /// Get all users (admin only)
    pub async fn get_all(
        pool: &MySqlPool,
        page: i64,
        per_page: i64,
    ) -> ApiResult<(Vec<UserResponse>, i64)> {
        let offset = (page - 1) * per_page;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;

        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let user_responses: Vec<UserResponse> =
            users.into_iter().map(UserResponse::from).collect();

        Ok((user_responses, total))
    }

    /// Update user
    pub async fn update(
        pool: &MySqlPool,
        user_id: &str,
        request: crate::models::UpdateUserRequest,
        requester_id: &str,
        requester_role: &str,
    ) -> ApiResult<UserResponse> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        if user_id != requester_id && requester_role != "admin" {
            return Err(ApiError::Forbidden);
        }

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        if let Some(ref new_email) = request.email {
            if new_email != &user.email {
                let existing = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM users WHERE email = ? AND id != ?",
                )
                .bind(new_email)
                .bind(user_id)
                .fetch_one(pool)
                .await?;

                if existing > 0 {
                    return Err(ApiError::AlreadyExists("Email".to_string()));
                }
            }
        }

        let email = request.email.unwrap_or(user.email);
        let first_name = request.first_name.or(user.first_name);
        let last_name = request.last_name.or(user.last_name);

        sqlx::query(
            "UPDATE users SET email = ?, first_name = ?, last_name = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&email).bind(&first_name).bind(&last_name).bind(Utc::now()).bind(user_id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, user_id).await
    }

    /// Delete user (admin only)
    pub async fn delete(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        if user.role == "admin" {
            return Err(ApiError::ValidationError("Tidak dapat menghapus admin user".to_string()));
        }

        // 1. Delete System User
        let system_username = format!("user_{}", user_id.replace("-", "").chars().take(8).collect::<String>());
        let _ = Command::new("userdel").arg("-r").arg(&system_username).output();

        // 2. Delete from DB
        sqlx::query("DELETE FROM users WHERE id = ?").bind(user_id).execute(pool).await?;

        Ok(())
    }

    /// Update user status (admin only)
    pub async fn update_status(
        pool: &MySqlPool,
        user_id: &str,
        status: &str,
    ) -> ApiResult<UserResponse> {
        sqlx::query("UPDATE users SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status).bind(Utc::now()).bind(user_id)
            .execute(pool)
            .await?;
        Self::get_by_id(pool, user_id).await
    }

    /// Update user role (admin only)
    pub async fn update_role(
        pool: &MySqlPool,
        user_id: &str,
        role: &str,
    ) -> ApiResult<UserResponse> {
        sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
            .bind(role).bind(Utc::now()).bind(user_id)
            .execute(pool)
            .await?;
        Self::get_by_id(pool, user_id).await
    }
}
