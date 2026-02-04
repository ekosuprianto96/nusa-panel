//! # App Installer Routes
//!
//! Route handlers untuk Module App Installer.

use rocket::serde::json::Json;
use rocket::{get, post, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::{AuthenticatedUser, ResellerOrAdmin};
use crate::models::{AppInstallationResponse, InstallAppRequest, InstallAppResponse};
use crate::services::AppInstallerService;
use crate::utils::response::{success, ApiResponse};

// ==========================================
// INSTALLER ENDPOINTS
// ==========================================

/// Install Application (WordPress, Laravel)
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "domain_id": "uuid",
///   "app_type": "wordpress",
///   "site_title": "My Blog",
///   "admin_username": "admin",
///   "admin_password": "securepassword",
///   "admin_email": "admin@example.com"
/// }
/// ```
#[post("/install", format = "json", data = "<request>")]
pub async fn install_app(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<InstallAppRequest>,
) -> ApiResult<Json<ApiResponse<InstallAppResponse>>> {
    let result =
        AppInstallerService::install_app(db.get_pool(), &user.username, &user.id, request.into_inner()).await?;
    Ok(success(result))
}

/// List installations untuk user saat ini
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/installations")]
pub async fn list_installations(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<AppInstallationResponse>>>> {
    let data = AppInstallerService::list_installations_by_user(db.get_pool(), &user.id).await?;
    Ok(success(data))
}

/// List installations untuk user tertentu (Admin/Reseller)
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - user_id: User ID
#[get("/admin/user/<user_id>/installations")]
pub async fn list_installations_by_user_admin(
    db: &State<Database>,
    _admin: ResellerOrAdmin,
    user_id: &str,
) -> ApiResult<Json<ApiResponse<Vec<AppInstallationResponse>>>> {
    let data = AppInstallerService::list_installations_by_user(db.get_pool(), user_id).await?;
    Ok(success(data))
}

/// Mendapatkan routes untuk app installer
pub fn app_routes() -> Vec<Route> {
    routes![install_app, list_installations, list_installations_by_user_admin]
}
