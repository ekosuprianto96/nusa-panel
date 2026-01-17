//! # Redis Model
//!
//! Model dan DTO untuk Redis Management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Redis Instance entity
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RedisInstance {
    /// Unique identifier
    pub id: String,

    /// User ID pemilik
    pub user_id: String,

    /// Status aktif
    pub is_active: bool,

    /// Password Redis (untuk ACL/Auth)
    #[serde(skip_serializing)]
    pub password: String,

    /// Max memory limit (e.g. "64mb")
    pub max_memory: String,

    /// Path ke Unix Socket (e.g. /home/user/.redis/redis.sock)
    pub socket_path: String,

    /// Waktu pembuatan
    pub created_at: DateTime<Utc>,

    /// Waktu update terakhir
    pub updated_at: DateTime<Utc>,
}

/// DTO Response Redis
#[derive(Debug, Serialize)]
pub struct RedisInstanceResponse {
    pub is_active: bool,
    pub socket_path: String,
    pub max_memory: String,
    // Password ditampilkan saat create/get detail agar user bisa config di aplikasinya
    pub password: Option<String>, 
}

/// DTO Request Enable Redis
#[derive(Debug, Deserialize, Validate)]
pub struct EnableRedisRequest {
    /// Max Memory (opsional, default 64mb)
    pub max_memory: Option<String>,
}

/// DTO Request Update Redis Config
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRedisRequest {
    pub max_memory: Option<String>,
    pub password: Option<String>, // Reset password
}
