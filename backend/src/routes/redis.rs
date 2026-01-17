//! # Redis Routes
//!
//! Route handlers untuk Redis Management.

use rocket::serde::json::Json;
use rocket::{delete, get, post, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{EnableRedisRequest, RedisInstanceResponse};
use crate::services::RedisService; // Ganti dengan RedisServiceReal di production
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// REDIS ENDPOINTS
// ==========================================

/// Get Redis Status
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/")]
pub async fn get_redis_status(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Option<RedisInstanceResponse>>>> {
    let status = RedisService::get_status(db.get_pool(), &user.id).await?;
    Ok(success(Some(status)))
}

/// Enable Redis
///
/// # Request Body
/// ```json
/// {
///   "max_memory": "128mb"
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn enable_redis(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<EnableRedisRequest>,
) -> ApiResult<Json<ApiResponse<RedisInstanceResponse>>> {
    let instance = RedisService::enable_redis(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(instance))
}

/// Disable Redis
#[delete("/")]
pub async fn disable_redis(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<()>>> {
    RedisService::disable_redis(db.get_pool(), &user.id).await?;
    Ok(success_message("Redis instance berhasil dimatikan"))
}

/// Mendapatkan routes untuk redis
pub fn redis_routes() -> Vec<Route> {
    routes![get_redis_status, enable_redis, disable_redis]
}
