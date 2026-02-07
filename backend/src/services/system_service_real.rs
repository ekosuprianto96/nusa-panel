//! # System Service (REAL IMPLEMENTATION)
//!
//! Implementasi nyata untuk System Tools (Cron, Backup, Services).
//! Gunakan file ini menggantikan `system_service.rs` saat deploy ke production server.

use chrono::Utc;
use sqlx::MySqlPool;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    CreateBackupRequest, CreateCronJobRequest, CronJob, ServiceStatus, SystemBackup,
    UpdateCronJobRequest, ResourceUsage, ProcessInfo,
};
use crate::services::PhpPoolService;
use crate::services::WebServerServiceReal;

pub struct SystemServiceReal;

impl SystemServiceReal {
    // ==========================================
    // CRON JOB OPERATIONS (REAL)
    // ==========================================

    /// Helper untuk update crontab system user
    async fn update_user_crontab(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        // Ambil semua cronjob user
        let jobs = sqlx::query_as::<_, CronJob>(
            "SELECT * FROM cron_jobs WHERE user_id = ? AND is_active = TRUE",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let system_username = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        
        // Format crontab content
        let mut crontab_content = String::new();
        crontab_content.push_str(&format!("# NusaPanel Cron for {}\n", system_username));
        crontab_content.push_str("MAILTO=\"\"\n"); // Disable default mailto

        // Ensure log directory exists
        let log_dir = format!("/home/{}/cron_logs", system_username);
        let _ = Command::new("mkdir").arg("-p").arg(&log_dir).output();
        let _ = Command::new("chown").arg("-R").arg(&system_username).arg(&log_dir).output();

        for job in jobs {
            // "0 * * * * command >> /home/user/cron_logs/job_id.log 2>&1 # job_id"
            let log_file = format!("{}/{}.log", log_dir, job.id);
            crontab_content.push_str(&format!("{} {} >> {} 2>&1 # {}\n", job.schedule, job.command, log_file, job.id));
        }

        // Tulis ke stdin command `crontab -u user -`
        let mut child = Command::new("crontab")
            .arg("-u")
            .arg(&system_username)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ApiError::InternalError(format!("Failed to spawn crontab: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(crontab_content.as_bytes())
                .map_err(|e| ApiError::InternalError(format!("Failed to write to crontab stdin: {}", e)))?;
        }

        let output = child.wait_with_output()
            .map_err(|e| ApiError::InternalError(format!("Failed to wait crontab: {}", e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("Crontab error: {}", err)));
        }

        Ok(())
    }

    /// Initialize crontab for user (called during registration)
    pub async fn init_user_crontab(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        Self::update_user_crontab(pool, user_id).await
    }

    pub async fn create_cron_job(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateCronJobRequest,
    ) -> ApiResult<CronJob> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 1. Simpan ke database
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

        // 2. Update real System Crontab
        Self::update_user_crontab(pool, user_id).await?;

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

    pub async fn update_cron_job(
        pool: &MySqlPool,
        job_id: &str,
        user_id: &str,
        request: UpdateCronJobRequest,
    ) -> ApiResult<CronJob> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // ... Fetch dan Validate ownership sama dengan simulasi ...
        let job = sqlx::query_as::<_, CronJob>("SELECT * FROM cron_jobs WHERE id = ?").bind(job_id).fetch_optional(pool).await?.ok_or(ApiError::NotFound("Cron".to_string()))?;
        if job.user_id != user_id { return Err(ApiError::Forbidden); }

        let schedule = request.schedule.unwrap_or(job.schedule);
        let command = request.command.unwrap_or(job.command);
        let description = request.description.or(job.description);
        let is_active = request.is_active.unwrap_or(job.is_active);
        let email_notification = request.email_notification.or(job.email_notification);
        let now = Utc::now();

        sqlx::query(
            "UPDATE cron_jobs SET schedule = ?, command = ?, description = ?, is_active = ?, email_notification = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&schedule).bind(&command).bind(&description).bind(is_active).bind(&email_notification).bind(now).bind(job_id)
        .execute(pool)
        .await?;

        // Update real System Crontab
        Self::update_user_crontab(pool, user_id).await?;

        Ok(CronJob {
            id: job_id.to_string(), user_id: user_id.to_string(), schedule, command, description, is_active, email_notification, created_at: job.created_at, updated_at: now
        })
    }

    pub async fn delete_cron_job(
        pool: &MySqlPool,
        job_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let job = sqlx::query_as::<_, CronJob>("SELECT * FROM cron_jobs WHERE id = ?").bind(job_id).fetch_optional(pool).await?.ok_or(ApiError::NotFound("Cron".to_string()))?;
        if job.user_id != user_id { return Err(ApiError::Forbidden); }

        sqlx::query("DELETE FROM cron_jobs WHERE id = ?").bind(job_id).execute(pool).await?;

        // Update real System Crontab
        Self::update_user_crontab(pool, user_id).await?;

        Ok(())
    }

    /// Get cron job logs
    pub async fn get_cron_logs(pool: &MySqlPool, job_id: &str, user_id: &str) -> ApiResult<String> {
        let job = sqlx::query_as::<_, CronJob>("SELECT * FROM cron_jobs WHERE id = ?").bind(job_id).fetch_optional(pool).await?.ok_or(ApiError::NotFound("Cron".to_string()))?;
        if job.user_id != user_id { return Err(ApiError::Forbidden); }

        let system_username = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let log_file = format!("/home/{}/cron_logs/{}.log", system_username, job_id);
        
        // Read file
        match fs::read_to_string(&log_file) {
            Ok(content) => {
                // If content is too large, take last 50KB
                if content.len() > 50_000 {
                    let split_idx = content.len() - 50_000;
                    Ok(format!("... (Truncated)\n{}", &content[split_idx..]))
                } else {
                    Ok(content)
                }
            },
            Err(_) => Ok("No logs available yet.".to_string())
        }
    }

    /// Clear cron job logs
    pub async fn clear_cron_logs(pool: &MySqlPool, job_id: &str, user_id: &str) -> ApiResult<()> {
        let job = sqlx::query_as::<_, CronJob>("SELECT * FROM cron_jobs WHERE id = ?").bind(job_id).fetch_optional(pool).await?.ok_or(ApiError::NotFound("Cron".to_string()))?;
        if job.user_id != user_id { return Err(ApiError::Forbidden); }

        let system_username = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let log_file = format!("/home/{}/cron_logs/{}.log", system_username, job_id);
        
        let _ = fs::write(&log_file, "");
        
        // Ensure ownership
        let _ = Command::new("chown").arg(&system_username).arg(&log_file).output();

        Ok(())
    }

    // ==========================================
    // BACKUP OPERATIONS (REAL)
    // ==========================================

    pub async fn create_backup(
        pool: &MySqlPool,
        user_id: &str,
        username: &str,
        request: CreateBackupRequest,
    ) -> ApiResult<SystemBackup> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;
        
        // Import DatabaseService here or at top level. 
        // Rust allows imports inside functions but better at top. 
        // I will assume DatabaseService is available via crate::services::DatabaseService
        
        let backup_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let system_username = username.to_string();
        let backup_filename = format!("backup_{}_{}_{}.tar.gz", username, request.backup_type, now.format("%Y%m%d%H%M%S"));
        let backup_dir = format!("/home/{}/backups", system_username);
        let backup_path = format!("{}/{}", backup_dir, backup_filename);

        // Ensure backup dir exists
        fs::create_dir_all(&backup_dir).map_err(|e| ApiError::InternalError(format!("Backup dir error: {}", e)))?;
        
        // Ensure ownership of backup dir
        let _ = Command::new("chown").arg("-R").arg(&system_username).arg(&backup_dir).output();

        // Prepare for Database Backup
        let mut temp_sql_dir = None;
        if request.backup_type == "database" || request.backup_type == "full" {
            let sql_dir = format!("{}/temp_sql_{}", backup_dir, backup_id);
            fs::create_dir_all(&sql_dir).map_err(|e| ApiError::InternalError(format!("Failed to create temp sql dir: {}", e)))?;
            temp_sql_dir = Some(sql_dir.clone());

            // Get user databases
            let databases = crate::services::DatabaseService::get_user_databases(pool, user_id).await?;
            
            // Get DB Credentials from CONFIG
            // Format: mysql://user:pass@host:port/db
            let db_url = &crate::config::CONFIG.database_url;
            // Simple parsing (naive)
            let parts: Vec<&str> = db_url.split("://").collect();
            if parts.len() < 2 { return Err(ApiError::InternalError("Invalid DB URL format".to_string())); }
            let auth_part = parts[1].split('@').next().unwrap_or("");
            let auth_parts: Vec<&str> = auth_part.split(':').collect();
            let db_user = auth_parts.get(0).unwrap_or(&"root");
            let db_pass = auth_parts.get(1).unwrap_or(&"");

            for db in databases {
                let dump_file = format!("{}/{}.sql", sql_dir, db.db_name);
                // mysqldump -u user -p'pass' db_name > dump_file
                // Note: passing password in command line is insecure (visible in ps). 
                // Better to use defaults-extra-file, but for now we stick to simple Command.
                // We will use MYSQL_PWD env var for slightly better security than CLI arg
                
                let output = Command::new("mysqldump")
                    .env("MYSQL_PWD", db_pass)
                    .arg("-u")
                    .arg(db_user)
                    .arg(&db.db_name)
                    .output(); // Capture output to avoid dumping to stdout if > file fails

                match output {
                    Ok(out) => {
                         if out.status.success() {
                             fs::write(&dump_file, out.stdout).map_err(|e| ApiError::InternalError(format!("Failed to write SQL dump: {}", e)))?;
                         } else {
                             tracing::error!("Backup DB {} failed: {}", db.db_name, String::from_utf8_lossy(&out.stderr));
                             // Continue with other DBs or error? Let's log and continue partial backup.
                         }
                    },
                    Err(e) => tracing::error!("Failed to run mysqldump for {}: {}", db.db_name, e),
                }
            }
        }

        // 1. Execute Backup Command (Tar)
        let output = match request.backup_type.as_str() {
            "homedir" => {
                // tar -czf /path/backup.tar.gz -C /home/user public_html
                Command::new("tar")
                    .arg("-czf")
                    .arg(&backup_path)
                    .arg("-C")
                    .arg(format!("/home/{}", system_username))
                    .arg("public_html")
                    .output()?
            },
            "database" => {
                // tar -czf /path/backup.tar.gz -C /home/user/backups/temp... .
                if let Some(ref sql_dir) = temp_sql_dir {
                     Command::new("tar")
                        .arg("-czf")
                        .arg(&backup_path)
                        .arg("-C")
                        .arg(sql_dir)
                        .arg(".")
                        .output()?
                } else {
                    return Err(ApiError::InternalError("SQL dir missing".to_string()));
                }
            },
            "full" => {
                // tar -czf ... -C /home/user public_html -C /temp_sql . 
                // Tar multiple -C is order dependent. 
                // Easier: link temp_sql to /home/user/database_backups then tar everything?
                // Or just Tar public_html from /home/user AND sql files from absolute path?
                // Tar absolute paths strips leading / by default.
                // Let's try: tar -czf backup.tar.gz -C /home/user public_html -C /path/to/temp_sql .
                // Result: public_html/ and ./sql_files.sql
                
                if let Some(ref sql_dir) = temp_sql_dir {
                    Command::new("tar")
                        .arg("-czf")
                        .arg(&backup_path)
                        .arg("-C")
                        .arg(format!("/home/{}", system_username))
                        .arg("public_html")
                        .arg("-C")
                        .arg(sql_dir)
                        .arg(".") // puts sql files at root of tar alongside public_html
                        .output()?
                } else {
                     return Err(ApiError::InternalError("SQL dir missing".to_string()));
                }
            },
            _ => return Err(ApiError::ValidationError("Invalid backup type".to_string())),
        };

        // Cleanup temp SQL dir
        if let Some(sql_dir) = temp_sql_dir {
            let _ = fs::remove_dir_all(sql_dir);
        }

        if !output.status.success() {
             return Err(ApiError::InternalError(format!("Backup tar failed: {}", String::from_utf8_lossy(&output.stderr))));
        }
        
        // Ensure ownership of the backup file so user can download/delete if they have shell access (optional but good)
        let _ = Command::new("chown").arg(&system_username).arg(&backup_path).output();

        // 2. Get File Size
        let file_metadata = fs::metadata(&backup_path).map_err(|_| ApiError::InternalError("Failed to get backup metadata".to_string()))?;
        let size_bytes = file_metadata.len() as i64;

        // 3. Save to DB
        sqlx::query(
            r#"
            INSERT INTO system_backups (id, user_id, filename, size_bytes, backup_type, status, created_at, completed_at)
            VALUES (?, ?, ?, ?, ?, 'Completed', ?, ?)
            "#,
        )
        .bind(&backup_id)
        .bind(user_id)
        .bind(&backup_filename)
        .bind(size_bytes)
        .bind(&request.backup_type)
        .bind(now)
        .bind(now) // Completed immediately for sync process
        .execute(pool)
        .await?;

        Ok(SystemBackup {
            id: backup_id,
            user_id: user_id.to_string(),
            filename: backup_filename,
            size_bytes,
            backup_type: request.backup_type,
            status: "Completed".to_string(),
            created_at: now,
            completed_at: Some(now),
        })
    }

    /// Get all cron jobs for user
    pub async fn get_user_cron_jobs(pool: &MySqlPool, user_id: &str) -> ApiResult<Vec<CronJob>> {
        let jobs = sqlx::query_as::<_, CronJob>(
            "SELECT * FROM cron_jobs WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(jobs)
    }

    /// Get all backups for user
    pub async fn get_user_backups(pool: &MySqlPool, user_id: &str) -> ApiResult<Vec<SystemBackup>> {
        let backups = sqlx::query_as::<_, SystemBackup>(
            "SELECT * FROM system_backups WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(backups)
    }

    /// Delete backup
    pub async fn delete_backup(pool: &MySqlPool, id: &str, user_id: &str) -> ApiResult<()> {
        let backup = sqlx::query_as::<_, SystemBackup>(
            "SELECT * FROM system_backups WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Backup".to_string()))?;

        if backup.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // 1. Delete file
        let system_username = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|_| ApiError::InternalError("Failed to get username for backup deletion".to_string()))?;
        let backup_path = format!("/home/{}/backups/{}", system_username, backup.filename);
        let _ = fs::remove_file(backup_path);

        // 2. Delete from DB
        sqlx::query("DELETE FROM system_backups WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_backup_path(pool: &MySqlPool, id: &str, user_id: &str) -> ApiResult<String> {
        let backup = sqlx::query_as::<_, SystemBackup>(
            "SELECT * FROM system_backups WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Backup".to_string()))?;

        if backup.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let system_username = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
            
        Ok(format!("/home/{}/backups/{}", system_username, backup.filename))
    }

    // ==========================================
    // SERVICE STATUS OPERATIONS (REAL)
    // ==========================================

    pub async fn get_service_status() -> ApiResult<Vec<ServiceStatus>> {
        let services = vec!["nginx", "mysql", "cron"];
        let mut statuses = Vec::new();

        for service in services {
            // systemctl is-active service_name
            let output = Command::new("systemctl")
                .arg("is-active")
                .arg(service)
                .output()?;
            
            let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
            // Untuk detail uptime/memory, butuh parsing output `systemctl status` yang kompleks
            // Untuk MVP "Real", status active/inactive sudah cukup valid
            statuses.push(ServiceStatus {
                name: service.to_string(),
                status: status_str,
                uptime: "Check via CLI".to_string(), // Placeholder complexity
                memory_usage: "Check via CLI".to_string(),
            });
        }

        Ok(statuses)
    }

    /// Control service (start/stop/restart)
    pub async fn control_service(service_name: &str, action: &str) -> ApiResult<ServiceStatus> {
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

        // Update DB first
        sqlx::query("UPDATE users SET php_version = ?, updated_at = ? WHERE id = ?")
            .bind(version)
            .bind(Utc::now())
            .bind(user_id)
            .execute(pool)
            .await?;

        // Ensure PHP-FPM pool for this user/version
        let system_username = sqlx::query_scalar::<_, String>(
            "SELECT username FROM users WHERE id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        let system_username = format!("user_{}", system_username);

        PhpPoolService::ensure_user_pool(version, &system_username)?;

        // Update vhost configs to use new PHP socket
        WebServerServiceReal::update_user_vhosts_php(pool, user_id, version).await?;

        Ok(version.to_string())
    }

    /// Get installed PHP versions
    pub async fn get_php_versions() -> ApiResult<Vec<String>> {
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

        Ok(logs)
    }

    /// Clear error logs
    pub async fn clear_error_logs() -> ApiResult<()> {
        let log_files = vec![
            "/var/log/php7.4-fpm.log",
            "/var/log/php8.3-fpm.log",
            "/var/log/php8.2-fpm.log",
            "/var/log/php8.1-fpm.log",
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

    // ==========================================
    // DNS TRACKER OPERATIONS
    // ==========================================

    pub async fn dns_lookup(domain: &str, record_type: &str) -> ApiResult<String> {
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

    pub async fn trace_route(domain: &str) -> ApiResult<Vec<String>> {
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

    pub async fn get_resource_usage() -> ApiResult<ResourceUsage> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        
        let cpu_usage = if sys.cpus().is_empty() {
            0.0
        } else {
            sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
        };
        
        let memory_usage = sys.used_memory();
        
        let mut total_disk = 0;
        let disks = sysinfo::Disks::new_with_refreshed_list();
        for disk in &disks {
            total_disk += disk.total_space() - disk.available_space();
        }

        let mut processes = Vec::new();
        // Get top 10 processes by CPU usage
        let mut process_list: Vec<_> = sys.processes().iter().collect();
        process_list.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

        for (pid, process) in process_list.iter().take(10) {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu: process.cpu_usage(),
                memory: process.memory(),
            });
        }

        Ok(ResourceUsage {
            cpu: cpu_usage,
            memory: memory_usage,
            disk: total_disk,
            processes,
        })
    }
}
