//! # Package Routes
//!
//! Route handlers untuk package management (Admin only).

use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AdminUser;
use crate::models::{CreatePackageRequest, PackageResponse, UpdatePackageRequest};
use crate::services::PackageService;
use crate::utils::response::{success, success_message, ApiResponse};

/// List all packages
///
/// Returns all active packages. Admin can see all (including inactive).
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/")]
pub async fn list_packages(
    db: &State<Database>,
    admin: Option<AdminUser>,
) -> ApiResult<Json<ApiResponse<Vec<PackageResponse>>>> {
    // Admin dapat melihat inactive packages
    let include_inactive = admin.is_some();
    let packages = PackageService::get_all(db.get_pool(), include_inactive).await?;
    Ok(success(packages))
}

/// Get package by ID
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Package ID
#[get("/<id>")]
pub async fn get_package(
    db: &State<Database>,
    id: &str,
) -> ApiResult<Json<ApiResponse<PackageResponse>>> {
    let package = PackageService::get_by_id(db.get_pool(), id).await?;
    Ok(success(package))
}

/// Get default package
///
/// Returns the default package for new users
#[get("/default")]
pub async fn get_default_package(
    db: &State<Database>,
) -> ApiResult<Json<ApiResponse<PackageResponse>>> {
    let package = PackageService::get_default(db.get_pool()).await?;
    Ok(success(package))
}

/// Create new package (Admin only)
///
/// # Headers
/// - Authorization: Bearer <access_token> (Admin)
///
/// # Request Body
/// ```json
/// {
///   "name": "Basic",
///   "description": "Basic hosting package",
///   "disk_quota_mb": 1024,
///   "bandwidth_mb": 10240,
///   "max_domains": 1,
///   "max_databases": 1,
///   "price_monthly": 25000
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn create_package(
    db: &State<Database>,
    _admin: AdminUser,
    request: Json<CreatePackageRequest>,
) -> ApiResult<Json<ApiResponse<PackageResponse>>> {
    let package = PackageService::create(db.get_pool(), request.into_inner()).await?;
    Ok(success(package))
}

/// Update package (Admin only)
///
/// # Headers
/// - Authorization: Bearer <access_token> (Admin)
///
/// # Path Parameters
/// - id: Package ID
///
/// # Request Body
/// All fields are optional
#[put("/<id>", format = "json", data = "<request>")]
pub async fn update_package(
    db: &State<Database>,
    _admin: AdminUser,
    id: &str,
    request: Json<UpdatePackageRequest>,
) -> ApiResult<Json<ApiResponse<PackageResponse>>> {
    let package = PackageService::update(db.get_pool(), id, request.into_inner()).await?;
    Ok(success(package))
}

/// Delete package (Admin only)
///
/// Cannot delete if users are assigned to this package.
///
/// # Headers
/// - Authorization: Bearer <access_token> (Admin)
///
/// # Path Parameters
/// - id: Package ID
#[delete("/<id>")]
pub async fn delete_package(
    db: &State<Database>,
    _admin: AdminUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    PackageService::delete(db.get_pool(), id).await?;
    Ok(success_message("Package berhasil dihapus"))
}

/// Get routes for packages
pub fn package_routes() -> Vec<Route> {
    routes![
        list_packages,
        get_default_package,  // Must come before get_package to avoid matching "default" as id
        get_package,
        create_package,
        update_package,
        delete_package
    ]
}
