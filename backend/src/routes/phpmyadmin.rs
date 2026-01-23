//! # phpMyAdmin Signon Routes
//!
//! Route handlers untuk phpMyAdmin SSO (Single Sign-On).

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::{get, post, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{SignonCredentials, SignonResponse, ValidateTokenRequest};
use crate::services::PhpMyAdminSignonService;
use crate::utils::response::{success, ApiResponse};

// ==========================================
// INTERNAL KEY GUARD
// ==========================================

/// Guard untuk memvalidasi internal key dari phpMyAdmin signon script
pub struct InternalKeyGuard(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for InternalKeyGuard {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("X-Internal-Key") {
            Some(key) => Outcome::Success(InternalKeyGuard(key.to_string())),
            None => Outcome::Error((Status::Unauthorized, "Missing X-Internal-Key header")),
        }
    }
}

// ==========================================
// SIGNON ENDPOINTS
// ==========================================

/// Generate signon token untuk phpMyAdmin
///
/// Endpoint ini dipanggil oleh frontend untuk mendapatkan URL SSO.
/// Token hanya valid selama 30 detik dan hanya bisa digunakan sekali.
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - db_user_id: ID database user yang akan di-login
///
/// # Returns
/// ```json
/// {
///   "success": true,
///   "data": {
///     "signon_url": "https://panel.example.com/phpmyadmin/signon.php?token=xxx",
///     "expires_in": 30
///   }
/// }
/// ```
#[post("/signon/<db_user_id>")]
pub async fn generate_signon_token(
    db: &State<Database>,
    user: AuthenticatedUser,
    db_user_id: &str,
) -> ApiResult<Json<ApiResponse<SignonResponse>>> {
    let response =
        PhpMyAdminSignonService::generate_token(db.get_pool(), db_user_id, &user.id).await?;

    Ok(success(response))
}

/// Validate signon token (dipanggil oleh phpMyAdmin signon script)
///
/// Endpoint internal yang hanya boleh diakses dari localhost.
/// Memerlukan X-Internal-Key header untuk autentikasi.
/// Token akan dihapus setelah validasi (one-time use).
///
/// # Headers
/// - X-Internal-Key: <internal_key>
///
/// # Request Body
/// ```json
/// {
///   "token": "signon-token-uuid"
/// }
/// ```
///
/// # Returns
/// ```json
/// {
///   "user": "db_username",
///   "password": "decrypted_password",
///   "host": "localhost",
///   "port": 3306,
///   "database": "mydb"
/// }
/// ```
#[post("/validate", format = "json", data = "<request>")]
pub async fn validate_signon_token(
    db: &State<Database>,
    internal_key: InternalKeyGuard,
    request: Json<ValidateTokenRequest>,
) -> ApiResult<Json<SignonCredentials>> {
    let credentials = PhpMyAdminSignonService::validate_token(
        db.get_pool(),
        &request.into_inner(),
        &internal_key.0,
    )
    .await?;

    Ok(Json(credentials))
}

/// Get signon service status (untuk monitoring)
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Returns
/// ```json
/// {
///   "success": true,
///   "data": {
///     "active_tokens": 5
///   }
/// }
/// ```
#[get("/status")]
pub async fn signon_status(
    _user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let count = PhpMyAdminSignonService::get_active_token_count();

    Ok(success(serde_json::json!({
        "active_tokens": count
    })))
}

/// Mendapatkan routes untuk phpMyAdmin signon
pub fn phpmyadmin_routes() -> Vec<Route> {
    routes![generate_signon_token, validate_signon_token, signon_status]
}
