//! # Security Service
//!
//! Business logic untuk Security & Monitoring.
//! Menangani IP Blocking, SSH Key management, dan Resource Monitoring (Simulasi).

use chrono::{TimeZone, Utc};
use rand::Rng; // For simulation
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    AccessLogEntry, BlockedIp, CreateBlockedIpRequest, CreateSshKeyRequest, ResourceUsageStats,
    SshAccess, SshAccessResponse,
};

/// Service untuk security & monitoring operations
pub struct SecurityService;

impl SecurityService {
    // ==========================================
    // IP BLOCKING OPERATIONS
    // ==========================================

    /// Get details of all blocked IPs
    pub async fn get_blocked_ips(pool: &MySqlPool) -> ApiResult<Vec<BlockedIp>> {
        let ips = sqlx::query_as::<_, BlockedIp>(
            "SELECT * FROM blocked_ips ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(ips)
    }

    /// Block IP Address
    pub async fn block_ip(pool: &MySqlPool, request: CreateBlockedIpRequest) -> ApiResult<BlockedIp> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Check if exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM blocked_ips WHERE ip_address = ?",
        )
        .bind(&request.ip_address)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("IP Address".to_string()));
        }

        let ip_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO blocked_ips (id, ip_address, reason, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&ip_id)
        .bind(&request.ip_address)
        .bind(&request.reason)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // TODO: Call Fail2Ban or iptables command here
        tracing::info!("IP Address blocked: {}", request.ip_address);

        Ok(BlockedIp {
            id: ip_id,
            ip_address: request.ip_address,
            reason: request.reason,
            created_at: now,
            updated_at: now,
        })
    }

    /// Unblock IP Address
    pub async fn unblock_ip(pool: &MySqlPool, ip_id: &str) -> ApiResult<()> {
        let ip = sqlx::query_as::<_, BlockedIp>(
            "SELECT * FROM blocked_ips WHERE id = ?",
        )
        .bind(ip_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Blocked IP".to_string()))?;

        sqlx::query("DELETE FROM blocked_ips WHERE id = ?")
            .bind(ip_id)
            .execute(pool)
            .await?;

        // TODO: Remove from Fail2Ban or iptables
        tracing::info!("IP Address unblocked: {}", ip.ip_address);

        Ok(())
    }

    // ==========================================
    // SSH ACCESS OPERATIONS
    // ==========================================

    /// Get all SSH Keys for user
    pub async fn get_user_ssh_keys(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<SshAccessResponse>> {
        let keys = sqlx::query_as::<_, SshAccess>(
            "SELECT * FROM ssh_keys WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let responses = keys
            .into_iter()
            .map(|k| SshAccessResponse {
                id: k.id,
                label: k.label,
                fingerprint: "SHA256:SimulatedFingerprint...".to_string(), // TODO: Calculate real fingerprint
                created_at: k.created_at,
            })
            .collect();

        Ok(responses)
    }

    /// Add SSH Key
    pub async fn add_ssh_key(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateSshKeyRequest,
    ) -> ApiResult<SshAccessResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let key_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO ssh_keys (id, user_id, public_key, label, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&key_id)
        .bind(user_id)
        .bind(&request.public_key)
        .bind(&request.label)
        .bind(now)
        .execute(pool)
        .await?;

        // TODO: Append to ~/.ssh/authorized_keys
        tracing::info!("SSH Key added for user: {}", user_id);

        Ok(SshAccessResponse {
            id: key_id,
            label: request.label,
            fingerprint: "SHA256:SimulatedFingerprint...".to_string(),
            created_at: now,
        })
    }

    /// Delete SSH Key
    pub async fn delete_ssh_key(
        pool: &MySqlPool,
        key_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let key = sqlx::query_as::<_, SshAccess>(
            "SELECT * FROM ssh_keys WHERE id = ?",
        )
        .bind(key_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("SSH Key".to_string()))?;

        if key.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        sqlx::query("DELETE FROM ssh_keys WHERE id = ?")
            .bind(key_id)
            .execute(pool)
            .await?;

        // TODO: Remove from ~/.ssh/authorized_keys
        tracing::info!("SSH Key deleted: {}", key.label);

        Ok(())
    }

    // ==========================================
    // MONITORING OPERATIONS (SIMULATION)
    // ==========================================

    /// Get Resource Usage Stats (Simulasi)
    pub async fn get_resource_usage(user_id: &str) -> ApiResult<ResourceUsageStats> {
        // Simulasi data random untuk CPU usage
        let mut rng = rand::thread_rng();
        let cpu_usage_percent = rng.gen_range(5.0..45.0); // 5% to 45% usage

        // Simulasi RAM usage
        let ram_usage_bytes = rng.gen_range(100 * 1024 * 1024..400 * 1024 * 1024); // 100MB - 400MB
        let ram_limit_bytes = 1024 * 1024 * 1024; // 1GB Limit

        // Simulasi Disk usage
        let disk_usage_bytes = rng.gen_range(500 * 1024 * 1024..2 * 1024 * 1024 * 1024); // 500MB - 2GB
        let disk_limit_bytes = 10 * 1024 * 1024 * 1024; // 10GB Limit

        Ok(ResourceUsageStats {
            user_id: user_id.to_string(),
            cpu_usage_percent,
            ram_usage_bytes,
            ram_limit_bytes,
            disk_usage_bytes,
            disk_limit_bytes,
            timestamp: Utc::now(),
        })
    }

    /// Get Access Logs (Simulasi)
    pub async fn get_access_logs(user_id: &str) -> ApiResult<Vec<AccessLogEntry>> {
        // Simulasi dummy logs
        let mut logs = Vec::new();
        let now = Utc::now();
        let methods = vec!["GET", "POST", "PUT"];
        let paths = vec!["/index.php", "/wp-admin", "/api/v1/data", "/images/logo.png"];
        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Safari/605.1.15",
            "Googlebot/2.1 (+http://www.google.com/bot.html)",
        ];

        let mut rng = rand::thread_rng();

        for i in 0..10 {
            let method = methods[rng.gen_range(0..methods.len())].to_string();
            let path = paths[rng.gen_range(0..paths.len())].to_string();
            let status_code = if rng.gen_bool(0.9) { 200 } else { 404 };
            let user_agent = user_agents[rng.gen_range(0..user_agents.len())].to_string();
            
            logs.push(AccessLogEntry {
                ip_address: format!("192.168.1.{}", rng.gen_range(10..200)),
                method,
                path,
                status_code,
                user_agent,
                timestamp: now - chrono::Duration::minutes(i * 5),
            });
        }

        Ok(logs)
    }
}
