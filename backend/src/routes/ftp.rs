//! # FTP Routes
//!
//! Route handlers untuk FTP account management.

use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{
    ChangeFtpPasswordRequest, CreateFtpAccountRequest, FtpAccountResponse, FtpServerInfo,
    UpdateFtpAccountRequest,
};
use crate::services::FtpService;
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// FTP ACCOUNT ENDPOINTS
// ==========================================

/// List user's FTP accounts
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/")]
pub async fn list_ftp_accounts(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<FtpAccountResponse>>>> {
    let accounts = FtpService::get_user_accounts(db.get_pool(), &user.id).await?;
    Ok(success(accounts))
}

/// Get FTP account by ID
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: FTP Account ID
#[get("/<id>")]
pub async fn get_ftp_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<FtpAccountResponse>>> {
    let account = FtpService::get_by_id(db.get_pool(), id, &user.id).await?;
    Ok(success(account))
}

/// Create new FTP account
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "username": "myftpuser",
///   "password": "SecureP@ss123",
///   "home_directory": "/public_html",  // optional
///   "quota_mb": 1024  // optional, 0 = unlimited
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn create_ftp_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateFtpAccountRequest>,
) -> ApiResult<Json<ApiResponse<FtpAccountResponse>>> {
    let account = FtpService::create(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(account))
}

/// Update FTP account
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: FTP Account ID
///
/// # Request Body
/// ```json
/// {
///   "password": "NewP@ssword",  // optional
///   "home_directory": "/new/path",  // optional
///   "is_active": true,  // optional
///   "quota_mb": 2048  // optional
/// }
/// ```
#[put("/<id>", format = "json", data = "<request>")]
pub async fn update_ftp_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateFtpAccountRequest>,
) -> ApiResult<Json<ApiResponse<FtpAccountResponse>>> {
    let account =
        FtpService::update(db.get_pool(), id, &user.id, request.into_inner()).await?;
    Ok(success(account))
}

/// Change FTP password
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: FTP Account ID
///
/// # Request Body
/// ```json
/// {
///   "new_password": "NewSecureP@ss456"
/// }
/// ```
#[put("/<id>/password", format = "json", data = "<request>")]
pub async fn change_ftp_password(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<ChangeFtpPasswordRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    FtpService::change_password(db.get_pool(), id, &user.id, request.into_inner()).await?;
    Ok(success_message("Password FTP berhasil diubah"))
}

/// Toggle FTP account status
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: FTP Account ID
#[put("/<id>/toggle")]
pub async fn toggle_ftp_status(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<FtpAccountResponse>>> {
    let account = FtpService::toggle_status(db.get_pool(), id, &user.id).await?;
    Ok(success(account))
}

/// Delete FTP account
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: FTP Account ID
#[delete("/<id>")]
pub async fn delete_ftp_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    FtpService::delete(db.get_pool(), id, &user.id).await?;
    Ok(success_message("FTP account berhasil dihapus"))
}

/// Get FTP server info
///
/// Returns FTP server connection information
#[get("/server-info")]
pub async fn get_ftp_server_info(
    _user: AuthenticatedUser,
) -> Json<ApiResponse<FtpServerInfo>> {
    success(FtpService::get_server_info())
}

/// Mendapatkan routes untuk FTP accounts
pub fn ftp_routes() -> Vec<Route> {
    routes![
        list_ftp_accounts,
        get_ftp_account,
        create_ftp_account,
        update_ftp_account,
        change_ftp_password,
        toggle_ftp_status,
        delete_ftp_account,
        get_ftp_server_info
    ]
}
