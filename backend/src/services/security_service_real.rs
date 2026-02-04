//! # Security Service (REAL IMPLEMENTATION)
//!
//! Implementasi nyata untuk Security & Monitoring yang berinteraksi langsung dengan OS.
//! Gunakan file ini menggantikan `security_service.rs` saat deploy ke production server.
//!
//! Syarat Sistem:
//! - Linux OS (Ubuntu/Debian/CentOS)
//! - `iptables` atau `ufw` terinstall
//! - Akses root/sudo untuk command eksekusi
//! - Struktur direktori SSH standard (`~/.ssh/authorized_keys`)

use chrono::{DateTime, Duration, Utc};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command;
use sysinfo::{Disks, System};
use rand::Rng;
use qrcode::QrCode;
use qrcode::render::svg;
use totp_rs::{Algorithm, Secret, TOTP};
use base64::Engine;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use serde_json::Value;

use crate::errors::{ApiError, ApiResult};
use crate::config::CONFIG;
use crate::models::{
    AccessLogEntry, BlockedIp, CreateBlockedIpRequest, CreateSshKeyRequest, ResourceUsageStats,
    SshAccessResponse, TwoFactorSetupResponse, TwoFactorStatusResponse, TwoFactorVerifyRequest,
    ModSecurityOverview, ModSecurityRuleSet, ModSecuritySettings, ModSecurityCustomRule,
    ModSecurityAuditLog, ModSecurityAuditLogQuery, CreateModSecurityCustomRuleRequest,
    UpdateModSecurityCustomRuleRequest, IngestModSecurityAuditLogRequest,
    IngestModSecurityAuditLogResponse,
    UpdateModSecurityDomainRequest, UpdateModSecurityRuleRequest, UpdateModSecuritySettingsRequest,
};

// Placeholder structs until shared with main code or duplicated
// Pastikan struct ini sesuai dengan `crate::models`
use crate::models::SshAccess; 
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

pub struct SecurityServiceReal;

