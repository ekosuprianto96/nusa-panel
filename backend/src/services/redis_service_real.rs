//! # Redis Service (REAL IMPLEMENTATION)
//!
//! Service Logic untuk Redis Management (REAL with systemd & unix socket).
//! Creates private Redis instance per user.

use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::MySqlPool;
use std::fs;
use std::process::Command;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{EnableRedisRequest, RedisInstance, RedisInstanceResponse};

pub struct RedisServiceReal;

impl RedisServiceReal {
    /// Helper to get system username
    fn get_system_username(username: &str) -> String {
        format!("user_{}", username)
    }

    /// Enable Redis for user (Real)
    pub async fn enable_redis(
        pool: &MySqlPool,
        user_id: &str,
        request: EnableRedisRequest,
    ) -> ApiResult<RedisInstanceResponse> {
        // 1. Check DB
        let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM redis_instances WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        
        if existing > 0 {
            return Err(ApiError::AlreadyExists("Redis Instance".to_string()));
        }

        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 2. Generate Params
        let id = Uuid::new_v4().to_string();
        let password: String = thread_rng().sample_iter(&Alphanumeric).take(32).map(char::from).collect();
        let system_username = Self::get_system_username(user_id);
        let redis_dir = format!("/home/{}/.redis", system_username);
        let socket_path = format!("{}/redis.sock", redis_dir);
        let pid_path = format!("{}/redis.pid", redis_dir);
        let conf_path = format!("{}/redis.conf", redis_dir);
        let max_memory = request.max_memory.unwrap_or_else(|| "64mb".to_string());
        
        // 3. Create configs & dirs
        // Generate redis.conf
        let redis_conf = format!(
            r#"
daemonize yes
pidfile {}
port 0  # Disable networking
unixsocket {}
unixsocketperm 770
timeout 0
tcp-keepalive 300
loglevel notice
logfile {}/redis.log
databases 16
requirepass {}
maxmemory {}
maxmemory-policy allkeys-lru
            "#,
            pid_path,
            socket_path,
            redis_dir,
            password,
            max_memory
        );

        // Mkdir & Write
        // sudo mkdir -p /home/user/.redis
        fs::create_dir_all(&redis_dir).map_err(|e| ApiError::InternalError(format!("Mkdir failed: {}", e)))?;
        fs::write(&conf_path, redis_conf).map_err(|e| ApiError::InternalError(format!("Write conf failed: {}", e)))?;
        
        // Chown
        Command::new("chown")
            .arg("-R")
            .arg(format!("{}:{}", system_username, system_username))
            .arg(&redis_dir)
            .output()?;

        // 4. Create Systemd Service (User level or Template)
        // We use template `nusa-redis@.service` approach or direct unit file
        // For simplicity: Create a specific unit file
        let service_name = format!("nusa-redis-{}", system_username);
        let service_path = format!("/etc/systemd/system/{}.service", service_name);
        
        let unit_file = format!(
            r#"
[Unit]
Description=Redis Server for {}
After=network.target

[Service]
Type=forking
User={}
Group={}
ExecStart=/usr/bin/redis-server {}
PIDFile={}
Restart=always

[Install]
WantedBy=multi-user.target
            "#,
            system_username,
            system_username,
            system_username,
            conf_path,
            pid_path
        );

        fs::write(&service_path, unit_file).map_err(|e| ApiError::InternalError(format!("Write service failed: {}", e)))?;

        // Reload daemon & Start
        Command::new("systemctl").arg("daemon-reload").output()?;
        Command::new("systemctl").arg("enable").arg(&service_name).output()?;
        let start = Command::new("systemctl").arg("start").arg(&service_name).output()?;

        if !start.status.success() {
             return Err(ApiError::InternalError(format!("Start failed: {}", String::from_utf8_lossy(&start.stderr))));
        }

        // 5. Save to DB
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO redis_instances (id, user_id, is_active, password, max_memory, socket_path, created_at, updated_at) VALUES (?, ?, TRUE, ?, ?, ?, ?, ?)"
        )
        .bind(&id).bind(user_id).bind(&password).bind(&max_memory).bind(&socket_path).bind(now).bind(now)
        .execute(pool)
        .await?;

        Ok(RedisInstanceResponse {
            is_active: true,
            socket_path,
            max_memory,
            password: Some(password),
        })
    }
    /// Get Redis status for user (Real)
    pub async fn get_status(pool: &MySqlPool, user_id: &str) -> ApiResult<RedisInstanceResponse> {
        let instance = sqlx::query_as::<_, RedisInstance>("SELECT * FROM redis_instances WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Redis Instance".to_string()))?;

        Ok(RedisInstanceResponse {
            is_active: instance.is_active,
            socket_path: instance.socket_path,
            max_memory: instance.max_memory,
            password: Some(instance.password),
        })
    }

    /// Disable Redis for user (Real)
    pub async fn disable_redis(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        let system_username = Self::get_system_username(user_id);
        let service_name = format!("nusa-redis-{}", system_username);

        // 1. Stop and Disable Service
        let _ = Command::new("systemctl").arg("stop").arg(&service_name).output();
        let _ = Command::new("systemctl").arg("disable").arg(&service_name).output();

        // 2. Remove unit file
        let service_path = format!("/etc/systemd/system/{}.service", service_name);
        let _ = fs::remove_file(service_path);
        let _ = Command::new("systemctl").arg("daemon-reload").output();

        // 3. Delete from DB
        sqlx::query("DELETE FROM redis_instances WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
