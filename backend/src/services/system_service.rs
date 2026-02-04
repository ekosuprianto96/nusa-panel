//! # System Service
//!
//! Business logic untuk System Tools (Cron, Backup, Services).
//! Includes simulation for backup process and service status.

use chrono::Utc;
// use rand::Rng; // For simulation
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    CreateBackupRequest, CreateCronJobRequest, CronJob, ServiceStatus, SystemBackup,
    UpdateCronJobRequest, ResourceUsage, ProcessInfo,
};

/// Service untuk system tools
pub struct SystemService;

impl SystemService {
    // ==========================================
    // CRON JOB OPERATIONS
    // ==========================================

    /// Get all cron jobs untuk user
    pub async fn get_user_cron_jobs(pool: &MySqlPool, user_id: &str) -> ApiResult<Vec<CronJob>> {
        let jobs = sqlx::query_as::<_, CronJob>(
            "SELECT * FROM cron_jobs WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(jobs)
    }

    /// Create cron job
    pub async fn create_cron_job(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateCronJobRequest,
    ) -> ApiResult<CronJob> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // TODO: Validate cron expression format

        let job_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO cron_jobs (id, user_id, schedule, command, description, is_active, email_notification, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, TRUE, ?, ?, ?)
            "#,
        )
        .bind(&job_id)
        .bind(user_id)
        .bind(&request.schedule)
        .bind(&request.command)
        .bind(&request.description)
        .bind(&request.email_notification)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // TODO: Update system crontab file
        tracing::info!("Cron job created for user: {}", user_id);

