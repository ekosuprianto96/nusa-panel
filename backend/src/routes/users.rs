//! # User Routes
//!
//! Route handlers untuk user management.

use rocket::serde::json::Json;
use rocket::{delete, get, put, routes, FromForm, Route, State};
use serde::Deserialize;

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::{AdminUser, AuthenticatedUser};
use crate::models::{UpdateUserRequest, UserResponse};
use crate::services::UserService;
use crate::utils::response::{paginated, success, success_message, ApiResponse, PaginatedResponse};

/// Query parameters untuk pagination
#[derive(Debug, Deserialize, FromForm)]
pub struct PaginationParams {
    /// Halaman (default: 1)
    #[field(default = 1)]
    pub page: i64,
    /// Items per halaman (default: 10, max: 100)
    #[field(default = 10)]
    pub per_page: i64,
}

/// DTO untuk update status
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

/// DTO untuk update role
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

/// List all users (Admin only)
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Query Parameters
/// - page: Page number (default: 1)
/// - per_page: Items per page (default: 10, max: 100)
///
/// # Returns
/// Paginated list of users
#[get("/?<params..>")]
pub async fn list_users(
    db: &State<Database>,
    _admin: AdminUser,
    params: PaginationParams,
) -> ApiResult<Json<PaginatedResponse<UserResponse>>> {
    let per_page = params.per_page.min(100).max(1);
    let page = params.page.max(1);

    let (users, total) = UserService::get_all(db.get_pool(), page, per_page).await?;
    Ok(paginated(users, total, page, per_page))
}

/// Get user by ID
///
/// User dapat melihat data sendiri, admin dapat melihat semua user.
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: User ID
#[get("/<id>")]
pub async fn get_user(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    // Check authorization: user can only view self, admin can view anyone
    if id != user.id && !user.is_admin() {
        return Err(crate::errors::ApiError::Forbidden);
    }

    let user_response = UserService::get_by_id(db.get_pool(), id).await?;
    Ok(success(user_response))
}

/// Update user
///
/// User dapat update data sendiri, admin dapat update semua user.
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: User ID
///
/// # Request Body
/// ```json
/// {
///   "email": "new@example.com",
///   "first_name": "John",
///   "last_name": "Doe"
/// }
/// ```
#[put("/<id>", format = "json", data = "<request>")]
pub async fn update_user(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateUserRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let updated = UserService::update(
        db.get_pool(),
        id,
        request.into_inner(),
        &user.id,
        &user.role,
    )
    .await?;
    Ok(success(updated))
}

/// Delete user (Admin only)
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: User ID
#[delete("/<id>")]
pub async fn delete_user(
    db: &State<Database>,
    _admin: AdminUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    UserService::delete(db.get_pool(), id).await?;
    Ok(success_message("User berhasil dihapus"))
}

/// Update user status (Admin only)
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: User ID
///
/// # Request Body
/// ```json
/// {
///   "status": "active|inactive|blocked"
/// }
/// ```
#[put("/<id>/status", format = "json", data = "<request>")]
pub async fn update_user_status(
    db: &State<Database>,
    _admin: AdminUser,
    id: &str,
    request: Json<UpdateStatusRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let updated = UserService::update_status(db.get_pool(), id, &request.status).await?;
    Ok(success(updated))
}

/// Update user role (Admin only)
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: User ID
///
/// # Request Body
/// ```json
/// {
///   "role": "admin|reseller|user"
/// }
/// ```
#[put("/<id>/role", format = "json", data = "<request>")]
pub async fn update_user_role(
    db: &State<Database>,
    _admin: AdminUser,
    id: &str,
    request: Json<UpdateRoleRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let updated = UserService::update_role(db.get_pool(), id, &request.role).await?;
    Ok(success(updated))
}

/// Get current user resource usage
///
/// Menampilkan penggunaan resource user (disk, bandwidth, dll).
/// Untuk saat ini mengembalikan placeholder data.
#[get("/<id>/usage")]
pub async fn get_user_usage(
    _db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<UserResourceUsage>>> {
    // Check authorization
    if id != user.id && !user.is_admin() {
        return Err(crate::errors::ApiError::Forbidden);
    }

    // TODO: Implement actual resource tracking
    let usage = UserResourceUsage {
        disk_used_mb: 0,
        disk_limit_mb: 10240, // 10GB default
        bandwidth_used_mb: 0,
        bandwidth_limit_mb: 102400, // 100GB default
        domains_count: 0,
        domains_limit: 10,
        databases_count: 0,
        databases_limit: 10,
        email_accounts_count: 0,
        email_accounts_limit: 50,
    };

    Ok(success(usage))
}

/// Resource usage data untuk user
#[derive(Debug, serde::Serialize)]
pub struct UserResourceUsage {
    pub disk_used_mb: i64,
    pub disk_limit_mb: i64,
    pub bandwidth_used_mb: i64,
    pub bandwidth_limit_mb: i64,
    pub domains_count: i32,
    pub domains_limit: i32,
    pub databases_count: i32,
    pub databases_limit: i32,
    pub email_accounts_count: i32,
    pub email_accounts_limit: i32,
}

/// Mendapatkan routes untuk users
pub fn user_routes() -> Vec<Route> {
    routes![
        list_users,
        get_user,
        update_user,
        delete_user,
        update_user_status,
        update_user_role,
        get_user_usage
    ]
}
