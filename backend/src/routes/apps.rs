//! # App Installer Routes
//!
//! Route handlers untuk Module App Installer.

use rocket::serde::json::Json;
use rocket::{post, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{InstallAppRequest, InstallAppResponse};
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
        AppInstallerService::install_app(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(result))
}

/// Mendapatkan routes untuk app installer
pub fn app_routes() -> Vec<Route> {
    routes![install_app]
}
