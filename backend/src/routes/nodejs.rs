use rocket::serde::json::Json;
use rocket::{get, post, put, routes, Route};
use serde::Deserialize;

use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::services::nodejs_service::NodejsService;
use crate::utils::response::{success, success_message, ApiResponse};

#[derive(Deserialize)]
pub struct VersionRequest {
    pub version: String,
}

/// Get Node.js status
#[get("/")]
pub async fn get_status(user: AuthenticatedUser) -> ApiResult<Json<ApiResponse<crate::services::nodejs_service::NodejsStatus>>> {
    let status = NodejsService::get_status(&user.username)?;
    Ok(success(status))
}

/// Install NVM
#[post("/install-nvm")]
pub async fn install_nvm(user: AuthenticatedUser) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::install_nvm(&user.username)?;
    Ok(success_message("NVM installed successfully"))
}

/// List available LTS versions
#[get("/available")]
pub async fn list_available(user: AuthenticatedUser) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let versions = NodejsService::list_available_lts(&user.username)?;
    Ok(success(versions))
}

/// Install Node.js version
#[post("/install", format = "json", data = "<data>")]
pub async fn install_version(
    user: AuthenticatedUser,
    data: Json<VersionRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::install_version(&user.username, &data.version)?;
    Ok(success_message(&format!("Node.js {} installed", data.version)))
}

/// Uninstall Node.js version
#[post("/uninstall", format = "json", data = "<data>")]
pub async fn uninstall_version(
    user: AuthenticatedUser,
    data: Json<VersionRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::uninstall_version(&user.username, &data.version)?;
    Ok(success_message(&format!("Node.js {} uninstalled", data.version)))
}

/// Set default Node.js version
#[put("/default", format = "json", data = "<data>")]
pub async fn set_default(
    user: AuthenticatedUser,
    data: Json<VersionRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::set_default(&user.username, &data.version)?;
    Ok(success_message(&format!("Default Node.js version set to {}", data.version)))
}

pub fn nodejs_routes() -> Vec<Route> {
    routes![
        get_status,
        install_nvm,
        list_available,
        install_version,
        uninstall_version,
        set_default
    ]
}
