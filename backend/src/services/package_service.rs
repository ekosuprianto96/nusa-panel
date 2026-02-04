//! # Package Service
//!
//! Business logic untuk package management.

use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{CreatePackageRequest, Package, PackageResponse, UpdatePackageRequest};

/// Service untuk operasi package
pub struct PackageService;

impl PackageService {
    /// Create new package (Admin only)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `request` - Package creation data
    ///
    /// # Returns
    /// PackageResponse of created package
    pub async fn create(pool: &MySqlPool, request: CreatePackageRequest) -> ApiResult<PackageResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Check name uniqueness
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM packages WHERE name = ?"
        )
        .bind(&request.name)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Package name".to_string()));
        }

        // If setting as default, unset other defaults
        if request.is_default.unwrap_or(false) {
            sqlx::query("UPDATE packages SET is_default = FALSE WHERE is_default = TRUE")
                .execute(pool)
                .await?;
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO packages (
                id, name, description, disk_quota_mb, bandwidth_mb,
                max_domains, max_subdomains, max_databases, max_email_accounts,
                max_ftp_accounts, max_cron_jobs, price_monthly, price_yearly,
                is_active, is_default, sort_order, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.disk_quota_mb.unwrap_or(1024))
        .bind(request.bandwidth_mb.unwrap_or(10240))
        .bind(request.max_domains.unwrap_or(1))
        .bind(request.max_subdomains.unwrap_or(5))
        .bind(request.max_databases.unwrap_or(1))
        .bind(request.max_email_accounts.unwrap_or(5))
        .bind(request.max_ftp_accounts.unwrap_or(2))
        .bind(request.max_cron_jobs.unwrap_or(2))
        .bind(request.price_monthly.unwrap_or(0.0))
        .bind(request.price_yearly.unwrap_or(0.0))
        .bind(request.is_active.unwrap_or(true))
        .bind(request.is_default.unwrap_or(false))
        .bind(request.sort_order.unwrap_or(0))
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let package = sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await?;

        tracing::info!("Package created: {}", package.name);

        Ok(PackageResponse::from(package))
    }

    /// Get all packages
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `include_inactive` - Whether to include inactive packages
    ///
    /// # Returns
    /// Vector of PackageResponse with users_count
    pub async fn get_all(
        pool: &MySqlPool,
        include_inactive: bool,
    ) -> ApiResult<Vec<PackageResponse>> {
        let query = if include_inactive {
            "SELECT * FROM packages ORDER BY sort_order ASC, name ASC"
        } else {
            "SELECT * FROM packages WHERE is_active = TRUE ORDER BY sort_order ASC, name ASC"
        };

        let packages = sqlx::query_as::<_, Package>(query)
            .fetch_all(pool)
            .await?;

        let mut responses = Vec::with_capacity(packages.len());
        
        for pkg in packages {
            // Count users for each package
            let users_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE package_id = ?"
            )
            .bind(&pkg.id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            responses.push(PackageResponse::from(pkg).with_users_count(users_count));
        }

        Ok(responses)
    }

    /// Get package by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `package_id` - Package ID
    ///
    /// # Returns
    /// PackageResponse
    pub async fn get_by_id(pool: &MySqlPool, package_id: &str) -> ApiResult<PackageResponse> {
        let package = sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
            .bind(package_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound("Package".to_string()))?;

        // Count users
        let users_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE package_id = ?"
        )
        .bind(package_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(PackageResponse::from(package).with_users_count(users_count))
    }

    /// Get default package
    ///
    /// # Returns
    /// PackageResponse of default package
    pub async fn get_default(pool: &MySqlPool) -> ApiResult<PackageResponse> {
        let package = sqlx::query_as::<_, Package>(
            "SELECT * FROM packages WHERE is_default = TRUE AND is_active = TRUE LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;

        match package {
            Some(pkg) => Ok(PackageResponse::from(pkg)),
            None => {
                // Fallback to first active package
                let pkg = sqlx::query_as::<_, Package>(
                    "SELECT * FROM packages WHERE is_active = TRUE ORDER BY sort_order ASC LIMIT 1"
                )
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| ApiError::NotFound("Default package".to_string()))?;

                Ok(PackageResponse::from(pkg))
            }
        }
    }

    /// Update package (Admin only)
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `package_id` - Package ID
    /// * `request` - Update data
    ///
    /// # Returns
    /// Updated PackageResponse
    pub async fn update(
        pool: &MySqlPool,
        package_id: &str,
        request: UpdatePackageRequest,
    ) -> ApiResult<PackageResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Get existing package
        let package = sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
            .bind(package_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound("Package".to_string()))?;

        // Check name uniqueness if changing name
        if let Some(ref new_name) = request.name {
            if new_name != &package.name {
                let existing = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM packages WHERE name = ? AND id != ?"
                )
                .bind(new_name)
                .bind(package_id)
                .fetch_one(pool)
                .await?;

                if existing > 0 {
                    return Err(ApiError::AlreadyExists("Package name".to_string()));
                }
            }
        }

        // If setting as default, unset other defaults
        if request.is_default.unwrap_or(false) && !package.is_default {
            sqlx::query("UPDATE packages SET is_default = FALSE WHERE is_default = TRUE")
                .execute(pool)
                .await?;
        }

        // Build update
        let name = request.name.unwrap_or(package.name);
        let description = request.description.or(package.description);
        let disk_quota_mb = request.disk_quota_mb.unwrap_or(package.disk_quota_mb);
        let bandwidth_mb = request.bandwidth_mb.unwrap_or(package.bandwidth_mb);
        let max_domains = request.max_domains.unwrap_or(package.max_domains);
        let max_subdomains = request.max_subdomains.unwrap_or(package.max_subdomains);
        let max_databases = request.max_databases.unwrap_or(package.max_databases);
        let max_email_accounts = request.max_email_accounts.unwrap_or(package.max_email_accounts);
        let max_ftp_accounts = request.max_ftp_accounts.unwrap_or(package.max_ftp_accounts);
        let max_cron_jobs = request.max_cron_jobs.unwrap_or(package.max_cron_jobs);
        let price_monthly = request.price_monthly.unwrap_or(package.price_monthly);
        let price_yearly = request.price_yearly.unwrap_or(package.price_yearly);
        let is_active = request.is_active.unwrap_or(package.is_active);
        let is_default = request.is_default.unwrap_or(package.is_default);
        let sort_order = request.sort_order.unwrap_or(package.sort_order);

        sqlx::query(
            r#"
            UPDATE packages SET
                name = ?, description = ?, disk_quota_mb = ?, bandwidth_mb = ?,
                max_domains = ?, max_subdomains = ?, max_databases = ?, max_email_accounts = ?,
                max_ftp_accounts = ?, max_cron_jobs = ?, price_monthly = ?, price_yearly = ?,
                is_active = ?, is_default = ?, sort_order = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(disk_quota_mb)
        .bind(bandwidth_mb)
        .bind(max_domains)
        .bind(max_subdomains)
        .bind(max_databases)
        .bind(max_email_accounts)
        .bind(max_ftp_accounts)
        .bind(max_cron_jobs)
        .bind(price_monthly)
        .bind(price_yearly)
        .bind(is_active)
        .bind(is_default)
        .bind(sort_order)
        .bind(Utc::now())
        .bind(package_id)
        .execute(pool)
        .await?;

        // Fetch updated package
        let updated = sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
            .bind(package_id)
            .fetch_one(pool)
            .await?;

        tracing::info!("Package updated: {}", updated.name);

        Ok(PackageResponse::from(updated))
    }

    /// Delete package (Admin only)
    ///
    /// Cannot delete if users are assigned to this package.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `package_id` - Package ID
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn delete(pool: &MySqlPool, package_id: &str) -> ApiResult<()> {
        // Get package
        let package = sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE id = ?")
            .bind(package_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound("Package".to_string()))?;

        // Check if users are assigned
        let users_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE package_id = ?"
        )
        .bind(package_id)
        .fetch_one(pool)
        .await?;

        if users_count > 0 {
            return Err(ApiError::ValidationError(format!(
                "Tidak dapat menghapus package. Masih ada {} user yang menggunakan package ini.",
                users_count
            )));
        }

        // Cannot delete default package
        if package.is_default {
            return Err(ApiError::ValidationError(
                "Tidak dapat menghapus default package".to_string()
            ));
        }

        // Delete package
        sqlx::query("DELETE FROM packages WHERE id = ?")
            .bind(package_id)
            .execute(pool)
            .await?;

        tracing::info!("Package deleted: {}", package.name);

        Ok(())
    }
}
