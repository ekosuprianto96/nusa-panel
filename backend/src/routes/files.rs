//! # File Routes
//!
//! Route handlers untuk file management.

use rocket::form::FromForm;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};
use serde::Deserialize;

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{
    CopyRequest, CreateFileRequest, DeleteRequest, FileContentResponse, FileInfo,
    FileListResponse, MoveRequest, RenameRequest, WriteFileRequest,
};
use crate::services::FileService;
use crate::utils::response::{success, success_message, ApiResponse};

/// Query parameters untuk list files
#[derive(Debug, Deserialize, FromForm)]
pub struct ListFilesParams {
    /// Path relatif dari user home
    pub path: Option<String>,
    /// Tampilkan hidden files
    #[field(default = false)]
    pub show_hidden: bool,
}

/// Request body untuk read file
#[derive(Debug, Deserialize)]
pub struct ReadFileBody {
    pub path: String,
}

// ==========================================
// FILE ENDPOINTS
// ==========================================

/// List files in directory
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Query Parameters
/// - path: Path relatif dari user home (default: root)
/// - show_hidden: Tampilkan hidden files (default: false)
#[get("/?<params..>")]
pub async fn list_files(
    _db: &State<Database>,
    user: AuthenticatedUser,
    params: ListFilesParams,
) -> ApiResult<Json<ApiResponse<FileListResponse>>> {
    let response =
        FileService::list_files(&user.username, params.path.as_deref(), params.show_hidden).await?;
    Ok(success(response))
}

/// Get file content
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Query Parameters
/// - path: Path to file
#[get("/content?<path>")]
pub async fn get_file_content(
    _db: &State<Database>,
    user: AuthenticatedUser,
    path: String,
) -> ApiResult<Json<ApiResponse<FileContentResponse>>> {
    let response = FileService::read_file(&user.username, &path).await?;
    Ok(success(response))
}

/// Create file or directory
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "path": "/path/to/parent",
///   "name": "new_file.txt",
///   "file_type": "file",  // or "directory"
///   "content": "file content here"  // optional, for files only
/// }
/// ```
#[post("/create", format = "json", data = "<request>")]
pub async fn create_file(
    _db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateFileRequest>,
) -> ApiResult<Json<ApiResponse<FileInfo>>> {
    let file = FileService::create(&user.username, request.into_inner()).await?;
    Ok(success(file))
}

/// Write file content
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "path": "/path/to/file.txt",
///   "content": "new content",
///   "encoding": "utf-8",  // or "base64"
///   "create_if_not_exists": false
/// }
/// ```
#[put("/content", format = "json", data = "<request>")]
pub async fn write_file_content(
    _db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<WriteFileRequest>,
) -> ApiResult<Json<ApiResponse<FileInfo>>> {
    let file = FileService::write_file(&user.username, request.into_inner()).await?;
    Ok(success(file))
}

/// Rename file or directory
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "path": "/current/path/file.txt",
///   "new_name": "new_name.txt"
/// }
/// ```
#[put("/rename", format = "json", data = "<request>")]
pub async fn rename_file(
    _db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<RenameRequest>,
) -> ApiResult<Json<ApiResponse<FileInfo>>> {
    let file = FileService::rename(&user.username, request.into_inner()).await?;
    Ok(success(file))
}

/// Copy file or directory
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "source": "/path/to/source",
///   "destination": "/path/to/destination",
///   "overwrite": false
/// }
/// ```
#[post("/copy", format = "json", data = "<request>")]
pub async fn copy_file(
    _db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CopyRequest>,
) -> ApiResult<Json<ApiResponse<FileInfo>>> {
    let file = FileService::copy(&user.username, request.into_inner()).await?;
    Ok(success(file))
}

/// Move file or directory
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "source": "/path/to/source",
///   "destination": "/path/to/destination",
///   "overwrite": false
/// }
/// ```
#[post("/move", format = "json", data = "<request>")]
pub async fn move_file(
    _db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<MoveRequest>,
) -> ApiResult<Json<ApiResponse<FileInfo>>> {
    let file = FileService::move_file(&user.username, request.into_inner()).await?;
    Ok(success(file))
}

/// Delete file or directory
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "path": "/path/to/delete",
///   "recursive": false  // required true for non-empty directories
/// }
/// ```
#[post("/delete", format = "json", data = "<request>")]
pub async fn delete_file(
    _db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<DeleteRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    FileService::delete(&user.username, request.into_inner()).await?;
    Ok(success_message("File/directory berhasil dihapus"))
}

/// Mendapatkan routes untuk files
pub fn file_routes() -> Vec<Route> {
    routes![
        list_files,
        get_file_content,
        create_file,
        write_file_content,
        rename_file,
        copy_file,
        move_file,
        delete_file
    ]
}
