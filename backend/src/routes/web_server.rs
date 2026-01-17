//! # Web Server Routes
//!
//! Route handlers untuk Web Server management (VirtualHost, SSL).

use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{
    CreateVirtualHostRequest, RequestSslRequest, SslCertificateResponse, UpdateVirtualHostRequest,
    VirtualHostResponse,
};
use crate::services::WebServerService;
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// VIRTUAL HOST ENDPOINTS
// ==========================================

/// List user's virtual hosts
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/")]
pub async fn list_vhosts(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<VirtualHostResponse>>>> {
    let vhosts = WebServerService::get_user_vhosts(db.get_pool(), &user.id).await?;
    Ok(success(vhosts))
}

/// Get virtual host by ID
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/<id>")]
pub async fn get_vhost(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<VirtualHostResponse>>> {
    let vhost = WebServerService::get_vhost_by_id(db.get_pool(), id, &user.id).await?;
    Ok(success(vhost))
}

/// Create virtual host
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "domain_id": "domain-uuid",
///   "php_version": "8.2",  // optional
///   "ssl_enabled": false  // optional
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn create_vhost(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateVirtualHostRequest>,
) -> ApiResult<Json<ApiResponse<VirtualHostResponse>>> {
    let vhost =
        WebServerService::create_vhost(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(vhost))
}

/// Update virtual host
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "php_version": "8.3",
///   "force_https": true,
///   "custom_config": "location /new { ... }"
/// }
/// ```
#[put("/<id>", format = "json", data = "<request>")]
pub async fn update_vhost(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateVirtualHostRequest>,
) -> ApiResult<Json<ApiResponse<VirtualHostResponse>>> {
    let vhost =
        WebServerService::update_vhost(db.get_pool(), id, &user.id, request.into_inner()).await?;
    Ok(success(vhost))
}

/// Delete virtual host
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[delete("/<id>")]
pub async fn delete_vhost(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    WebServerService::delete_vhost(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Virtual Host berhasil dihapus"))
}

// ==========================================
// SSL ENDPOINTS
// ==========================================

/// Request SSL Certificate (Let's Encrypt)
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[post("/ssl", format = "json", data = "<request>")]
pub async fn request_ssl(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<RequestSslRequest>,
) -> ApiResult<Json<ApiResponse<SslCertificateResponse>>> {
    let cert = WebServerService::request_ssl(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(cert))
}

/// Get SSL Certificate status
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/ssl/<vhost_id>")]
pub async fn get_ssl_status(
    db: &State<Database>,
    user: AuthenticatedUser,
    vhost_id: &str,
) -> ApiResult<Json<ApiResponse<SslCertificateResponse>>> {
    let cert = WebServerService::get_ssl_status(db.get_pool(), vhost_id, &user.id).await?;
    Ok(success(cert))
}

/// Mendapatkan routes untuk web server
pub fn web_server_routes() -> Vec<Route> {
    routes![
        list_vhosts,
        get_vhost,
        create_vhost,
        update_vhost,
        delete_vhost,
        request_ssl,
        get_ssl_status
    ]
}