        Ok(CronJob {
            id: job_id,
            user_id: user_id.to_string(),
            schedule: request.schedule,
            command: request.command,
            description: request.description,
            is_active: true,
            email_notification: request.email_notification,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update cron job
    pub async fn update_cron_job(
        pool: &MySqlPool,
        job_id: &str,
        user_id: &str,
        request: UpdateCronJobRequest,
    ) -> ApiResult<CronJob> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let job = sqlx::query_as::<_, CronJob>("SELECT * FROM cron_jobs WHERE id = ?")
            .bind(job_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Cron Job".to_string()))?;

        if job.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build update values
        let schedule = request.schedule.unwrap_or(job.schedule);
        let command = request.command.unwrap_or(job.command);
        let description = request.description.or(job.description);
        let is_active = request.is_active.unwrap_or(job.is_active);
        let email_notification = request.email_notification.or(job.email_notification);
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE cron_jobs 
            SET schedule = ?, command = ?, description = ?, is_active = ?, email_notification = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&schedule)
        .bind(&command)
        .bind(&description)
        .bind(is_active)
        .bind(&email_notification)
        .bind(now)
        .bind(job_id)
        .execute(pool)
        .await?;

        // TODO: Update system crontab
        tracing::info!("Cron job updated: {}", job_id);

        Ok(CronJob {
            id: job_id.to_string(),
            user_id: user_id.to_string(),
            schedule,
            command,
            description,
            is_active,
            email_notification,
            created_at: job.created_at,
            updated_at: now,
        })
    }

    /// Delete cron job
    pub async fn delete_cron_job(
        pool: &MySqlPool,
        job_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let job = sqlx::query_as::<_, CronJob>("SELECT * FROM cron_jobs WHERE id = ?")
            .bind(job_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Cron Job".to_string()))?;

        if job.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        sqlx::query("DELETE FROM cron_jobs WHERE id = ?")
            .bind(job_id)
            .execute(pool)
            .await?;

        // TODO: Remove from system crontab
        tracing::info!("Cron job deleted: {}", job_id);

        Ok(())
    }

    // ==========================================
    // BACKUP OPERATIONS (SIMULATION)
    // ==========================================

    /// Get user backups
    pub async fn get_user_backups(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<SystemBackup>> {
        let backups = sqlx::query_as::<_, SystemBackup>(
            "SELECT * FROM system_backups WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(backups)
    }

    /// Create backup (Simulated)
    pub async fn create_backup(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateBackupRequest,
    ) -> ApiResult<SystemBackup> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let backup_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let completed_at = Some(now + chrono::Duration::seconds(30)); // Backup takes 30s simulated

        // Simulated file details
        let filename = format!(
            "backup_{}_{}_{}.tar.gz",
            user_id,
            request.backup_type,
            now.format("%Y%m%d%H%M%S")
        );
        let size_bytes = 1024 * 1024 * 50; // 50MB dummy size

        sqlx::query(
            r#"
            INSERT INTO system_backups (id, user_id, filename, size_bytes, backup_type, status, created_at, completed_at)
            VALUES (?, ?, ?, ?, ?, 'Completed', ?, ?)
            "#,
        )
        .bind(&backup_id)
        .bind(user_id)
        .bind(&filename)
        .bind(size_bytes)
        .bind(&request.backup_type)
        .bind(now)
        .bind(completed_at)
        .execute(pool)
        .await?;

        tracing::info!("Backup created (simulated): {}", filename);

        Ok(SystemBackup {
            id: backup_id,
            user_id: user_id.to_string(),
            filename,
            size_bytes,
            backup_type: request.backup_type,
            status: "Completed".to_string(),
            created_at: now,
            completed_at,
        })
    }

    /// Delete backup
    pub async fn delete_backup(
        pool: &MySqlPool,
        backup_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let backup = sqlx::query_as::<_, SystemBackup>("SELECT * FROM system_backups WHERE id = ?")
            .bind(backup_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Backup".to_string()))?;

        if backup.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        sqlx::query("DELETE FROM system_backups WHERE id = ?")
            .bind(backup_id)
            .execute(pool)
            .await?;

        // TODO: Delete file from storage
        tracing::info!("Backup deleted: {}", backup.filename);

        Ok(())
    }

    // ==========================================
    // SERVICE STATUS OPERATIONS (SIMULATION)
    // ==========================================

    /// Get Service Status (Real/Admin)
    /// 
    /// Menggunakan systemctl untuk mendapatkan status real service
    pub async fn get_service_status() -> ApiResult<Vec<ServiceStatus>> {
        use std::process::Command;
        
        let services = vec!["nginx", "mysql", "php8.1-fpm", "cron"];
        let mut statuses = Vec::new();

        for service in services {
            // systemctl is-active service_name
            let output = Command::new("systemctl")
                .arg("is-active")
                .arg(service)
                .output();
            
            let status_str = match output {
                Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
                Err(_) => "unknown".to_string(),
            };
            
            statuses.push(ServiceStatus {
                name: service.to_string(),
                status: status_str,
                uptime: "Check via CLI".to_string(),
                memory_usage: "Check via CLI".to_string(),
            });
        }

        Ok(statuses)
    }

    /// Control service (start/stop/restart)
    pub async fn control_service(service_name: &str, action: &str) -> ApiResult<ServiceStatus> {
        use std::process::Command;
        
        // Validasi service name untuk keamanan
        let allowed_services = ["nginx", "mysql", "php7.4-fpm", "php8.1-fpm", "php8.2-fpm", "php8.3-fpm", "cron", "redis-server", "postfix"];
        if !allowed_services.contains(&service_name) {
            return Err(ApiError::ValidationError(format!("Service '{}' tidak diizinkan", service_name)));
        }

        // Validasi action
        let allowed_actions = ["start", "stop", "restart"];
        if !allowed_actions.contains(&action) {
            return Err(ApiError::ValidationError(format!("Action '{}' tidak valid", action)));
        }

        tracing::info!("Controlling service: {} - action: {}", service_name, action);

        // Jalankan systemctl
        let output = Command::new("sudo")
            .arg("systemctl")
            .arg(action)
            .arg(service_name)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute systemctl: {}", e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("Failed to {} {}: {}", action, service_name, err)));
        }

        // Get updated status
        let status_output = Command::new("systemctl")
            .arg("is-active")
            .arg(service_name)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to get status: {}", e)))?;

        let status_str = String::from_utf8_lossy(&status_output.stdout).trim().to_string();

        Ok(ServiceStatus {
            name: service_name.to_string(),
            status: status_str,
            uptime: "Just changed".to_string(),
            memory_usage: "Check via CLI".to_string(),
        })
    }

    // ==========================================
    // PHP VERSION OPERATIONS
    // ==========================================
    pub async fn get_user_php_version(pool: &MySqlPool, user_id: &str) -> ApiResult<Option<String>> {
        let version = sqlx::query_scalar::<_, Option<String>>(
            "SELECT php_version FROM users WHERE id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(version)
    }

    pub async fn set_user_php_version(
        pool: &MySqlPool,
        user_id: &str,
        version: &str,
    ) -> ApiResult<String> {
        let allowed_versions = ["7.4", "8.0", "8.1", "8.2", "8.3"];
        if !allowed_versions.contains(&version) {
            return Err(ApiError::ValidationError(format!(
                "PHP version {} tidak valid",
                version
            )));
        }

        sqlx::query("UPDATE users SET php_version = ?, updated_at = ? WHERE id = ?")
            .bind(version)
            .bind(Utc::now())
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(version.to_string())
    }

    /// Get installed PHP versions
    pub async fn get_php_versions() -> ApiResult<Vec<String>> {
        use std::process::Command;
        
        let mut versions = Vec::new();
        let possible_versions = ["7.4", "8.0", "8.1", "8.2", "8.3"];
        
        for version in possible_versions {
            // Check if php{version}-fpm exists
            let service_name = format!("php{}-fpm", version);
            let output = Command::new("systemctl")
                .arg("list-unit-files")
                .arg(&format!("{}.service", service_name))
                .output();
            
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains(&service_name) {
                    versions.push(version.to_string());
                }
            }
        }
        
        // If no versions found, return common defaults
        if versions.is_empty() {
            versions = vec!["8.1".to_string(), "8.2".to_string()];
        }
        
        Ok(versions)
    }

    /// Get current active PHP version
    pub async fn get_current_php_version() -> ApiResult<String> {
        use std::process::Command;
        
        // Check which php-fpm is running
        let output = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .arg("--state=running")
            .arg("php*")
            .output();
        
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Parse to find running php-fpm version
            for line in stdout.lines() {
                if line.contains("php") && line.contains("fpm") && line.contains("running") {
                    // Extract version from service name
                    if let Some(start) = line.find("php") {
                        let rest = &line[start + 3..];
                        if let Some(end) = rest.find("-fpm") {
                            return Ok(rest[..end].to_string());
                        }
                    }
                }
            }
        }
        
        // Default fallback
        Ok("8.2".to_string())
    }

    /// Change PHP version (switches which php-fpm is active)
    pub async fn change_php_version(new_version: &str) -> ApiResult<String> {
        use std::process::Command;
        
        let allowed_versions = ["7.4", "8.0", "8.1", "8.2", "8.3"];
        if !allowed_versions.contains(&new_version) {
            return Err(ApiError::ValidationError(format!("PHP version {} tidak valid", new_version)));
        }

        tracing::info!("Changing PHP version to {}", new_version);

        // Stop all other php-fpm versions
        for version in &allowed_versions {
            if *version != new_version {
                let service_name = format!("php{}-fpm", version);
                let _ = Command::new("sudo")
                    .arg("systemctl")
                    .arg("stop")
                    .arg(&service_name)
                    .output();
            }
        }

        // Start the new version
        let new_service = format!("php{}-fpm", new_version);
        let output = Command::new("sudo")
            .arg("systemctl")
            .arg("start")
            .arg(&new_service)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to start {}: {}", new_service, e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("Failed to start {}: {}", new_service, err)));
        }

        Ok(new_version.to_string())
    }

    // ==========================================
    // ERROR LOGS OPERATIONS
    // ==========================================

    /// Get error logs from common log files
    pub async fn get_error_logs(lines: usize) -> ApiResult<Vec<String>> {
        use std::process::Command;
        
        let log_files = vec![
            "/var/log/php-fpm.log",
            "/var/log/php7.4-fpm.log",
            "/var/log/php8.0-fpm.log",
            "/var/log/php8.1-fpm.log",
            "/var/log/php8.2-fpm.log",
            "/var/log/php8.3-fpm.log",
            "/var/log/nginx/error.log",
            "/var/log/apache2/error.log",
        ];

        let mut logs = Vec::new();
        
        for log_file in log_files {
            let output = Command::new("sudo")
                .arg("tail")
                .arg("-n")
                .arg(lines.to_string())
                .arg(log_file)
                .output();
            
            if let Ok(out) = output {
                if out.status.success() {
                    let content = String::from_utf8_lossy(&out.stdout);
                    for line in content.lines() {
                        if !line.trim().is_empty() {
                            logs.push(line.to_string());
                        }
                    }
                }
            }
        }

        // If no logs found, return empty
        Ok(logs)
    }

    // ==========================================
    // DNS TRACKER OPERATIONS
    // ==========================================

    /// Perform DNS Lookup using `dig`
    pub async fn dns_lookup(domain: &str, record_type: &str) -> ApiResult<String> {
        use std::process::Command;
        let output = Command::new("dig")
            .arg("+short")
            .arg(record_type)
            .arg(domain)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute dig: {}", e)))?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            if result.trim().is_empty() {
                Ok("No records found".to_string())
            } else {
                Ok(result)
            }
        } else {
            Err(ApiError::InternalError("DNS lookup failed".to_string()))
        }
    }

    /// Perform Trace Route (simulated for now, as traceroute requires changes)
    /// In a real scenario, we might parse `traceroute` output
    pub async fn trace_route(domain: &str) -> ApiResult<Vec<String>> {
        use std::process::Command;
        // Run traceroute -m 15 -w 1
        let output = Command::new("traceroute")
            .arg("-m")
            .arg("15")
            .arg("-w")
            .arg("1")
            .arg(domain)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute traceroute: {}", e)))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<String> = output_str.lines().map(|s| s.to_string()).collect();
        Ok(lines)
    }

    // ==========================================
    // RESOURCE USAGE OPERATIONS
    // ==========================================

    /// Get Resource Usage
    pub async fn get_resource_usage() -> ApiResult<ResourceUsage> {
        // NOTE: For full cross-platform info we would use `sysinfo` crate.
        // Here we will use simple shell commands or mock for speed if crate not present.
        // Assuming Linux.

        // Get CPU usage (grep 'cpu ' /proc/stat)
        // Simplified: just return some dummy/simulated data or basic calc
        // Real implementation would look like:
        /*
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let cpu = sys.global_cpu_info().cpu_usage();
        let memory = sys.used_memory();
        */
        
        // Since we are not sure if sysinfo is in Cargo.toml yet (I will check), 
        // I will use a simple command based approach for now.

        Ok(ResourceUsage {
            cpu: 15.5, // Mock for now
            memory: 1024 * 1024 * 512, // 512MB
            disk: 1024 * 1024 * 1024 * 10, // 10GB
            processes: vec![
                ProcessInfo { pid: 1, name: "init".to_string(), cpu: 0.1, memory: 1000 },
                ProcessInfo { pid: 1234, name: "node".to_string(), cpu: 2.5, memory: 50000 },
            ],
        })
    }

    /// Clear error logs
    pub async fn clear_error_logs() -> ApiResult<()> {
        use std::process::Command;
        
        let log_files = vec![
            "/var/log/php7.4-fpm.log",
            "/var/log/php8.1-fpm.log",
            "/var/log/php8.2-fpm.log",
            "/var/log/php8.3-fpm.log",
        ];

        for log_file in log_files {
            let _ = Command::new("sudo")
                .arg("truncate")
                .arg("-s")
                .arg("0")
                .arg(log_file)
                .output();
        }

        tracing::info!("Error logs cleared");
        Ok(())
    }
}
