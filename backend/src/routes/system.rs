//! # System Routes
//!
//! Route handlers untuk System Tools (Cron, Backup, Services).

use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::{AuthenticatedUser};
use crate::models::{
    CreateBackupRequest, CreateCronJobRequest, CronJob, ServiceStatus, SystemBackup,
    UpdateCronJobRequest, ResourceUsage,
};
use crate::services::SystemService;
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// CRON JOB ENDPOINTS
// ==========================================

/// List user's cron jobs
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/cron")]
pub async fn list_cron_jobs(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<CronJob>>>> {
    let jobs = SystemService::get_user_cron_jobs(db.get_pool(), &user.id).await?;
    Ok(success(jobs))
}

/// Create cron job
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "schedule": "0 * * * *",
///   "command": "php /home/user/script.php",
///   "description": "Hourly script"
/// }
/// ```
#[post("/cron", format = "json", data = "<request>")]
pub async fn create_cron_job(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateCronJobRequest>,
) -> ApiResult<Json<ApiResponse<CronJob>>> {
    let job = SystemService::create_cron_job(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(job))
}

/// Update cron job
#[put("/cron/<id>", format = "json", data = "<request>")]
pub async fn update_cron_job(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateCronJobRequest>,
) -> ApiResult<Json<ApiResponse<CronJob>>> {
    let job =
        SystemService::update_cron_job(db.get_pool(), id, &user.id, request.into_inner()).await?;
    Ok(success(job))
}

/// Delete cron job
#[delete("/cron/<id>")]
pub async fn delete_cron_job(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SystemService::delete_cron_job(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Cron job berhasil dihapus"))
}

// ==========================================
// BACKUP ENDPOINTS
// ==========================================

/// List user's backups
#[get("/backups")]
pub async fn list_backups(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<SystemBackup>>>> {
    let backups = SystemService::get_user_backups(db.get_pool(), &user.id).await?;
    Ok(success(backups))
}

/// Create backup
///
/// # Request Body
/// ```json
/// {
///   "backup_type": "full", // full, database, homedir
///   "description": "Weekly backup"
/// }
/// ```
#[post("/backups", format = "json", data = "<request>")]
pub async fn create_backup(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateBackupRequest>,
) -> ApiResult<Json<ApiResponse<SystemBackup>>> {
    let backup =
        SystemService::create_backup(db.get_pool(), &user.id, &user.username, request.into_inner()).await?;
    Ok(success(backup))
}

/// Delete backup
#[delete("/backups/<id>")]
pub async fn delete_backup(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SystemService::delete_backup(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Backup berhasil dihapus"))
}

// ==========================================
// SERVICE STATUS ENDPOINTS (Any Authenticated User)
// ==========================================

/// Get system service status
#[get("/services")]
pub async fn get_services_status(
    _user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<ServiceStatus>>>> {
    let status = SystemService::get_service_status().await?;
    Ok(success(status))
}

/// Request untuk control service
#[derive(Debug, serde::Deserialize)]
pub struct ControlServiceRequest {
    pub action: String, // start, stop, restart
}

/// Control system service (start/stop/restart)
///
/// # Request Body
/// ```json
/// {
///   "action": "restart" // start, stop, restart
/// }
/// ```
#[post("/services/<service_name>", format = "json", data = "<request>")]
pub async fn control_service(
    _user: AuthenticatedUser,
    service_name: &str,
    request: Json<ControlServiceRequest>,
) -> ApiResult<Json<ApiResponse<ServiceStatus>>> {
    let status = SystemService::control_service(service_name, &request.action).await?;
    Ok(success(status))
}

// ==========================================
// PHP VERSION ROUTES
// ==========================================

/// Get installed PHP versions
#[get("/php/versions")]
pub async fn get_php_versions(_user: AuthenticatedUser) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let versions = SystemService::get_php_versions().await?;
    Ok(success(versions))
}

/// Get current PHP version
#[get("/php/current")]
pub async fn get_current_php_version(_user: AuthenticatedUser) -> ApiResult<Json<ApiResponse<String>>> {
    let version = SystemService::get_current_php_version().await?;
    Ok(success(version))
}

/// Request schema for changing PHP version
#[derive(Debug, serde::Deserialize)]
pub struct ChangePhpRequest {
    pub version: String,
}

/// Change PHP version
#[post("/php/change", format = "json", data = "<request>")]
pub async fn change_php_version(
    _user: AuthenticatedUser,
    request: Json<ChangePhpRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let version = SystemService::change_php_version(&request.version).await?;
    Ok(success(version))
}

// ==========================================
// USER PHP VERSION (PER-USER)
// ==========================================

/// Get preferred PHP version for current user
#[get("/php/user-version")]
pub async fn get_user_php_version(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Option<String>>>> {
    let version = SystemService::get_user_php_version(db.get_pool(), &user.id).await?;
    Ok(success(version))
}

/// Set preferred PHP version for current user
#[post("/php/user-version", format = "json", data = "<request>")]
pub async fn set_user_php_version(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<ChangePhpRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let version =
        SystemService::set_user_php_version(db.get_pool(), &user.id, &request.version).await?;
    Ok(success(version))
}

// ==========================================
// ERROR LOGS ROUTES
// ==========================================

/// Get system error logs
#[get("/logs?<lines>")]
pub async fn get_error_logs(
    _user: AuthenticatedUser,
    lines: Option<usize>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    // Default 50 baris, max 200
    let lines = lines.unwrap_or(50).min(200);
    let logs = SystemService::get_error_logs(lines).await?;
    Ok(success(logs))
}

/// Clear system logs
#[post("/logs/clear")]
pub async fn clear_error_logs(_user: AuthenticatedUser) -> ApiResult<Json<ApiResponse<()>>> {
    SystemService::clear_error_logs().await?;
    Ok(success_message("Error logs cleared"))
}

// ==========================================
// DNS TRACKER ENDPOINTS
// ==========================================

#[derive(serde::Deserialize)]
pub struct DnsLookupRequest {
    pub domain: String,
    pub record_type: String, // A, AAAA, MX, NS, TXT, CNAME
}

#[derive(serde::Deserialize)]
pub struct TraceRouteRequest {
    pub domain: String,
}

/// DNS Lookup
#[post("/dns/lookup", format = "json", data = "<request>")]
pub async fn dns_lookup(
    _user: AuthenticatedUser,
    request: Json<DnsLookupRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let result = SystemService::dns_lookup(&request.domain, &request.record_type).await?;
    Ok(success(result))
}

/// Trace Route
#[post("/dns/traceroute", format = "json", data = "<request>")]
pub async fn trace_route(
    _user: AuthenticatedUser,
    request: Json<TraceRouteRequest>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let result = SystemService::trace_route(&request.domain).await?;
    Ok(success(result))
}

// ==========================================
// RESOURCE USAGE ENDPOINTS
// ==========================================

/// Get Resource Usage (CPU, RAM, Disk)
#[get("/resources")]
pub async fn get_resource_usage(
    _user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<ResourceUsage>>> {
    let usage = SystemService::get_resource_usage().await?;
    Ok(success(usage))
}

/// Mendapatkan routes untuk system tools
pub fn system_routes() -> Vec<Route> {
    routes![
        list_cron_jobs,
        create_cron_job,
        update_cron_job,
        delete_cron_job,
        list_backups,
        create_backup,
        delete_backup,
        get_services_status,
        control_service,
        get_php_versions,
        get_current_php_version,
        change_php_version,
        get_user_php_version,
        set_user_php_version,
        get_error_logs,
        clear_error_logs,
        dns_lookup,
        trace_route,
        get_resource_usage
    ]
}
