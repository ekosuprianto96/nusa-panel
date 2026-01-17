//! # Redis Service
//!
//! Service Logic untuk Redis Management (Simulation).
//! Mengatur konfigurasi dan status Redis instance per user.

use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{EnableRedisRequest, RedisInstance, RedisInstanceResponse, UpdateRedisRequest};

pub struct RedisService;

impl RedisService {
    /// Get Redis status for user
    pub async fn get_status(pool: &MySqlPool, user_id: &str) -> ApiResult<Option<RedisInstanceResponse>> {
        let instance = sqlx::query_as::<_, RedisInstance>(
            "SELECT * FROM redis_instances WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match instance {
            Some(inst) => Ok(Some(RedisInstanceResponse {
                is_active: inst.is_active,
                socket_path: inst.socket_path,
                max_memory: inst.max_memory,
                password: Some(inst.password), // Show password for client config
            })),
            None => Ok(None),
        }
    }

    /// Enable Redis for user
    pub async fn enable_redis(
        pool: &MySqlPool,
        user_id: &str,
        request: EnableRedisRequest,
    ) -> ApiResult<RedisInstanceResponse> {
        // 1. Check if already exists
        let existing = Self::get_status(pool, user_id).await?;
        if existing.is_some() {
            return Err(ApiError::AlreadyExists("Redis Instance".to_string()));
        }

        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 2. Generate config
        let id = Uuid::new_v4().to_string();
        let password: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        
        let socket_path = format!("/home/user_{}/.redis/redis.sock", user_id.replace("-", ""));
        let max_memory = request.max_memory.unwrap_or_else(|| "64mb".to_string());
        let now = Utc::now();

        // 3. Save to DB
        sqlx::query(
            r#"
            INSERT INTO redis_instances (id, user_id, is_active, password, max_memory, socket_path, created_at, updated_at)
            VALUES (?, ?, TRUE, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&id)
        .bind(user_id)
        .bind(&password)
        .bind(&max_memory)
        .bind(&socket_path)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // TODO: Start system service (simulated here)
        tracing::info!("Redis instance enabled for user {}", user_id);

        Ok(RedisInstanceResponse {
            is_active: true,
            socket_path,
            max_memory,
            password: Some(password),
        })
    }

    /// Disable Redis
    pub async fn disable_redis(pool: &MySqlPool, user_id: &str) -> ApiResult<()> {
        let instance = sqlx::query_as::<_, RedisInstance>("SELECT * FROM redis_instances WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Redis Instance".to_string()))?;

        sqlx::query("DELETE FROM redis_instances WHERE id = ?")
            .bind(&instance.id)
            .execute(pool)
            .await?;

        // TODO: Stop system service
        tracing::info!("Redis instance disabled for user {}", user_id);

        Ok(())
    }
}
