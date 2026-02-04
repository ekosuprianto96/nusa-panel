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
use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    AdminUserStats, AssignPackageRequest, CreateUserByAdminRequest, CreateUserRequest, Package,
    User, UserResponse, UserResourceUsage,
};
use crate::config::CONFIG;
use walkdir::WalkDir;

pub struct UserServiceReal;

impl UserServiceReal {
    fn apply_disk_quota(username: &str, quota_mb: i64) -> ApiResult<()> {
        let mount_point = &CONFIG.file.user_home_base;
        let blocks = if quota_mb <= 0 { 0 } else { quota_mb * 1024 };

        let output = match Command::new("setquota")
            .arg("-u")
            .arg(username)
            .arg(blocks.to_string())
            .arg(blocks.to_string())
            .arg("0")
            .arg("0")
            .arg(mount_point)
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    tracing::warn!("setquota not found on system; skipping disk quota enforcement");
                    return Ok(());
                }
                return Err(ApiError::InternalError(format!(
                    "Failed to execute setquota: {}",
                    e
                )));
            }
        };

        if output.status.code().is_none() {
            if CONFIG.is_development() {
                tracing::warn!("setquota terminated by signal; skipping in development");
                return Ok(());
            }
        }

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            if CONFIG.is_development() {
                tracing::warn!("setquota failed in development mode: {}", error_msg);
                return Ok(());
            }
            return Err(ApiError::InternalError(format!("setquota error: {}", error_msg)));
        }

        Ok(())
    }

    fn apply_package_limits_os(username: &str, package: &Package) -> ApiResult<()> {
        // Disk quota (enforced via setquota)
        Self::apply_disk_quota(username, package.disk_quota_mb)?;

        // TODO: Bandwidth limits require traffic control setup (tc/htb) per user.
        // Placeholder until OS-level shaping strategy is defined.

        Ok(())
    }

    async fn get_default_package(pool: &MySqlPool) -> ApiResult<Package> {
        let package = sqlx::query_as::<_, Package>(
            "SELECT * FROM packages WHERE is_default = TRUE AND is_active = TRUE LIMIT 1",
        )
        .fetch_optional(pool)
        .await?;

        match package {
            Some(pkg) => Ok(pkg),
            None => {
                let fallback = sqlx::query_as::<_, Package>(
                    "SELECT * FROM packages WHERE is_active = TRUE ORDER BY sort_order ASC LIMIT 1",
                )
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| ApiError::NotFound("Default package".to_string()))?;
                Ok(fallback)
            }
        }
    }
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
        // Use DB username as system username for consistency with file_service
        let system_username = &request.username;

        // 3. Create Linux System User
        // useradd -m -s /bin/bash -d /home/username username
        let output = Command::new("useradd")
            .arg("-m") // Create home dir
            .arg("-s").arg("/bin/bash")
            .arg("-d").arg(format!("/home/{}", system_username))
            .arg(system_username)
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
            php_version: None,
            created_at: now,
            last_login_at: None,
            package_id: None,
            created_by: None,
            company: None,
            phone: None,
            address: None,
        })
    }

    /// Create user by admin/reseller with package assignment
    pub async fn create_by_admin(
        pool: &MySqlPool,
        request: CreateUserByAdminRequest,
        creator_id: &str,
        creator_role: &str,
    ) -> ApiResult<UserResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Only admin/reseller can create users
        if creator_role != "admin" && creator_role != "reseller" {
            return Err(ApiError::Forbidden);
        }

        // Reseller can only create 'user' role
        let role = request.role.unwrap_or_else(|| "user".to_string());
        if creator_role == "reseller" && role != "user" {
            return Err(ApiError::ValidationError(
                "Reseller hanya bisa membuat user dengan role 'user'".to_string(),
            ));
        }

        // Admin cannot create another admin
        if role == "admin" {
            return Err(ApiError::ValidationError(
                "Tidak bisa membuat user dengan role admin".to_string(),
            ));
        }

        // Check username uniqueness
        let username_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = ?"
        )
        .bind(&request.username)
        .fetch_one(pool)
        .await?;

        if username_exists > 0 {
            return Err(ApiError::AlreadyExists("Username".to_string()));
        }

        // Check email uniqueness
        let email_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE email = ?"
        )
        .bind(&request.email)
        .fetch_one(pool)
        .await?;

        if email_exists > 0 {
            return Err(ApiError::AlreadyExists("Email".to_string()));
        }

        // Validate package_id if provided
        if let Some(ref pkg_id) = request.package_id {
            let pkg_exists = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM packages WHERE id = ? AND is_active = TRUE"
            )
            .bind(pkg_id)
            .fetch_one(pool)
            .await?;

            if pkg_exists == 0 {
                return Err(ApiError::NotFound("Package".to_string()));
            }
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(request.password.as_bytes(), &salt)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .to_string();

        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        // Use DB username as system username for consistency with file_service
        let system_username = &request.username;
        let status = request.status.unwrap_or_else(|| "active".to_string());

        // Create Linux System User
        let output = Command::new("useradd")
            .arg("-m")
            .arg("-s").arg("/bin/bash")
            .arg("-d").arg(format!("/home/{}", system_username))
            .arg(system_username)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute useradd: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("System user creation failed (continuing anyway): {}", error_msg);
        }

        // Set System Password
        let mut child = Command::new("chpasswd")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ApiError::InternalError(format!("Failed to spawn chpasswd: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            let input = format!("{}:{}", system_username, request.password);
            let _ = stdin.write_all(input.as_bytes());
        }
        let _ = child.wait();

        // Setup directories
        let dirs = vec!["public_html", "backups", "logs"];
        for dir in dirs {
            let path = format!("/home/{}/{}", system_username, dir);
            let _ = fs::create_dir_all(&path);
            let _ = Command::new("chown")
                .arg("-R")
                .arg(format!("{}:{}", system_username, system_username))
                .arg(&path)
                .output();
        }

        // Insert into database
        sqlx::query(
            r#"
            INSERT INTO users (
                id, username, email, password_hash, first_name, last_name,
                company, phone, address, role, status, package_id, created_by,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&user_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.first_name)
        .bind(&request.last_name)
        .bind(&request.company)
        .bind(&request.phone)
        .bind(&request.address)
        .bind(&role)
        .bind(&status)
        .bind(&request.package_id)
        .bind(creator_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let package = if let Some(pkg_id) = &request.package_id {
            sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
                .bind(pkg_id)
                .fetch_one(pool)
                .await?
        } else {
            Self::get_default_package(pool).await?
        };

        Self::apply_package_limits_os(system_username, &package)?;

        tracing::info!("User created by {}: {} (role: {})", creator_role, request.email, role);

        Self::get_by_id(pool, &user_id).await
    }

    /// Get resource usage for a user (disk, bandwidth, counts)
    pub async fn get_resource_usage(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<UserResourceUsage> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("User".to_string()))?;

        // Resolve package (user package or default)
        let package = if let Some(pkg_id) = &user.package_id {
            sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
                .bind(pkg_id)
                .fetch_optional(pool)
                .await?
        } else {
            sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE is_default = TRUE LIMIT 1")
                .fetch_optional(pool)
                .await?
        };

        let (disk_limit_mb, bandwidth_limit_mb, domains_limit, databases_limit, email_accounts_limit) =
            match package {
                Some(pkg) => (
                    pkg.disk_quota_mb,
                    pkg.bandwidth_mb,
                    pkg.max_domains,
                    pkg.max_databases,
                    pkg.max_email_accounts,
                ),
                None => (10240, 102400, 10, 10, 50),
            };

        // Counts from DB
        let domains_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM domains WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await? as i32;

        let databases_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM managed_databases WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await? as i32;

        let email_accounts_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_accounts WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await? as i32;

        // Disk usage from filesystem (best-effort)
        let username = &user.username;
        let base = &CONFIG.file.user_home_base;
        let user_home = format!("{}/{}", base.trim_end_matches('/'), username);

        let disk_used_mb: i64 = tokio::task::spawn_blocking(move || {
            if !std::path::Path::new(&user_home).exists() {
                return 0_i64;
            }
            let mut total: u64 = 0;
            for entry in WalkDir::new(&user_home).into_iter().filter_map(Result::ok) {
                if entry.file_type().is_file() {
                    if let Ok(meta) = entry.metadata() {
                        total = total.saturating_add(meta.len());
                    }
                }
            }
            (total / (1024 * 1024)) as i64
        })
        .await
        .unwrap_or(0);

        // Bandwidth usage not tracked yet
        let bandwidth_used_mb = 0;

        Ok(UserResourceUsage {
            disk_used_mb,
            disk_limit_mb,
            bandwidth_used_mb,
            bandwidth_limit_mb,
            domains_count,
            domains_limit,
            databases_count,
            databases_limit,
            email_accounts_count,
            email_accounts_limit,
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

    /// Get admin user stats (counts)
    pub async fn get_admin_stats(pool: &MySqlPool) -> ApiResult<AdminUserStats> {
        let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;

        let active_users = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE status = 'active'",
        )
        .fetch_one(pool)
        .await?;

        let blocked_users = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE status = 'blocked'",
        )
        .fetch_one(pool)
        .await?;

        let since = Utc::now() - Duration::days(7);
        let new_signups_7d = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE created_at >= ?",
        )
        .bind(since)
        .fetch_one(pool)
        .await?;

        Ok(AdminUserStats {
            total_users,
            active_users,
            blocked_users,
            new_signups_7d,
        })
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

    /// Assign package to user (admin only)
    pub async fn assign_package(
        pool: &MySqlPool,
        user_id: &str,
        request: AssignPackageRequest,
    ) -> ApiResult<UserResponse> {
        // Check user exists
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound("User".to_string()))?;

        // Check package exists and is active
        let pkg_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM packages WHERE id = ? AND is_active = TRUE"
        )
        .bind(&request.package_id)
        .fetch_one(pool)
        .await?;

        if pkg_exists == 0 {
            return Err(ApiError::NotFound("Package".to_string()));
        }

        // Update user's package
        sqlx::query("UPDATE users SET package_id = ?, updated_at = ? WHERE id = ?")
            .bind(&request.package_id)
            .bind(Utc::now())
            .bind(user_id)
            .execute(pool)
            .await?;

        tracing::info!("Package {} assigned to user {}", request.package_id, user_id);

        let package = sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
            .bind(&request.package_id)
            .fetch_one(pool)
            .await?;

        Self::apply_package_limits_os(&user.username, &package)?;

        Self::get_by_id(pool, user_id).await
    }

    /// Get users with filters (for admin/reseller)
    pub async fn get_users_filtered(
        pool: &MySqlPool,
        page: i64,
        per_page: i64,
        role_filter: Option<&str>,
        status_filter: Option<&str>,
        package_filter: Option<&str>,
        created_by_filter: Option<&str>,
        search: Option<&str>,
    ) -> ApiResult<(Vec<UserResponse>, i64)> {
        let offset = (page - 1) * per_page;
        
        // Build dynamic WHERE clause
        let mut conditions = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(role) = role_filter {
            conditions.push("role = ?");
            bindings.push(role.to_string());
        }
        if let Some(status) = status_filter {
            conditions.push("status = ?");
            bindings.push(status.to_string());
        }
        if let Some(package) = package_filter {
            conditions.push("package_id = ?");
            bindings.push(package.to_string());
        }
        if let Some(creator) = created_by_filter {
            conditions.push("created_by = ?");
            bindings.push(creator.to_string());
        }
        if let Some(q) = search {
            conditions.push("(username LIKE ? OR email LIKE ? OR first_name LIKE ? OR last_name LIKE ?)");
            let search_pattern = format!("%{}%", q);
            bindings.push(search_pattern.clone());
            bindings.push(search_pattern.clone());
            bindings.push(search_pattern.clone());
            bindings.push(search_pattern);
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count query
        let count_sql = format!("SELECT COUNT(*) FROM users {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for b in &bindings {
            count_query = count_query.bind(b);
        }
        let total = count_query.fetch_one(pool).await?;

        // Data query
        let data_sql = format!(
            "SELECT * FROM users {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut data_query = sqlx::query_as::<_, User>(&data_sql);
        for b in &bindings {
            data_query = data_query.bind(b);
        }
        data_query = data_query.bind(per_page).bind(offset);
        
        let users = data_query.fetch_all(pool).await?;
        let responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

        Ok((responses, total))
    }
}