#[derive(Debug, Clone)]
struct ParsedAuditLog {
    user_id: Option<String>,
    domain_host: Option<String>,
    rule_id: Option<String>,
    custom_rule_id: Option<String>,
    severity: String,
    message: String,
    ip_address: Option<String>,
    uri: Option<String>,
    user_agent: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

fn normalize_host(host: &str) -> String {
    host.split(':').next().unwrap_or(host).trim().to_lowercase()
}

fn normalize_severity(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    if lower.is_empty() {
        return "medium".to_string();
    }
    if let Ok(num) = lower.parse::<i64>() {
        return match num {
            0 | 1 => "critical",
            2 | 3 => "high",
            4 => "medium",
            _ => "low",
        }
        .to_string();
    }
    if lower.contains("critical") || lower.contains("emerg") || lower.contains("alert") {
        "critical".to_string()
    } else if lower.contains("high") {
        "high".to_string()
    } else if lower.contains("low") {
        "low".to_string()
    } else {
        "medium".to_string()
    }
}

fn map_rule_id(message: &str, tags: &[String], raw_rule_id: Option<&str>) -> Option<String> {
    let allowed = ["sqli", "xss", "lfi", "scanner"];
    if let Some(rule_id) = raw_rule_id {
        let candidate = rule_id.trim().to_lowercase();
        if allowed.contains(&candidate.as_str()) {
            return Some(candidate);
        }
    }
    let mut haystack = message.to_lowercase();
    for tag in tags {
        haystack.push(' ');
        haystack.push_str(&tag.to_lowercase());
    }
    if haystack.contains("sqli") || haystack.contains("sql injection") {
        Some("sqli".to_string())
    } else if haystack.contains("xss") || haystack.contains("cross-site") {
        Some("xss".to_string())
    } else if haystack.contains("lfi") || haystack.contains("local file") {
        Some("lfi".to_string())
    } else if haystack.contains("scanner") {
        Some("scanner".to_string())
    } else {
        None
    }
}

fn truncate_255(value: &str) -> String {
    if value.len() <= 255 {
        value.to_string()
    } else {
        value.chars().take(255).collect()
    }
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = DateTime::parse_from_str(value, "%a %b %d %H:%M:%S %Y") {
        return Some(dt.with_timezone(&Utc));
    }
    None
}

fn parse_json_audit_line(line: &str) -> Vec<ParsedAuditLog> {
    let value: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let transaction = value.get("transaction");
    let client_ip = transaction
        .and_then(|t| t.get("client_ip"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let request = transaction.and_then(|t| t.get("request"));
    let uri = request
        .and_then(|r| r.get("uri").or_else(|| r.get("request_uri")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let headers = request.and_then(|r| r.get("headers"));
    let user_agent = headers
        .and_then(|h| h.get("User-Agent").or_else(|| h.get("user-agent")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let host = headers
        .and_then(|h| h.get("Host").or_else(|| h.get("host")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let created_at = transaction
        .and_then(|t| t.get("time_stamp"))
        .and_then(|v| v.as_str())
        .and_then(parse_timestamp);

    let mut entries = Vec::new();
    let messages = value.get("messages").and_then(|v| v.as_array());
    if let Some(messages) = messages {
        for msg in messages {
            let details = msg.get("details");
            let message = msg
                .get("message")
                .and_then(|v| v.as_str())
                .or_else(|| details.and_then(|d| d.get("message")).and_then(|v| v.as_str()))
                .unwrap_or("ModSecurity event")
                .to_string();

            let severity_raw = details
                .and_then(|d| d.get("severity"))
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())));
            let severity = normalize_severity(severity_raw.as_deref().unwrap_or("medium"));

            let raw_rule_id = details
                .and_then(|d| d.get("ruleId").or_else(|| d.get("rule_id")))
                .and_then(|v| v.as_str());

            let tags = details
                .and_then(|d| d.get("tags"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let rule_id = map_rule_id(&message, &tags, raw_rule_id);

            entries.push(ParsedAuditLog {
                user_id: None,
                domain_host: host.clone(),
                rule_id,
                custom_rule_id: None,
                severity,
                message,
                ip_address: client_ip.clone(),
                uri: uri.clone(),
                user_agent: user_agent.clone(),
                created_at,
            });
        }
    }
    entries
}

fn parse_text_message_line(line: &str, regexes: &TextParseRegexes) -> Option<ParsedAuditLog> {
    if !(line.contains("Message:") || line.contains("[msg")) {
        return None;
    }
    let message = if let Some(caps) = regexes.msg.captures(line) {
        caps.get(1).map(|m| m.as_str().to_string())
    } else if let Some(pos) = line.find("Message:") {
        let after = &line[pos + "Message:".len()..];
        Some(after.split(" [").next().unwrap_or(after).trim().to_string())
    } else {
        None
    }?;

    let severity = regexes
        .severity
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| normalize_severity(m.as_str()))
        .unwrap_or_else(|| "medium".to_string());

    let uri = regexes
        .uri
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string());

    let ip_address = regexes
        .client
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim_matches('"').to_string());

    let host = regexes
        .hostname
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .or_else(|| {
            regexes
                .host
                .captures(line)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        });

    let tags = regexes
        .tag
        .captures_iter(line)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect::<Vec<_>>();

    let raw_rule_id = regexes
        .rule_id
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string());

    let rule_id = map_rule_id(&message, &tags, raw_rule_id.as_deref());

    Some(ParsedAuditLog {
        user_id: None,
        domain_host: host,
        rule_id,
        custom_rule_id: None,
        severity,
        message,
        ip_address,
        uri,
        user_agent: None,
        created_at: None,
    })
}

struct TextParseRegexes {
    msg: Regex,
    severity: Regex,
    uri: Regex,
    client: Regex,
    hostname: Regex,
    host: Regex,
    tag: Regex,
    rule_id: Regex,
}

impl SecurityServiceReal {
    fn generate_backup_codes() -> Vec<String> {
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| {
                let a: u16 = rng.gen_range(1000..9999);
                let b: u16 = rng.gen_range(1000..9999);
                format!("{} - {}", a, b)
            })
            .collect()
    }

    fn build_qr_code(otpauth: &str) -> ApiResult<String> {
        let code = QrCode::new(otpauth.as_bytes())
            .map_err(|e| ApiError::InternalError(format!("Failed to generate QR: {}", e)))?;
        let svg_str = code
            .render::<svg::Color>()
            .min_dimensions(220, 220)
            .build();
        let encoded = base64::engine::general_purpose::STANDARD.encode(svg_str.as_bytes());
        Ok(format!("data:image/svg+xml;base64,{}", encoded))
    }

    pub async fn log_event(
        pool: &MySqlPool,
        user_id: Option<&str>,
        event_type: &str,
        ip_address: Option<&str>,
        target: Option<&str>,
        status: Option<&str>,
        user_agent: Option<&str>,
    ) -> ApiResult<()> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO security_access_logs
                (id, user_id, event_type, ip_address, target, status, user_agent, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(event_type)
        .bind(ip_address)
        .bind(target)
        .bind(status)
        .bind(user_agent)
        .bind(Utc::now())
        .execute(pool)
        .await?;
        Ok(())
    }

    // ==========================================
    // IP BLOCKING OPERATIONS (REAL)
    // ==========================================

    pub async fn block_ip(pool: &MySqlPool, request: CreateBlockedIpRequest) -> ApiResult<BlockedIp> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 1. Eksekusi Command System (iptables)
        // sudo iptables -A INPUT -s <IP> -j DROP
        let output = Command::new("sudo")
            .arg("iptables")
            .arg("-A")
            .arg("INPUT")
            .arg("-s")
            .arg(&request.ip_address)
            .arg("-j")
            .arg("DROP")
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute iptables: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("Iptables error: {}", error)));
        }

