//! # Security Routes
//!
//! Route handlers untuk Security & Monitoring.

use rocket::serde::json::Json;
use rocket::{delete, get, post, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::{AdminUser, AuthenticatedUser};
use crate::models::{
    AccessLogEntry, BlockedIp, CreateBlockedIpRequest, CreateSshKeyRequest, ResourceUsageStats,
    SshAccessResponse,
};
use crate::services::SecurityService;
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// IP BLOCKING ENDPOINTS (Admin Only)
// ==========================================

/// List blocked IPs
///
/// # Headers
/// - Authorization: Bearer <admin_token>
#[get("/ips")]
pub async fn list_blocked_ips(
    db: &State<Database>,
    _admin: AdminUser,
) -> ApiResult<Json<ApiResponse<Vec<BlockedIp>>>> {
    let ips = SecurityService::get_blocked_ips(db.get_pool()).await?;
    Ok(success(ips))
}

/// Block IP Address
///
/// # Headers
/// - Authorization: Bearer <admin_token>
///
/// # Request Body
/// ```json
/// {
///   "ip_address": "192.168.1.100",
///   "reason": "Brute force attempt"
/// }
/// ```
#[post("/ips", format = "json", data = "<request>")]
pub async fn block_ip(
    db: &State<Database>,
    _admin: AdminUser,
    request: Json<CreateBlockedIpRequest>,
) -> ApiResult<Json<ApiResponse<BlockedIp>>> {
    let ip = SecurityService::block_ip(db.get_pool(), request.into_inner()).await?;
    Ok(success(ip))
}

/// Unblock IP Address
///
/// # Headers
/// - Authorization: Bearer <admin_token>
#[delete("/ips/<id>")]
pub async fn unblock_ip(
    db: &State<Database>,
    _admin: AdminUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::unblock_ip(db.get_pool(), id).await?;
    Ok(success_message("IP Address berhasil dibuka"))
}

// ==========================================
// SSH ENDPOINTS
// ==========================================

/// List user's SSH keys
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/ssh")]
pub async fn list_ssh_keys(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<SshAccessResponse>>>> {
    let keys = SecurityService::get_user_ssh_keys(db.get_pool(), &user.id).await?;
    Ok(success(keys))
}

/// Add SSH Key
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "label": "My Laptop",
///   "public_key": "ssh-rsa AAAAB3Nz..."
/// }
/// ```
#[post("/ssh", format = "json", data = "<request>")]
pub async fn add_ssh_key(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateSshKeyRequest>,
) -> ApiResult<Json<ApiResponse<SshAccessResponse>>> {
    let key = SecurityService::add_ssh_key(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(key))
}

/// Delete SSH Key
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[delete("/ssh/<id>")]
pub async fn delete_ssh_key(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::delete_ssh_key(db.get_pool(), id, &user.id).await?;
    Ok(success_message("SSH Key berhasil dihapus"))
}

// ==========================================
// MONITORING ENDPOINTS
// ==========================================

/// Get Resource Usage Stats
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/stats")]
pub async fn get_resource_stats(
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<ResourceUsageStats>>> {
    let stats = SecurityService::get_resource_usage(&user.id).await?;
    Ok(success(stats))
}

/// Get Access Logs
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/logs")]
pub async fn get_access_logs(
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<AccessLogEntry>>>> {
    let logs = SecurityService::get_access_logs(&user.id).await?;
    Ok(success(logs))
}

/// Mendapatkan routes untuk security
pub fn security_routes() -> Vec<Route> {
    routes![
        list_blocked_ips,
        block_ip,
        unblock_ip,
        list_ssh_keys,
        add_ssh_key,
        delete_ssh_key,
        get_resource_stats,
        get_access_logs
    ]
}
