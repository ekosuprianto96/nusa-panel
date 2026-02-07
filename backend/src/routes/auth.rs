//! # Authentication Routes
//!
//! Route handlers untuk authentication: register, login, logout, refresh token.

use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::serde::json::Json;
use rocket::time::Duration;
use rocket::{get, post, routes, Route, State};


use crate::config::AppConfig;
use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::{AuthenticatedUser, RequestInfo};
use crate::models::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, RefreshTokenRequest, UserResponse,
};
use crate::services::AuthService;
use crate::utils::jwt::{TokenPair, validate_access_token};
use crate::utils::response::{success, success_message, ApiResponse};
use crate::services::SecurityService;

const ACCESS_TOKEN_COOKIE: &str = "access_token";
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

fn set_auth_cookies(cookies: &CookieJar<'_>, config: &AppConfig, tokens: &TokenPair) {
    // FIXME: Forcing secure=false for debugging context where HTTPS might not be detected/used
    let secure = false; // config.is_production();
    let access_max_age = Duration::seconds(config.jwt.expiration as i64);
    let refresh_max_age = Duration::seconds(config.jwt.refresh_expiration as i64);

    let access_cookie = Cookie::build((ACCESS_TOKEN_COOKIE, tokens.access_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(access_max_age)
        .build();

    let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, tokens.refresh_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(refresh_max_age)
        .build();

    cookies.add(access_cookie);
    cookies.add(refresh_cookie);
}

fn clear_auth_cookies(cookies: &CookieJar<'_>, config: &AppConfig) {
    let secure = config.is_production();

    let access_cookie = Cookie::build((ACCESS_TOKEN_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(Duration::seconds(0))
        .build();

    let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(Duration::seconds(0))
        .build();

    cookies.remove(access_cookie);
    cookies.remove(refresh_cookie);
}

/// Register user baru
///
/// # Request Body
/// ```json
/// {
///   "username": "johndoe",
///   "email": "john@example.com",
///   "password": "SecureP@ss123",
///   "first_name": "John",
///   "last_name": "Doe"
/// }
/// ```
///
/// # Returns
/// UserResponse dengan data user yang baru dibuat
#[post("/register", format = "json", data = "<request>")]
pub async fn register(
    db: &State<Database>,
    request: Json<CreateUserRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let user = AuthService::register(db.get_pool(), request.into_inner()).await?;
    Ok(success(user))
}

/// Login user
///
/// # Request Body
/// ```json
/// {
///   "username_or_email": "johndoe",
///   "password": "SecureP@ss123"
/// }
/// ```
///
/// # Returns
/// TokenPair dengan access dan refresh token
#[post("/login", format = "json", data = "<request>")]
pub async fn login(
    db: &State<Database>,
    config: &State<AppConfig>,
    cookies: &CookieJar<'_>,
    req_info: RequestInfo,
    request: Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<TokenPair>>> {
    let tokens = AuthService::login(db.get_pool(), request.into_inner()).await?;
    set_auth_cookies(cookies, config, &tokens);
    if let Ok(claims) = validate_access_token(&tokens.access_token) {
        SecurityService::log_event(
            db.get_pool(),
            Some(&claims.sub),
            "auth_login_success",
            req_info.ip_address.as_deref(),
            Some(&claims.username),
            Some("success"),
            req_info.user_agent.as_deref(),
        )
        .await
        .ok();
    }
    Ok(success(tokens))
}

/// Refresh access token
///
/// # Request Body
/// ```json
/// {
///   "refresh_token": "your-refresh-token-here"
/// }
/// ```
///
/// # Returns
/// TokenPair baru dengan access dan refresh token
#[post("/refresh", format = "json", data = "<request>")]
pub async fn refresh_token(
    db: &State<Database>,
    config: &State<AppConfig>,
    cookies: &CookieJar<'_>,
    request: Option<Json<RefreshTokenRequest>>,
) -> ApiResult<Json<ApiResponse<TokenPair>>> {
    let refresh_token = match request {
        Some(body) => body.into_inner().refresh_token,
        None => cookies
            .get(REFRESH_TOKEN_COOKIE)
            .map(|cookie| cookie.value().to_string())
            .ok_or(crate::errors::ApiError::MissingToken)?,
    };
    let tokens = AuthService::refresh_token(
        db.get_pool(),
        RefreshTokenRequest { refresh_token },
    )
    .await?;
    set_auth_cookies(cookies, config, &tokens);
    Ok(success(tokens))
}

/// Get current user info
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Returns
/// UserResponse dengan data user yang sedang login
#[get("/me")]
pub async fn get_current_user(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let user_response = crate::services::UserService::get_by_id(db.get_pool(), &user.id).await?;
    Ok(success(user_response))
}

/// Change password
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "current_password": "OldP@ssword123",
///   "new_password": "NewP@ssword456",
///   "confirm_password": "NewP@ssword456"
/// }
/// ```
#[post("/change-password", format = "json", data = "<request>")]
pub async fn change_password(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<ChangePasswordRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    AuthService::change_password(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success_message("Password berhasil diubah"))
}

/// Logout endpoint
///
/// Saat ini hanya mengembalikan pesan sukses karena JWT stateless.
/// Untuk implementasi lengkap, token bisa disimpan di blacklist.
#[post("/logout")]
pub async fn logout(
    _user: AuthenticatedUser,
    config: &State<AppConfig>,
    cookies: &CookieJar<'_>,
) -> Json<ApiResponse<()>> {
    // Note: Dalam implementasi produksi, token sebaiknya
    // ditambahkan ke blacklist untuk invalidasi
    clear_auth_cookies(cookies, config);
    success_message("Logged out successfully")
}

/// Mendapatkan routes untuk authentication
pub fn auth_routes() -> Vec<Route> {
    routes![
        register,
        login,
        refresh_token,
        get_current_user,
        change_password,
        logout
    ]
}
