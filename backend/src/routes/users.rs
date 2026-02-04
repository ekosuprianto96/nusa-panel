//! # User Routes
//!
//! Route handlers untuk user management.

use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, FromForm, Route, State};
use serde::Deserialize;

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::{AdminUser, AuthenticatedUser};
use crate::models::{
    AdminUserStats, AssignPackageRequest, CreateUserByAdminRequest, UpdateUserRequest,
    UserResponse, UserResourceUsage,
};
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

/// Admin dashboard user stats
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/stats")]
pub async fn get_admin_user_stats(
    db: &State<Database>,
    _admin: AdminUser,
) -> ApiResult<Json<ApiResponse<AdminUserStats>>> {
    let stats = UserService::get_admin_stats(db.get_pool()).await?;
    Ok(success(stats))
}

/// Create user (Admin/Reseller only)
///
/// Admin dapat create user dengan role apapun kecuali admin.
/// Reseller hanya dapat create user dengan role 'user'.
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "username": "newuser",
///   "email": "user@example.com",
///   "password": "securepassword123",
///   "first_name": "John",
///   "last_name": "Doe",
///   "role": "user",
///   "package_id": "pkg-xxxx"
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn create_user(
    db: &State<Database>,
    creator: AuthenticatedUser,
    request: Json<CreateUserByAdminRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let user = UserService::create_by_admin(
        db.get_pool(),
        request.into_inner(),
        &creator.id,
        &creator.role,
    )
    .await?;
    Ok(success(user))
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
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<UserResourceUsage>>> {
    // Check authorization
    if id != user.id && !user.is_admin() {
        return Err(crate::errors::ApiError::Forbidden);
    }

    let usage = crate::services::UserService::get_resource_usage(db.get_pool(), id).await?;

    Ok(success(usage))
}

/// Assign package to user (Admin only)
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
///   "package_id": "pkg-xxxx"
/// }
/// ```
#[put("/<id>/package", format = "json", data = "<request>")]
pub async fn assign_user_package(
    db: &State<Database>,
    _admin: AdminUser,
    id: &str,
    request: Json<AssignPackageRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let user = UserService::assign_package(db.get_pool(), id, request.into_inner()).await?;
    Ok(success(user))
}

/// Mendapatkan routes untuk users
pub fn user_routes() -> Vec<Route> {
    routes![
        list_users,
        get_admin_user_stats,
        create_user,
        get_user,
        update_user,
        delete_user,
        update_user_status,
        update_user_role,
        assign_user_package,
        get_user_usage
    ]
}
