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

#[derive(Deserialize)]
pub struct BootstrapRequest {
    pub version: Option<String>,
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

/// Bootstrap Node.js (install NVM, install version, set default)
#[post("/bootstrap", format = "json", data = "<data>")]
pub async fn bootstrap_nodejs(
    user: AuthenticatedUser,
    data: Json<BootstrapRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    let version = data.version.clone().unwrap_or_else(|| "20.10.0".to_string());
    NodejsService::install_nvm(&user.username)?;
    NodejsService::install_version(&user.username, &version)?;
    NodejsService::set_default(&user.username, &version)?;
    Ok(success_message(&format!("NVM installed and Node.js {} set as default", version)))
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

#[derive(Deserialize)]
pub struct EnvVarRequest {
    pub path: String,
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct DeleteEnvVarRequest {
    pub path: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct Pm2ActionRequest {
    pub action: String,
    pub target: String,
}

#[derive(Deserialize)]
pub struct NpmInstallRequest {
    pub package: Option<String>,
    pub version: Option<String>,
    pub dev: Option<bool>,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct NpmUninstallRequest {
    pub package: String,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct NpmRunRequest {
    pub script: String,
    pub path: Option<String>,
}

/// Get Env Vars
#[get("/env?<path>")]
pub async fn get_env_vars(user: AuthenticatedUser, path: String) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let vars = NodejsService::get_env_vars(&user.username, &path)?;
    Ok(success(vars))
}

/// Save Env Var
#[post("/env", format = "json", data = "<data>")]
pub async fn save_env_var(
    user: AuthenticatedUser,
    data: Json<EnvVarRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::save_env_var(&user.username, &data.path, &data.key, &data.value)?;
    Ok(success_message("Environment variable saved"))
}

/// Delete Env Var
#[post("/env/delete", format = "json", data = "<data>")]
pub async fn delete_env_var(
    user: AuthenticatedUser,
    data: Json<DeleteEnvVarRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::delete_env_var(&user.username, &data.path, &data.key)?;
    Ok(success_message("Environment variable deleted"))
}

/// List PM2 Processes
#[get("/pm2")]
pub async fn list_pm2(user: AuthenticatedUser) -> ApiResult<Json<ApiResponse<Vec<crate::services::nodejs_service::Pm2Process>>>> {
    let processes = NodejsService::pm2_list(&user.username)?;
    Ok(success(processes))
}

/// PM2 Action
#[post("/pm2/action", format = "json", data = "<data>")]
pub async fn pm2_action(
    user: AuthenticatedUser,
    data: Json<Pm2ActionRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::pm2_action(&user.username, &data.action, &data.target)?;
    Ok(success_message(&format!("PM2 action {} executed on {}", data.action, data.target)))
}

/// List NPM Packages
#[get("/npm?<path>")]
pub async fn list_npm_packages(
    user: AuthenticatedUser,
    path: Option<String>,
) -> ApiResult<Json<ApiResponse<Vec<crate::services::nodejs_service::NpmPackage>>>> {
    let packages = NodejsService::npm_list_in_path(&user.username, path.as_deref())?;
    Ok(success(packages))
}

/// Install NPM Package
#[post("/npm/install", format = "json", data = "<data>")]
pub async fn install_npm_package(
    user: AuthenticatedUser,
    data: Json<NpmInstallRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::npm_install(
        &user.username,
        data.package.as_deref(),
        data.version.as_deref(),
        data.dev.unwrap_or(false),
        data.path.as_deref(),
    )?;
    let pkg_label = data.package.clone().unwrap_or_else(|| "dependencies".to_string());
    Ok(success_message(&format!("NPM install completed for {}", pkg_label)))
}

/// Uninstall NPM Package
#[post("/npm/uninstall", format = "json", data = "<data>")]
pub async fn uninstall_npm_package(
    user: AuthenticatedUser,
    data: Json<NpmUninstallRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::npm_uninstall(&user.username, &data.package, data.path.as_deref())?;
    Ok(success_message(&format!("NPM package {} uninstalled", data.package)))
}

/// Run NPM script (e.g. start, build)
#[post("/npm/run", format = "json", data = "<data>")]
pub async fn run_npm_script(
    user: AuthenticatedUser,
    data: Json<NpmRunRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    NodejsService::npm_run(&user.username, &data.script, data.path.as_deref())?;
    Ok(success_message(&format!("NPM script {} executed", data.script)))
}

pub fn nodejs_routes() -> Vec<Route> {
    routes![
        get_status,
        install_nvm,
        bootstrap_nodejs,
        list_available,
        install_version,
        uninstall_version,
        set_default,
        get_env_vars,
        save_env_var,
        delete_env_var,
        list_pm2,
        pm2_action,
        list_npm_packages,
        install_npm_package,
        uninstall_npm_package,
        run_npm_script
    ]
}
