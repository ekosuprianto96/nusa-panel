//! # Security Routes
//!
//! Route handlers untuk Security & Monitoring.

use rocket::serde::json::Json;
use rocket::{delete, get, post, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::{AdminUser, AuthenticatedUser, RequestInfo};
use crate::models::{
    AccessLogEntry, BlockedIp, CreateBlockedIpRequest, CreateSshKeyRequest, ModSecurityOverview,
    ModSecurityRuleSet, ModSecuritySettings, ResourceUsageStats, SshAccessResponse,
    TwoFactorSetupResponse, TwoFactorStatusResponse, TwoFactorVerifyRequest,
    UpdateModSecurityDomainRequest, UpdateModSecurityRuleRequest, UpdateModSecuritySettingsRequest,
    ModSecurityCustomRule, ModSecurityAuditLog, ModSecurityAuditLogQuery,
    CreateModSecurityCustomRuleRequest, UpdateModSecurityCustomRuleRequest,
    IngestModSecurityAuditLogRequest, IngestModSecurityAuditLogResponse,
};
use crate::services::SecurityService;
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// IP BLOCKING ENDPOINTS (Admin Only)
// ==========================================

/// List blocked IPs
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/ips")]
pub async fn list_blocked_ips(
    db: &State<Database>,
    _user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<BlockedIp>>>> {
    let ips = SecurityService::get_blocked_ips(db.get_pool()).await?;
    Ok(success(ips))
}

/// Block IP Address
///
/// # Headers
/// - Authorization: Bearer <access_token>
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
    user: AuthenticatedUser,
    req_info: RequestInfo,
    request: Json<CreateBlockedIpRequest>,
) -> ApiResult<Json<ApiResponse<BlockedIp>>> {
    let ip = SecurityService::block_ip(db.get_pool(), request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "ip_blocked",
        req_info.ip_address.as_deref(),
        Some(&ip.ip_address),
        Some("blocked"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success(ip))
}

/// Unblock IP Address
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[delete("/ips/<id>")]
pub async fn unblock_ip(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::unblock_ip(db.get_pool(), id).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "ip_unblocked",
        req_info.ip_address.as_deref(),
        Some(id),
        Some("unblocked"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
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
    req_info: RequestInfo,
    request: Json<CreateSshKeyRequest>,
) -> ApiResult<Json<ApiResponse<SshAccessResponse>>> {
    let key = SecurityService::add_ssh_key(db.get_pool(), &user.id, request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "ssh_key_added",
        req_info.ip_address.as_deref(),
        Some(&key.label),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
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
    req_info: RequestInfo,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::delete_ssh_key(db.get_pool(), id, &user.id).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "ssh_key_deleted",
        req_info.ip_address.as_deref(),
        Some(id),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
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
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<ResourceUsageStats>>> {
    let stats = SecurityService::get_resource_usage(db.get_pool(), &user.id).await?;
    Ok(success(stats))
}

/// Get Access Logs
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/logs")]
pub async fn get_access_logs(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<AccessLogEntry>>>> {
    let logs = SecurityService::get_access_logs(db.get_pool(), &user.id).await?;
    Ok(success(logs))
}

// ==========================================
// 2FA ENDPOINTS
// ==========================================

#[get("/2fa/status")]
pub async fn get_2fa_status(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<TwoFactorStatusResponse>>> {
    let status = SecurityService::get_2fa_status(db.get_pool(), &user.id).await?;
    Ok(success(status))
}

#[post("/2fa/setup")]
pub async fn setup_2fa(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
) -> ApiResult<Json<ApiResponse<TwoFactorSetupResponse>>> {
    let setup = SecurityService::setup_2fa(db.get_pool(), &user.id, &user.email).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "2fa_setup",
        req_info.ip_address.as_deref(),
        Some("2fa"),
        Some("pending"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success(setup))
}

#[post("/2fa/verify", format = "json", data = "<request>")]
pub async fn verify_2fa(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    request: Json<TwoFactorVerifyRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::verify_2fa(db.get_pool(), &user.id, request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "2fa_enabled",
        req_info.ip_address.as_deref(),
        Some("2fa"),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success_message("2FA berhasil diaktifkan"))
}

#[delete("/2fa")]
pub async fn disable_2fa(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::disable_2fa(db.get_pool(), &user.id).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "2fa_disabled",
        req_info.ip_address.as_deref(),
        Some("2fa"),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success_message("2FA berhasil dinonaktifkan"))
}

// ==========================================
// SSL/TLS ENDPOINTS
// ==========================================

#[post("/ssl/auto")]
pub async fn run_auto_ssl(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::run_auto_ssl(db.get_pool(), &user.id).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "ssl_auto",
        req_info.ip_address.as_deref(),
        Some("auto_ssl"),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success_message("AutoSSL berhasil dijalankan"))
}

#[post("/ssl/renew/<domain_id>")]
pub async fn renew_ssl(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    domain_id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::renew_ssl(db.get_pool(), &user.id, domain_id).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "ssl_renew",
        req_info.ip_address.as_deref(),
        Some(domain_id),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success_message("SSL renewal dijadwalkan"))
}

// ==========================================
// MODSECURITY ENDPOINTS
// ==========================================

#[get("/modsecurity/overview")]
pub async fn get_modsecurity_overview(
    db: &State<Database>,
    _user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<ModSecurityOverview>>> {
    let overview = SecurityService::get_modsecurity_overview(db.get_pool()).await?;
    Ok(success(overview))
}

#[get("/modsecurity/settings")]
pub async fn get_modsecurity_settings(
    db: &State<Database>,
    _user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<ModSecuritySettings>>> {
    let overview = SecurityService::get_modsecurity_overview(db.get_pool()).await?;
    Ok(success(overview.settings))
}

#[post("/modsecurity/settings", format = "json", data = "<request>")]
pub async fn update_modsecurity_settings(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    request: Json<UpdateModSecuritySettingsRequest>,
) -> ApiResult<Json<ApiResponse<ModSecuritySettings>>> {
    let settings = SecurityService::update_modsecurity_settings(db.get_pool(), request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "modsecurity_settings_updated",
        req_info.ip_address.as_deref(),
        Some("modsecurity_settings"),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success(settings))
}

#[post("/modsecurity/rules/<rule_id>", format = "json", data = "<request>")]
pub async fn update_modsecurity_rule(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    rule_id: &str,
    request: Json<UpdateModSecurityRuleRequest>,
) -> ApiResult<Json<ApiResponse<ModSecurityRuleSet>>> {
    let rule = SecurityService::update_modsecurity_rule(db.get_pool(), rule_id, request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "modsecurity_rule_updated",
        req_info.ip_address.as_deref(),
        Some(rule_id),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success(rule))
}

#[post("/modsecurity/domains/<domain_id>", format = "json", data = "<request>")]
pub async fn update_modsecurity_domain(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    domain_id: &str,
    request: Json<UpdateModSecurityDomainRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::update_modsecurity_domain(db.get_pool(), domain_id, request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "modsecurity_domain_updated",
        req_info.ip_address.as_deref(),
        Some(domain_id),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success_message("ModSecurity domain berhasil diperbarui"))
}

// ==========================================
// MODSECURITY CUSTOM RULES
// ==========================================

#[get("/modsecurity/custom-rules")]
pub async fn list_custom_rules(
    db: &State<Database>,
    _user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<ModSecurityCustomRule>>>> {
    let rules = SecurityService::list_custom_rules(db.get_pool()).await?;
    Ok(success(rules))
}

#[post("/modsecurity/custom-rules", format = "json", data = "<request>")]
pub async fn create_custom_rule(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    request: Json<CreateModSecurityCustomRuleRequest>,
) -> ApiResult<Json<ApiResponse<ModSecurityCustomRule>>> {
    let rule = SecurityService::create_custom_rule(db.get_pool(), request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "modsecurity_custom_rule_created",
        req_info.ip_address.as_deref(),
        Some(&rule.id),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success(rule))
}

#[post("/modsecurity/custom-rules/<rule_id>", format = "json", data = "<request>")]
pub async fn update_custom_rule(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    rule_id: &str,
    request: Json<UpdateModSecurityCustomRuleRequest>,
) -> ApiResult<Json<ApiResponse<ModSecurityCustomRule>>> {
    let rule = SecurityService::update_custom_rule(db.get_pool(), rule_id, request.into_inner()).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "modsecurity_custom_rule_updated",
        req_info.ip_address.as_deref(),
        Some(rule_id),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success(rule))
}

#[delete("/modsecurity/custom-rules/<rule_id>")]
pub async fn delete_custom_rule(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    rule_id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    SecurityService::delete_custom_rule(db.get_pool(), rule_id).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "modsecurity_custom_rule_deleted",
        req_info.ip_address.as_deref(),
        Some(rule_id),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success_message("Custom rule berhasil dihapus"))
}

// ==========================================
// MODSECURITY AUDIT LOGS
// ==========================================

#[get("/modsecurity/audit-logs?<query..>")]
pub async fn list_audit_logs(
    db: &State<Database>,
    user: AuthenticatedUser,
    query: ModSecurityAuditLogQuery,
) -> ApiResult<Json<ApiResponse<Vec<ModSecurityAuditLog>>>> {
    let logs = SecurityService::list_audit_logs(db.get_pool(), &user.id, query).await?;
    Ok(success(logs))
}

#[get("/modsecurity/audit-logs/all?<query..>")]
pub async fn list_audit_logs_admin(
    db: &State<Database>,
    admin: AdminUser,
    req_info: RequestInfo,
    query: ModSecurityAuditLogQuery,
) -> ApiResult<Json<ApiResponse<Vec<ModSecurityAuditLog>>>> {
    let logs = SecurityService::list_audit_logs_admin(db.get_pool(), query).await?;
    SecurityService::log_event(
        db.get_pool(),
        Some(&admin.0.id),
        "modsecurity_audit_list_all",
        req_info.ip_address.as_deref(),
        None,
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();
    Ok(success(logs))
}

#[post("/modsecurity/audit-logs/ingest", format = "json", data = "<request>")]
pub async fn ingest_audit_logs(
    db: &State<Database>,
    user: AuthenticatedUser,
    req_info: RequestInfo,
    request: Json<IngestModSecurityAuditLogRequest>,
) -> ApiResult<Json<ApiResponse<IngestModSecurityAuditLogResponse>>> {
    let result = SecurityService::ingest_modsecurity_audit_log_file(
        db.get_pool(),
        request.into_inner(),
    )
    .await?;

    SecurityService::log_event(
        db.get_pool(),
        Some(&user.id),
        "modsecurity_audit_ingest",
        req_info.ip_address.as_deref(),
        Some(&result.source_path),
        Some("success"),
        req_info.user_agent.as_deref(),
    )
    .await
    .ok();

    Ok(success(result))
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
        get_access_logs,
        get_2fa_status,
        setup_2fa,
        verify_2fa,
        disable_2fa,
        run_auto_ssl,
        renew_ssl,
        get_modsecurity_overview,
        get_modsecurity_settings,
        update_modsecurity_settings,
        update_modsecurity_rule,
        update_modsecurity_domain,
        list_custom_rules,
        create_custom_rule,
        update_custom_rule,
        delete_custom_rule,
        list_audit_logs,
        list_audit_logs_admin,
        ingest_audit_logs
    ]
}
