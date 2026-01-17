//! # Database Management Service
//!
//! Business logic untuk managed database operations.
//! Database customer dipisah dari database system NusaPanel.
//!
//! ## Arsitektur
//! - Database system (nusa_panel) hanya diakses oleh API
//! - Database customer (userid_dbname) diakses via phpMyAdmin
//! - Setiap user hanya bisa melihat database miliknya sendiri

use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::config::CONFIG;
use crate::errors::{ApiError, ApiResult};
use crate::models::{
    CreateDatabaseRequest, CreateDatabaseUserRequest, DatabaseUser, DatabaseUserResponse,
    ManagedDatabase, ManagedDatabaseResponse, PhpMyAdminInfo, UpdateDatabaseRequest,
    UpdateDatabaseUserRequest, SUPPORTED_CHARSETS, SUPPORTED_COLLATIONS,
};
use crate::utils::password;

/// Service untuk managed database operations
pub struct DatabaseService;

impl DatabaseService {
    // ==========================================
    // DATABASE OPERATIONS
    // ==========================================

    /// Get all databases untuk user
    pub async fn get_user_databases(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<ManagedDatabaseResponse>> {
        let databases = sqlx::query_as::<_, ManagedDatabase>(
            "SELECT * FROM managed_databases WHERE user_id = ? ORDER BY db_name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut responses = Vec::with_capacity(databases.len());
        for db in databases {
            let users_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM database_users WHERE database_id = ?",
            )
            .bind(&db.id)
            .fetch_one(pool)
            .await? as i32;

            let mut response = ManagedDatabaseResponse::from(db);
            response.users_count = users_count;
            responses.push(response);
        }

        Ok(responses)
    }

    /// Get database by ID
    pub async fn get_by_id(
        pool: &MySqlPool,
        database_id: &str,
        user_id: &str,
    ) -> ApiResult<ManagedDatabaseResponse> {
        let db = sqlx::query_as::<_, ManagedDatabase>(
            "SELECT * FROM managed_databases WHERE id = ?",
        )
        .bind(database_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Database".to_string()))?;

        if db.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let users_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM database_users WHERE database_id = ?",
        )
        .bind(&db.id)
        .fetch_one(pool)
        .await? as i32;

        let mut response = ManagedDatabaseResponse::from(db);
        response.users_count = users_count;

        Ok(response)
    }

    /// Create new database
    ///
    /// Membuat database terpisah dari database system NusaPanel.
    /// Nama database akan di-prefix dengan user ID untuk isolasi.
    pub async fn create(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateDatabaseRequest,
    ) -> ApiResult<ManagedDatabaseResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Generate prefixed database name (userid_dbname)
        // Use first 8 chars of user_id to keep name short
        let short_user_id = &user_id[..8.min(user_id.len())];
        let db_name = format!("{}_{}", short_user_id, request.name.to_lowercase());

        // Validate charset and collation
        let charset = request.charset.unwrap_or_else(|| "utf8mb4".to_string());
        let collation = request
            .collation
            .unwrap_or_else(|| "utf8mb4_unicode_ci".to_string());

        if !SUPPORTED_CHARSETS.contains(&charset.as_str()) {
            return Err(ApiError::ValidationError(format!(
                "Charset '{}' tidak didukung",
                charset
            )));
        }

        if !SUPPORTED_COLLATIONS.contains(&collation.as_str()) {
            return Err(ApiError::ValidationError(format!(
                "Collation '{}' tidak didukung",
                collation
            )));
        }

        // Check if database already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM managed_databases WHERE db_name = ?",
        )
        .bind(&db_name)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Database".to_string()));
        }

        // Create the actual MySQL database
        // Note: This requires the panel's MySQL user to have CREATE privilege
        let create_db_sql = format!(
            "CREATE DATABASE IF NOT EXISTS `{}` CHARACTER SET {} COLLATE {}",
            db_name, charset, collation
        );

        sqlx::query(&create_db_sql).execute(pool).await.map_err(|e| {
            tracing::error!("Failed to create MySQL database: {}", e);
            ApiError::InternalError("Failed to create database".to_string())
        })?;

        // Record in managed_databases table
        let database_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO managed_databases (id, user_id, db_name, description, size_bytes, charset, collation, created_at, updated_at)
            VALUES (?, ?, ?, ?, 0, ?, ?, ?, ?)
            "#,
        )
        .bind(&database_id)
        .bind(user_id)
        .bind(&db_name)
        .bind(&request.description)
        .bind(&charset)
        .bind(&collation)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!(
            "Database created: {} for user {}",
            db_name,
            user_id
        );

        Self::get_by_id(pool, &database_id, user_id).await
    }

    /// Update database
    pub async fn update(
        pool: &MySqlPool,
        database_id: &str,
        user_id: &str,
        request: UpdateDatabaseRequest,
    ) -> ApiResult<ManagedDatabaseResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let db = sqlx::query_as::<_, ManagedDatabase>(
            "SELECT * FROM managed_databases WHERE id = ?",
        )
        .bind(database_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Database".to_string()))?;

        if db.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let description = request.description.or(db.description);

        sqlx::query(
            "UPDATE managed_databases SET description = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&description)
        .bind(Utc::now())
        .bind(database_id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, database_id, user_id).await
    }

    /// Delete database
    ///
    /// WARNING: This will permanently delete the MySQL database!
    pub async fn delete(pool: &MySqlPool, database_id: &str, user_id: &str) -> ApiResult<()> {
        let db = sqlx::query_as::<_, ManagedDatabase>(
            "SELECT * FROM managed_databases WHERE id = ?",
        )
        .bind(database_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Database".to_string()))?;

        if db.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Delete database users first
        let db_users = sqlx::query_as::<_, DatabaseUser>(
            "SELECT * FROM database_users WHERE database_id = ?",
        )
        .bind(database_id)
        .fetch_all(pool)
        .await?;

        for db_user in db_users {
            // Drop MySQL user
            let drop_user_sql = format!(
                "DROP USER IF EXISTS '{}'@'{}'",
                db_user.db_username, db_user.host
            );
            let _ = sqlx::query(&drop_user_sql).execute(pool).await;
        }

        // Delete from database_users table
        sqlx::query("DELETE FROM database_users WHERE database_id = ?")
            .bind(database_id)
            .execute(pool)
            .await?;

        // Drop the actual MySQL database
        let drop_db_sql = format!("DROP DATABASE IF EXISTS `{}`", db.db_name);
        sqlx::query(&drop_db_sql).execute(pool).await.map_err(|e| {
            tracing::error!("Failed to drop MySQL database: {}", e);
            ApiError::InternalError("Failed to delete database".to_string())
        })?;

        // Delete from managed_databases table
        sqlx::query("DELETE FROM managed_databases WHERE id = ?")
            .bind(database_id)
            .execute(pool)
            .await?;

        tracing::info!("Database deleted: {} by user {}", db.db_name, user_id);

        Ok(())
    }

    // ==========================================
    // DATABASE USER OPERATIONS
    // ==========================================

    /// Get all database users untuk user
    pub async fn get_database_users(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<DatabaseUserResponse>> {
        let db_users = sqlx::query_as::<_, DatabaseUser>(
            "SELECT * FROM database_users WHERE user_id = ? ORDER BY db_username",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut responses = Vec::with_capacity(db_users.len());
        for db_user in db_users {
            // Get database info
            let db = sqlx::query_as::<_, ManagedDatabase>(
                "SELECT * FROM managed_databases WHERE id = ?",
            )
            .bind(&db_user.database_id)
            .fetch_optional(pool)
            .await?;

            let db_name = db
                .map(|d| d.db_name)
                .unwrap_or_else(|| "unknown".to_string());

            responses.push(DatabaseUserResponse {
                id: db_user.id,
                user_id: db_user.user_id,
                database_id: db_user.database_id,
                db_name: db_name.clone(),
                db_username: db_user.db_username.clone(),
                host: db_user.host.clone(),
                privileges: db_user.privileges,
                is_active: db_user.is_active,
                created_at: db_user.created_at,
                updated_at: db_user.updated_at,
                phpmyadmin_info: PhpMyAdminInfo {
                    url: CONFIG.phpmyadmin_url.clone(),
                    username: db_user.db_username,
                    database: db_name,
                    mysql_host: "localhost".to_string(),
                },
            });
        }

        Ok(responses)
    }

    /// Get database user by ID
    pub async fn get_database_user_by_id(
        pool: &MySqlPool,
        db_user_id: &str,
        user_id: &str,
    ) -> ApiResult<DatabaseUserResponse> {
        let db_user = sqlx::query_as::<_, DatabaseUser>(
            "SELECT * FROM database_users WHERE id = ?",
        )
        .bind(db_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Database User".to_string()))?;

        if db_user.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let db = sqlx::query_as::<_, ManagedDatabase>(
            "SELECT * FROM managed_databases WHERE id = ?",
        )
        .bind(&db_user.database_id)
        .fetch_optional(pool)
        .await?;

        let db_name = db
            .map(|d| d.db_name)
            .unwrap_or_else(|| "unknown".to_string());

        Ok(DatabaseUserResponse {
            id: db_user.id,
            user_id: db_user.user_id,
            database_id: db_user.database_id,
            db_name: db_name.clone(),
            db_username: db_user.db_username.clone(),
            host: db_user.host.clone(),
            privileges: db_user.privileges,
            is_active: db_user.is_active,
            created_at: db_user.created_at,
            updated_at: db_user.updated_at,
            phpmyadmin_info: PhpMyAdminInfo {
                url: CONFIG.phpmyadmin_url.clone(),
                username: db_user.db_username,
                database: db_name,
                mysql_host: "localhost".to_string(),
            },
        })
    }

    /// Create database user
    ///
    /// User ini bisa digunakan untuk login ke phpMyAdmin.
    pub async fn create_database_user(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateDatabaseUserRequest,
    ) -> ApiResult<DatabaseUserResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Verify database ownership
        // Verify database ownership (if provided)
        let db = if let Some(db_id) = &request.database_id {
            if db_id.is_empty() {
                None
            } else {
                let db = sqlx::query_as::<_, ManagedDatabase>(
                    "SELECT * FROM managed_databases WHERE id = ?",
                )
                .bind(db_id)
                .fetch_optional(pool)
                .await?
                .ok_or(ApiError::NotFound("Database".to_string()))?;

                if db.user_id != user_id {
                    return Err(ApiError::Forbidden);
                }
                Some(db)
            }
        } else {
            None
        };



        // Generate prefixed username
        let short_user_id = &user_id[..8.min(user_id.len())];
        let db_username = format!("{}_{}", short_user_id, request.username.to_lowercase());
        let host = request.host.unwrap_or_else(|| "%".to_string());
        let privileges = request.privileges.unwrap_or_else(|| "ALL".to_string());

        // Check if username already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM database_users WHERE db_username = ? AND host = ?",
        )
        .bind(&db_username)
        .bind(&host)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Database Username".to_string()));
        }

        // Create MySQL user
        let create_user_sql = format!(
            "CREATE USER IF NOT EXISTS '{}'@'{}' IDENTIFIED BY '{}'",
            db_username, host, request.password
        );
        sqlx::query(&create_user_sql).execute(pool).await.map_err(|e| {
            tracing::error!("Failed to create MySQL user: {}", e);
            ApiError::InternalError("Failed to create database user".to_string())
        })?;

        // Grant privileges ONLY to this specific database
        // Grant privileges ONLY to this specific database (if database is selected)
        if let Some(db) = &db {
            let grant_sql = format!(
                "GRANT {} ON `{}`.* TO '{}'@'{}'",
                if privileges.to_uppercase() == "ALL" {
                    "ALL PRIVILEGES"
                } else {
                    &privileges
                },
                db.db_name,
                db_username,
                host
            );
            sqlx::query(&grant_sql).execute(pool).await.map_err(|e| {
                tracing::error!("Failed to grant privileges: {}", e);
                ApiError::InternalError("Failed to grant privileges".to_string())
            })?;
        }

        // Flush privileges
        let _ = sqlx::query("FLUSH PRIVILEGES").execute(pool).await;

        // Hash password for storage (for reference only, MySQL has its own hash)
        let password_hash = password::hash_password(&request.password)?;

        // Record in database_users table
        let db_user_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO database_users (id, user_id, database_id, db_username, password_hash, host, privileges, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, TRUE, ?, ?)
            "#,
        )
        .bind(&db_user_id)
        .bind(user_id)
        .bind(&request.database_id.clone().filter(|id| !id.is_empty()))
        .bind(&db_username)
        .bind(&password_hash)
        .bind(&host)
        .bind(&privileges)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!(
            "Database user created: {} for database {}",
            db_username,
            db.as_ref().map(|d| d.db_name.as_str()).unwrap_or("None")
        );

        Self::get_database_user_by_id(pool, &db_user_id, user_id).await
    }

    /// Update database user
    pub async fn update_database_user(
        pool: &MySqlPool,
        db_user_id: &str,
        user_id: &str,
        request: UpdateDatabaseUserRequest,
    ) -> ApiResult<DatabaseUserResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let db_user = sqlx::query_as::<_, DatabaseUser>(
            "SELECT * FROM database_users WHERE id = ?",
        )
        .bind(db_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Database User".to_string()))?;

        if db_user.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // 1. Handle Database Assignment Change
        let mut final_db_name = None;

        if let Some(ref new_db_id) = request.database_id {
            // Verify new database exists and belongs to user
            let new_db = sqlx::query_as::<_, ManagedDatabase>(
                "SELECT * FROM managed_databases WHERE id = ?",
            )
            .bind(new_db_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Database".to_string()))?;

            if new_db.user_id != user_id {
                return Err(ApiError::Forbidden);
            }

            final_db_name = Some(new_db.db_name.clone());

            // Update database_id in table
            sqlx::query("UPDATE database_users SET database_id = ? WHERE id = ?")
                .bind(new_db_id)
                .bind(db_user_id)
                .execute(pool)
                .await?;
            
            // Note: privileges will be granted below
        } else if let Some(ref current_db_id) = db_user.database_id {
             // If not changing DB, get current DB name for GRANT if needed
             let current_db = sqlx::query_as::<_, ManagedDatabase>(
                "SELECT * FROM managed_databases WHERE id = ?",
            )
            .bind(current_db_id)
            .fetch_optional(pool)
            .await?;
            
            if let Some(db) = current_db {
                final_db_name = Some(db.db_name);
            }
        }

        // Update MySQL password if provided
        if let Some(ref new_password) = request.password {
            let alter_user_sql = format!(
                "ALTER USER '{}'@'{}' IDENTIFIED BY '{}'",
                db_user.db_username, db_user.host, new_password
            );
            sqlx::query(&alter_user_sql).execute(pool).await.map_err(|e| {
                tracing::error!("Failed to update MySQL password: {}", e);
                ApiError::InternalError("Failed to update password".to_string())
            })?;
        }

        // Update record
        let password_hash = if let Some(ref new_password) = request.password {
            password::hash_password(new_password)?
        } else {
            db_user.password_hash.clone()
        };

        let is_active = request.is_active.unwrap_or(db_user.is_active);
        let privileges = request.privileges.clone().unwrap_or(db_user.privileges.clone()); // Use new privs or existing

        // Execute GRANT if we have a database assigned (either new or existing)
        if let Some(db_name) = final_db_name {
             // If privileges changed OR database changed, we need to GRANT again
             if request.privileges.is_some() || request.database_id.is_some() {
                let grant_sql = format!(
                    "GRANT {} ON `{}`.* TO '{}'@'{}'",
                    if privileges.to_uppercase() == "ALL" {
                        "ALL PRIVILEGES"
                    } else {
                        &privileges
                    },
                    db_name,
                    db_user.db_username,
                    db_user.host
                );
                
                // Log the grant attempt
                tracing::info!("Granting privileges to {} on {}: {}", db_user.db_username, db_name, privileges);

                sqlx::query(&grant_sql).execute(pool).await.map_err(|e| {
                    tracing::error!("Failed to grant privileges: {}", e);
                    ApiError::InternalError("Failed to grant privileges".to_string())
                })?;
                
                // Flush privileges
                let _ = sqlx::query("FLUSH PRIVILEGES").execute(pool).await;
             }
        }

        sqlx::query(
            "UPDATE database_users SET password_hash = ?, privileges = ?, is_active = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&password_hash)
        .bind(&privileges)
        .bind(is_active)
        .bind(Utc::now())
        .bind(db_user_id)
        .execute(pool)
        .await?;

        Self::get_database_user_by_id(pool, db_user_id, user_id).await
    }

    /// Delete database user
    pub async fn delete_database_user(
        pool: &MySqlPool,
        db_user_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let db_user = sqlx::query_as::<_, DatabaseUser>(
            "SELECT * FROM database_users WHERE id = ?",
        )
        .bind(db_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Database User".to_string()))?;

        if db_user.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Drop MySQL user
        let drop_user_sql = format!(
            "DROP USER IF EXISTS '{}'@'{}'",
            db_user.db_username, db_user.host
        );
        let _ = sqlx::query(&drop_user_sql).execute(pool).await;

        // Flush privileges
        let _ = sqlx::query("FLUSH PRIVILEGES").execute(pool).await;

        // Delete from database_users table
        sqlx::query("DELETE FROM database_users WHERE id = ?")
            .bind(db_user_id)
            .execute(pool)
            .await?;

        tracing::info!("Database user deleted: {}", db_user.db_username);

        Ok(())
    }

    /// Get database count untuk user
    pub async fn get_database_count(pool: &MySqlPool, user_id: &str) -> ApiResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM managed_databases WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Get database user count untuk user
    pub async fn get_database_user_count(pool: &MySqlPool, user_id: &str) -> ApiResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM database_users WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }
}