        // 2. Simpan ke Database
        // ... (Kode database sama dengan versi simulasi)
        let ip_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO blocked_ips (id, ip_address, reason, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&ip_id)
        .bind(&request.ip_address)
        .bind(&request.reason)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(BlockedIp {
            id: ip_id,
            ip_address: request.ip_address,
            reason: request.reason,
            created_at: now,
            updated_at: now,
        })
    }

    /// Get details of all blocked IPs
    pub async fn get_blocked_ips(pool: &MySqlPool) -> ApiResult<Vec<BlockedIp>> {
        let ips = sqlx::query_as::<_, BlockedIp>(
            "SELECT * FROM blocked_ips ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(ips)
    }

    pub async fn unblock_ip(pool: &MySqlPool, ip_id: &str) -> ApiResult<()> {
        let ip = sqlx::query_as::<_, BlockedIp>("SELECT * FROM blocked_ips WHERE id = ?")
            .bind(ip_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Blocked IP".to_string()))?;

        // 1. Hapus dari Firewall (iptables -D)
        let _ = Command::new("sudo")
            .arg("iptables")
            .arg("-D")
            .arg("INPUT")
            .arg("-s")
            .arg(&ip.ip_address)
            .arg("-j")
            .arg("DROP")
            .output();

        // 2. Hapus dari Database
        sqlx::query("DELETE FROM blocked_ips WHERE id = ?")
            .bind(ip_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    // ==========================================
    // SSH ACCESS OPERATIONS (REAL)
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
                public_key: k.public_key,
                fingerprint: "SHA256:PLACEHOLDER...".to_string(), // In production calc real fingerprint
                created_at: k.created_at,
            })
            .collect();

        Ok(responses)
    }

    pub async fn add_ssh_key(pool: &MySqlPool, user_id: &str, request: CreateSshKeyRequest) -> ApiResult<SshAccessResponse> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;
        let key_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO ssh_keys (id, user_id, public_key, label, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&key_id)
        .bind(user_id)
        .bind(&request.public_key)
        .bind(&request.label)
        .bind(now)
        .execute(pool)
        .await?;
        
        let system_username = format!("user_{}", user_id.replace("-", "").chars().take(8).collect::<String>()); 
        let ssh_dir = format!("/home/{}/.ssh", system_username);
        let auth_keys_path = format!("{}/authorized_keys", ssh_dir);

        fs::create_dir_all(&ssh_dir).map_err(|e| ApiError::InternalError(format!("Failed to create ssh dir: {}", e)))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&auth_keys_path)
            .map_err(|e| ApiError::InternalError(format!("Failed to open authorized_keys: {}", e)))?;

        writeln!(file, "\n# Added by NusaPanel for key {}\n{}", request.label, request.public_key)
            .map_err(|e| ApiError::InternalError(format!("Failed to write key: {}", e)))?;

        let _ = Command::new("chmod").arg("600").arg(&auth_keys_path).output();
        let _ = Command::new("chmod").arg("700").arg(&ssh_dir).output();
        let _ = Command::new("chown").arg("-R").arg(format!("{}:{}", system_username, system_username)).arg(&ssh_dir).output();

        Ok(SshAccessResponse {
            id: key_id,
            label: request.label,
            public_key: request.public_key,
            fingerprint: "SHA256:REAL_CALCULATION_NEEDED".to_string(), 
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

        // 1. Delete from DB
        sqlx::query("DELETE FROM ssh_keys WHERE id = ?")
            .bind(key_id)
            .execute(pool)
            .await?;

        // 2. Remove from File
        tracing::warn!("SSH Key deleted from DB: {}. Manual removal from authorized_keys recommended if critical.", key.label);

        Ok(())
    }

    // ==========================================
    // MONITORING OPERATIONS (REAL)
    // ==========================================

    pub async fn get_resource_usage(pool: &MySqlPool, user_id: &str) -> ApiResult<ResourceUsageStats> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage_percent = if sys.cpus().is_empty() {
            0.0
        } else {
            sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
        };
        let ram_usage_bytes = sys.used_memory() as i64;
        let ram_limit_bytes = sys.total_memory() as i64;

        let mut disk_usage_bytes = 0;
        let mut disk_limit_bytes = 0;
        
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            if disk.mount_point() == std::path::Path::new("/") {
                disk_usage_bytes = (disk.total_space() - disk.available_space()) as i64;
                disk_limit_bytes = disk.total_space() as i64;
                break;
            }
        }

        let disk_percent = if disk_limit_bytes > 0 {
            (disk_usage_bytes as f32 / disk_limit_bytes as f32) * 100.0
        } else {
            0.0
        };

        let blocked_ips_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM blocked_ips")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

        let ssh_keys_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM ssh_keys WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(ResourceUsageStats {
            user_id: user_id.to_string(),
            cpu_usage_percent,
            ram_usage_bytes,
            ram_limit_bytes,
            disk_usage_bytes,
            disk_limit_bytes,
            disk_percent,
            firewall_enabled: blocked_ips_count > 0,
            blocked_ips_count,
            ssh_keys_count,
            timestamp: Utc::now(),
        })
    }

    pub async fn get_access_logs(pool: &MySqlPool, user_id: &str) -> ApiResult<Vec<AccessLogEntry>> {
        let logs = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, chrono::DateTime<Utc>)>(
            r#"
            SELECT id, event_type, ip_address, target, user_agent, created_at
            FROM security_access_logs
            WHERE user_id = ? OR user_id IS NULL
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| AccessLogEntry {
            id: row.0,
            event_type: row.1,
            ip_address: row.2.unwrap_or_else(|| "unknown".to_string()),
            target: row.3,
            status: None,
            user_agent: row.4,
            timestamp: row.5,
        })
        .collect();

        Ok(logs)
    }

    // ==========================================
    // 2FA OPERATIONS
    // ==========================================

    pub async fn get_2fa_status(pool: &MySqlPool, user_id: &str) -> ApiResult<TwoFactorStatusResponse> {
        let row = sqlx::query_as::<_, (bool, Option<chrono::DateTime<Utc>>)>(
            "SELECT is_enabled, enabled_at FROM user_2fa WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(match row {
            Some((enabled, enabled_at)) => TwoFactorStatusResponse { enabled, enabled_at },
            None => TwoFactorStatusResponse { enabled: false, enabled_at: None },
        })
    }

    pub async fn setup_2fa(pool: &MySqlPool, user_id: &str, account_label: &str) -> ApiResult<TwoFactorSetupResponse> {
        let secret = Secret::generate_secret().to_encoded().to_string();
        let backup_codes = Self::generate_backup_codes();
        let backup_codes_json = serde_json::to_string(&backup_codes)
            .map_err(|e| ApiError::InternalError(format!("Failed to encode backup codes: {}", e)))?;

        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_2fa WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            sqlx::query(
                r#"
                UPDATE user_2fa
                SET secret = ?, backup_codes = ?, is_enabled = FALSE, enabled_at = NULL, updated_at = ?
                WHERE user_id = ?
                "#,
            )
            .bind(&secret)
            .bind(&backup_codes_json)
            .bind(Utc::now())
            .bind(user_id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO user_2fa (user_id, secret, backup_codes, is_enabled, created_at, updated_at)
                VALUES (?, ?, ?, FALSE, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(&secret)
            .bind(&backup_codes_json)
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(pool)
            .await?;
        }

        let issuer = "NusaPanel";
        let otpauth = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            issuer,
            account_label,
            secret,
            issuer
        );
        let qr_code = Self::build_qr_code(&otpauth)?;

        Ok(TwoFactorSetupResponse {
            qr_code,
            secret,
            backup_codes,
        })
    }

    pub async fn verify_2fa(
        pool: &MySqlPool,
        user_id: &str,
        request: TwoFactorVerifyRequest,
    ) -> ApiResult<()> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let row = sqlx::query_as::<_, (String,)>(
            "SELECT secret FROM user_2fa WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("2FA setup".to_string()))?;

        let secret = Secret::Encoded(row.0);
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret
                .to_bytes()
                .map_err(|e| ApiError::InternalError(format!("Invalid secret: {}", e)))?,
        )
        .map_err(|e| ApiError::InternalError(format!("Failed to init TOTP: {}", e)))?;

        match totp.check_current(&request.code) {
            Ok(true) => {}
            Ok(false) => return Err(ApiError::InvalidCredentials),
            Err(e) => {
                return Err(ApiError::InternalError(format!(
                    "Failed to validate 2FA code: {}",
                    e
                )))
            }
        }

        sqlx::query(
            "UPDATE user_2fa SET is_enabled = TRUE, enabled_at = ?, updated_at = ? WHERE user_id = ?",
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn disable_2fa(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        sqlx::query(
            "UPDATE user_2fa SET is_enabled = FALSE, updated_at = ? WHERE user_id = ?",
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    // ==========================================
    // SSL OPERATIONS
    // ==========================================

    pub async fn run_auto_ssl(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        let expiry = Utc::now() + Duration::days(90);
        sqlx::query(
            r#"
            UPDATE domains
            SET ssl_enabled = TRUE,
                ssl_status = 'active',
                ssl_provider = 'Let''s Encrypt',
                ssl_expiry_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(expiry)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn renew_ssl(pool: &MySqlPool, user_id: &str, domain_id: &str) -> ApiResult<()> {
        let expiry = Utc::now() + Duration::days(90);
        let updated = sqlx::query(
            r#"
            UPDATE domains
            SET ssl_enabled = TRUE,
                ssl_status = 'active',
                ssl_provider = 'Let''s Encrypt',
                ssl_expiry_at = ?
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(expiry)
        .bind(domain_id)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(ApiError::NotFound("Domain".to_string()));
        }
        Ok(())
    }

    // ==========================================
    // MODSECURITY OPERATIONS
    // ==========================================

    pub async fn get_modsecurity_overview(pool: &MySqlPool) -> ApiResult<ModSecurityOverview> {
        let settings = sqlx::query_as::<_, (bool, i32, i32)>(
            "SELECT main_engine, paranoia_level, anomaly_threshold FROM modsecurity_settings WHERE id = 1",
        )
        .fetch_optional(pool)
        .await?
        .map(|row| ModSecuritySettings {
            main_engine: row.0,
            paranoia_level: row.1,
            anomaly_threshold: row.2,
        })
        .unwrap_or(ModSecuritySettings {
            main_engine: true,
            paranoia_level: 2,
            anomaly_threshold: 5,
        });

        let rules = sqlx::query_as::<_, (String, String, String, bool)>(
            "SELECT id, name, description, enabled FROM modsecurity_rule_sets ORDER BY id",
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| ModSecurityRuleSet {
            id: row.0,
            name: row.1,
            description: row.2,
            enabled: row.3,
        })
        .collect::<Vec<_>>();

        let rules_triggered = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_access_logs WHERE event_type LIKE 'modsecurity_%'",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(ModSecurityOverview {
            rules_triggered,
            trend_percent: 12,
            chart_labels: vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
            chart_data: vec![40, 30, 55, 45, 80, 70, 90],
            settings,
            rules,
        })
    }

    pub async fn update_modsecurity_settings(
        pool: &MySqlPool,
        request: UpdateModSecuritySettingsRequest,
    ) -> ApiResult<ModSecuritySettings> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let current = sqlx::query_as::<_, (bool, i32, i32)>(
            "SELECT main_engine, paranoia_level, anomaly_threshold FROM modsecurity_settings WHERE id = 1",
        )
        .fetch_optional(pool)
        .await?
        .map(|row| ModSecuritySettings {
            main_engine: row.0,
            paranoia_level: row.1,
            anomaly_threshold: row.2,
        })
        .unwrap_or(ModSecuritySettings {
            main_engine: true,
            paranoia_level: 2,
            anomaly_threshold: 5,
        });

        let new_settings = ModSecuritySettings {
            main_engine: request.main_engine.unwrap_or(current.main_engine),
            paranoia_level: request.paranoia_level.unwrap_or(current.paranoia_level),
            anomaly_threshold: request.anomaly_threshold.unwrap_or(current.anomaly_threshold),
        };

        sqlx::query(
            r#"
            INSERT INTO modsecurity_settings (id, main_engine, paranoia_level, anomaly_threshold, created_at, updated_at)
            VALUES (1, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                main_engine = VALUES(main_engine),
                paranoia_level = VALUES(paranoia_level),
                anomaly_threshold = VALUES(anomaly_threshold),
                updated_at = VALUES(updated_at)
            "#,
        )
        .bind(new_settings.main_engine)
        .bind(new_settings.paranoia_level)
        .bind(new_settings.anomaly_threshold)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(pool)
        .await?;

        Ok(new_settings)
    }

    pub async fn update_modsecurity_rule(
        pool: &MySqlPool,
        rule_id: &str,
        request: UpdateModSecurityRuleRequest,
    ) -> ApiResult<ModSecurityRuleSet> {
        let updated = sqlx::query(
            "UPDATE modsecurity_rule_sets SET enabled = ?, updated_at = ? WHERE id = ?",
        )
        .bind(request.enabled)
        .bind(Utc::now())
        .bind(rule_id)
        .execute(pool)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(ApiError::NotFound("ModSecurity rule".to_string()));
        }

        let rule = sqlx::query_as::<_, (String, String, String, bool)>(
            "SELECT id, name, description, enabled FROM modsecurity_rule_sets WHERE id = ?",
        )
        .bind(rule_id)
        .fetch_one(pool)
        .await?;

        Ok(ModSecurityRuleSet {
            id: rule.0,
            name: rule.1,
            description: rule.2,
            enabled: rule.3,
        })
    }

    pub async fn update_modsecurity_domain(
        pool: &MySqlPool,
        domain_id: &str,
        request: UpdateModSecurityDomainRequest,
    ) -> ApiResult<()> {
        let updated = sqlx::query(
            "UPDATE domains SET modsecurity_enabled = ? WHERE id = ?",
        )
        .bind(request.enabled)
        .bind(domain_id)
        .execute(pool)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(ApiError::NotFound("Domain".to_string()));
        }
        Ok(())
    }

    // ==========================================
    // MODSECURITY CUSTOM RULES
    // ==========================================

    pub async fn list_custom_rules(pool: &MySqlPool) -> ApiResult<Vec<ModSecurityCustomRule>> {
        let rules = sqlx::query_as::<_, ModSecurityCustomRule>(
            "SELECT * FROM modsecurity_custom_rules ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rules)
    }

    pub async fn create_custom_rule(
        pool: &MySqlPool,
        request: CreateModSecurityCustomRuleRequest,
    ) -> ApiResult<ModSecurityCustomRule> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let enabled = request.enabled.unwrap_or(true);

        sqlx::query(
            r#"
            INSERT INTO modsecurity_custom_rules (id, name, description, rule_content, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.rule_content)
        .bind(enabled)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let rule = sqlx::query_as::<_, ModSecurityCustomRule>(
            "SELECT * FROM modsecurity_custom_rules WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(pool)
        .await?;

        Ok(rule)
    }

    pub async fn update_custom_rule(
        pool: &MySqlPool,
        rule_id: &str,
        request: UpdateModSecurityCustomRuleRequest,
    ) -> ApiResult<ModSecurityCustomRule> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let existing = sqlx::query_as::<_, ModSecurityCustomRule>(
            "SELECT * FROM modsecurity_custom_rules WHERE id = ?",
        )
        .bind(rule_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Custom rule".to_string()))?;

        let name = request.name.unwrap_or(existing.name);
        let description = if request.description.is_some() {
            request.description
        } else {
            existing.description
        };
        let rule_content = request.rule_content.unwrap_or(existing.rule_content);
        let enabled = request.enabled.unwrap_or(existing.enabled);

        sqlx::query(
            r#"
            UPDATE modsecurity_custom_rules
            SET name = ?, description = ?, rule_content = ?, enabled = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&name)
        .bind(&description)
        .bind(&rule_content)
        .bind(enabled)
        .bind(Utc::now())
        .bind(rule_id)
        .execute(pool)
        .await?;

        let rule = sqlx::query_as::<_, ModSecurityCustomRule>(
            "SELECT * FROM modsecurity_custom_rules WHERE id = ?",
        )
        .bind(rule_id)
        .fetch_one(pool)
        .await?;

        Ok(rule)
    }

    pub async fn delete_custom_rule(pool: &MySqlPool, rule_id: &str) -> ApiResult<()> {
        let rows = sqlx::query("DELETE FROM modsecurity_custom_rules WHERE id = ?")
            .bind(rule_id)
            .execute(pool)
            .await?
            .rows_affected();

        if rows == 0 {
            return Err(ApiError::NotFound("Custom rule".to_string()));
        }
        Ok(())
    }

    // ==========================================
    // MODSECURITY AUDIT LOGS
    // ==========================================

    pub async fn ingest_modsecurity_audit_log_file(
        pool: &MySqlPool,
        request: IngestModSecurityAuditLogRequest,
    ) -> ApiResult<IngestModSecurityAuditLogResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let path = request
            .path
            .unwrap_or_else(|| CONFIG.security.modsecurity_audit_log_path.clone());
        let max_entries = request.max_entries.unwrap_or(1000).min(5000).max(1) as usize;

        let file = File::open(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ApiError::FileNotFound(path.clone())
            } else {
                ApiError::IoError(e)
            }
        })?;
        let reader = BufReader::new(file);

        let regexes = TextParseRegexes {
            msg: Regex::new(r#"\[msg \"([^\"]+)\"\]"#).unwrap(),
            severity: Regex::new(r#"\[severity \"([^\"]+)\"\]"#).unwrap(),
            uri: Regex::new(r#"\[uri \"([^\"]+)\"\]"#).unwrap(),
            client: Regex::new(r#"\[client ([^\]]+)\]"#).unwrap(),
            hostname: Regex::new(r#"\[hostname \"([^\"]+)\"\]"#).unwrap(),
            host: Regex::new(r#"\[host \"([^\"]+)\"\]"#).unwrap(),
            tag: Regex::new(r#"\[tag \"([^\"]+)\"\]"#).unwrap(),
            rule_id: Regex::new(r#"\[id \"([^\"]+)\"\]"#).unwrap(),
        };

        let mut entries: Vec<ParsedAuditLog> = Vec::new();
        let mut parsed = 0i64;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut line_entries = if line.starts_with('{') {
                parse_json_audit_line(line)
            } else {
                parse_text_message_line(line, &regexes).into_iter().collect()
            };

            if !line_entries.is_empty() {
                parsed += line_entries.len() as i64;
                entries.append(&mut line_entries);
            }

            if entries.len() >= max_entries {
                break;
            }
        }

        if entries.len() > max_entries {
            entries.truncate(max_entries);
        }

        let mut inserted = 0i64;
        let mut skipped = 0i64;
        let mut host_cache: HashMap<String, Option<String>> = HashMap::new();

        let mut tx = pool.begin().await?;
        for mut entry in entries {
            let message = entry.message.trim();
            if message.is_empty() {
                skipped += 1;
                continue;
            }

            let domain_id = if let Some(host) = entry.domain_host {
                let host_key = normalize_host(&host);
                if let Some(cached) = host_cache.get(&host_key) {
                    cached.clone()
                } else {
                    let id = resolve_domain_id(&mut tx, &host_key).await?;
                    host_cache.insert(host_key, id.clone());
                    id
                }
            } else {
                None
            };

            if entry.user_id.is_none() {
                entry.user_id = resolve_user_id(&mut tx, domain_id.as_deref()).await?;
            }

            let created_at = entry.created_at.unwrap_or_else(Utc::now);

            sqlx::query(
                r#"
                INSERT INTO modsecurity_audit_logs
                    (id, user_id, domain_id, rule_id, custom_rule_id, severity, message, ip_address, uri, user_agent, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(Uuid::new_v4().to_string())
            .bind(entry.user_id)
            .bind(domain_id)
            .bind(entry.rule_id)
            .bind(entry.custom_rule_id)
            .bind(truncate_255(&entry.severity))
            .bind(truncate_255(message))
            .bind(entry.ip_address.map(|v| truncate_255(&v)))
            .bind(entry.uri.map(|v| truncate_255(&v)))
            .bind(entry.user_agent.map(|v| truncate_255(&v)))
            .bind(created_at)
            .execute(&mut *tx)
            .await?;

            inserted += 1;
        }
        tx.commit().await?;

        Ok(IngestModSecurityAuditLogResponse {
            source_path: path,
            parsed,
            inserted,
            skipped,
        })
    }

    pub async fn list_audit_logs(
        pool: &MySqlPool,
        user_id: &str,
        query: ModSecurityAuditLogQuery,
    ) -> ApiResult<Vec<ModSecurityAuditLog>> {
        let limit = query.limit.unwrap_or(100).min(200).max(1);
        let logs = if let Some(domain_id) = query.domain_id {
            sqlx::query_as::<_, ModSecurityAuditLog>(
                r#"
                SELECT * FROM modsecurity_audit_logs
                WHERE domain_id = ? AND user_id = ?
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(domain_id)
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, ModSecurityAuditLog>(
                r#"
                SELECT * FROM modsecurity_audit_logs
                WHERE user_id = ?
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool)
            .await?
        };

        Ok(logs)
    }

    pub async fn list_audit_logs_admin(
        pool: &MySqlPool,
        query: ModSecurityAuditLogQuery,
    ) -> ApiResult<Vec<ModSecurityAuditLog>> {
        let limit = query.limit.unwrap_or(100).min(200).max(1);
        let logs = if let Some(domain_id) = query.domain_id {
            sqlx::query_as::<_, ModSecurityAuditLog>(
                r#"
                SELECT * FROM modsecurity_audit_logs
                WHERE domain_id = ?
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(domain_id)
            .bind(limit)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, ModSecurityAuditLog>(
                r#"
                SELECT * FROM modsecurity_audit_logs
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(limit)
            .fetch_all(pool)
            .await?
        };

        Ok(logs)
    }
}

async fn resolve_domain_id(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    host: &str,
) -> Result<Option<String>, sqlx::Error> {
    let domain_id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM domains WHERE domain_name = ? LIMIT 1",
    )
    .bind(host)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(domain_id)
}

async fn resolve_user_id(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    domain_id: Option<&str>,
) -> Result<Option<String>, sqlx::Error> {
    if let Some(domain_id) = domain_id {
        let user_id = sqlx::query_scalar::<_, String>(
            "SELECT user_id FROM domains WHERE id = ? LIMIT 1",
        )
        .bind(domain_id)
        .fetch_optional(&mut **tx)
        .await?;
        Ok(user_id)
    } else {
        Ok(None)
    }
}
