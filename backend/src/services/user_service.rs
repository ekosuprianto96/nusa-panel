//! # User Service
//!
//! Business logic untuk user management.

use chrono::Utc;
use sqlx::MySqlPool;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{UpdateUserRequest, User, UserResponse};

/// Service untuk operasi user
pub struct UserService;

impl UserService {
    /// Get user by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user yang dicari
    ///
    /// # Returns
    /// UserResponse jika ditemukan
    pub async fn get_by_id(pool: &MySqlPool, user_id: &str) -> ApiResult<UserResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        Ok(UserResponse::from(user))
    }

    /// Get all users (admin only)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `page` - Halaman (1-indexed)
    /// * `per_page` - Items per halaman
    ///
    /// # Returns
    /// Vector of UserResponse dengan total count
    pub async fn get_all(
        pool: &MySqlPool,
        page: i64,
        per_page: i64,
    ) -> ApiResult<(Vec<UserResponse>, i64)> {
        let offset = (page - 1) * per_page;

        // Get total count
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;

        // Get paginated users
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
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user yang akan diupdate
    /// * `request` - Data update
    /// * `requester_id` - ID user yang melakukan request
    /// * `requester_role` - Role user yang melakukan request
    ///
    /// # Returns
    /// Updated UserResponse
    pub async fn update(
        pool: &MySqlPool,
        user_id: &str,
        request: UpdateUserRequest,
        requester_id: &str,
        requester_role: &str,
    ) -> ApiResult<UserResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Check authorization: user can only update self, admin can update anyone
        if user_id != requester_id && requester_role != "admin" {
            return Err(ApiError::Forbidden);
        }

        // Get existing user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        // Check email uniqueness if changing email
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

        // Build update query dynamically
        let email = request.email.unwrap_or(user.email);
        let first_name = request.first_name.or(user.first_name);
        let last_name = request.last_name.or(user.last_name);

        sqlx::query(
            r#"
            UPDATE users 
            SET email = ?, first_name = ?, last_name = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&email)
        .bind(&first_name)
        .bind(&last_name)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;

        // Fetch updated user
        let updated_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        tracing::info!("User updated: {}", updated_user.username);

        Ok(UserResponse::from(updated_user))
    }

    /// Delete user (admin only)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user yang akan dihapus
    ///
    /// # Returns
    /// Ok(()) jika berhasil
    pub async fn delete(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        // Get user to check existence
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        // Prevent deleting admin users
        if user.role == "admin" {
            return Err(ApiError::ValidationError(
                "Tidak dapat menghapus admin user".to_string(),
            ));
        }

        // Delete user
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;

        tracing::info!("User deleted: {}", user.username);

        Ok(())
    }

    /// Update user status (admin only)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user
    /// * `status` - Status baru (active, inactive, blocked)
    pub async fn update_status(
        pool: &MySqlPool,
        user_id: &str,
        status: &str,
    ) -> ApiResult<UserResponse> {
        // Validate status
        let valid_statuses = ["active", "inactive", "pending", "blocked"];
        if !valid_statuses.contains(&status) {
            return Err(ApiError::ValidationError(format!(
                "Status tidak valid. Harus salah satu dari: {:?}",
                valid_statuses
            )));
        }

        // Get user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        // Update status
        sqlx::query("UPDATE users SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(Utc::now())
            .bind(user_id)
            .execute(pool)
            .await?;

        // Fetch updated user
        let updated_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        tracing::info!("User {} status changed to: {}", user.username, status);

        Ok(UserResponse::from(updated_user))
    }

    /// Update user role (admin only)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user
    /// * `role` - Role baru (admin, reseller, user)
    pub async fn update_role(
        pool: &MySqlPool,
        user_id: &str,
        role: &str,
    ) -> ApiResult<UserResponse> {
        // Validate role
        let valid_roles = ["admin", "reseller", "user"];
        if !valid_roles.contains(&role) {
            return Err(ApiError::ValidationError(format!(
                "Role tidak valid. Harus salah satu dari: {:?}",
                valid_roles
            )));
        }

        // Get user
        let _user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        // Update role
        sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
            .bind(role)
            .bind(Utc::now())
            .bind(user_id)
            .execute(pool)
            .await?;

        // Fetch updated user
        let updated_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        tracing::info!("User {} role changed to: {}", updated_user.username, role);

        Ok(UserResponse::from(updated_user))
    }
}
