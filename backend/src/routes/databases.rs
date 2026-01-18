//! # Database Routes
//!
//! Route handlers untuk managed database operations.

use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{
    CreateDatabaseRequest, CreateDatabaseUserRequest, DatabaseUserResponse,
    ManagedDatabaseResponse, UpdateDatabaseRequest, UpdateDatabaseUserRequest,
};
use crate::services::DatabaseService;
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// DATABASE ENDPOINTS
// ==========================================

/// List user's databases
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/")]
pub async fn list_databases(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<ManagedDatabaseResponse>>>> {
    let databases = DatabaseService::get_user_databases(db.get_pool(), &user.id).await?;
    Ok(success(databases))
}

/// Get database by ID
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Database ID
#[get("/<id>")]
pub async fn get_database(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<ManagedDatabaseResponse>>> {
    let database = DatabaseService::get_by_id(db.get_pool(), id, &user.id).await?;
    Ok(success(database))
}

/// Create new database
///
/// Membuat database MySQL baru yang terpisah dari database system.
/// Nama database akan di-prefix dengan user ID untuk isolasi.
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "name": "wordpress",
///   "description": "WordPress database",  // optional
///   "charset": "utf8mb4",  // optional
///   "collation": "utf8mb4_unicode_ci"  // optional
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn create_database(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateDatabaseRequest>,
) -> ApiResult<Json<ApiResponse<ManagedDatabaseResponse>>> {
    let database =
        DatabaseService::create(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(database))
}

/// Update database
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Database ID
///
/// # Request Body
/// ```json
/// {
///   "description": "Updated description"
/// }
/// ```
#[put("/<id>", format = "json", data = "<request>")]
pub async fn update_database(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateDatabaseRequest>,
) -> ApiResult<Json<ApiResponse<ManagedDatabaseResponse>>> {
    let database =
        DatabaseService::update(db.get_pool(), id, &user.id, request.into_inner()).await?;
    Ok(success(database))
}

/// Delete database
///
/// WARNING: Ini akan menghapus database MySQL secara permanen!
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Database ID
#[delete("/<id>")]
pub async fn delete_database(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    DatabaseService::delete(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Database berhasil dihapus"))
}

// ==========================================
// DATABASE USER ENDPOINTS
// ==========================================

/// List user's database users
///
/// Database users ini bisa digunakan untuk login ke phpMyAdmin.
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/users")]
pub async fn list_database_users(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<DatabaseUserResponse>>>> {
    let db_users = DatabaseService::get_database_users(db.get_pool(), &user.id).await?;
    Ok(success(db_users))
}

/// Get database user by ID
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Database User ID
#[get("/users/<id>")]
pub async fn get_database_user(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<DatabaseUserResponse>>> {
    let db_user = DatabaseService::get_database_user_by_id(db.get_pool(), id, &user.id).await?;
    Ok(success(db_user))
}

/// Create database user
///
/// User ini bisa digunakan untuk login ke phpMyAdmin.
/// User hanya akan memiliki akses ke database yang ditentukan.
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "username": "wpuser",
///   "password": "SecureP@ss123",
///   "database_id": "database-uuid-here",
///   "host": "%",  // optional, default: % (semua host)
///   "privileges": "ALL"  // optional, default: ALL
/// }
/// ```
#[post("/users", format = "json", data = "<request>")]
pub async fn create_database_user(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateDatabaseUserRequest>,
) -> ApiResult<Json<ApiResponse<DatabaseUserResponse>>> {
    let db_user =
        DatabaseService::create_database_user(db.get_pool(), &user.id, request.into_inner())
            .await?;
    Ok(success(db_user))
}

/// Update database user
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Database User ID
///
/// # Request Body
/// ```json
/// {
///   "password": "NewP@ssword",  // optional
///   "privileges": "SELECT, INSERT",  // optional
///   "is_active": true  // optional
/// }
/// ```
#[put("/users/<id>", format = "json", data = "<request>")]
pub async fn update_database_user(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateDatabaseUserRequest>,
) -> ApiResult<Json<ApiResponse<DatabaseUserResponse>>> {
    let db_user = DatabaseService::update_database_user(
        db.get_pool(),
        id,
        &user.id,
        request.into_inner(),
    )
    .await?;
    Ok(success(db_user))
}

/// Delete database user
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Database User ID
#[delete("/users/<id>")]
pub async fn delete_database_user(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    DatabaseService::delete_database_user(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Database user berhasil dihapus"))
}

/// Mendapatkan routes untuk databases
pub fn database_routes() -> Vec<Route> {
    routes![
        // Database CRUD
        list_databases,
        get_database,
        create_database,
        update_database,
        delete_database,
        // Database Users
        list_database_users,
        get_database_user,
        create_database_user,
        update_database_user,
        delete_database_user
    ]
}
