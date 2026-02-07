//! # Authentication Guard
//!
//! Request guard untuk autentikasi JWT.
//! Memvalidasi token dan menyediakan user info ke route handlers.

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::errors::ApiError;
use crate::utils::jwt::{extract_token_from_header, validate_access_token, Claims};

const ACCESS_TOKEN_COOKIE: &str = "access_token";

/// Authenticated user dari JWT token
///
/// Guard ini digunakan untuk route yang memerlukan autentikasi.
/// Jika token valid, informasi user akan tersedia di handler.
///
/// # Example
/// ```rust
/// #[get("/profile")]
/// async fn get_profile(user: AuthenticatedUser) -> Json<UserProfile> {
///     // user.id, user.username, dll tersedia di sini
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// User ID dari token
    pub id: String,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Role user
    pub role: String,
}

impl From<Claims> for AuthenticatedUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            username: claims.username,
            email: claims.email,
            role: claims.role,
        }
    }
}

impl AuthenticatedUser {
    /// Cek apakah user adalah admin
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Cek apakah user adalah reseller atau admin
    pub fn is_reseller_or_above(&self) -> bool {
        self.role == "admin" || self.role == "reseller"
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ApiError;

    /// Implementasi request guard
    ///
    /// Mengambil token dari header Authorization,
    /// memvalidasi, dan mengekstrak informasi user.
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Ambil Authorization header
        let auth_header = request.headers().get_one("Authorization");

        // DEBUG: Print all cookies
        let all_cookies: Vec<_> = request.cookies().iter().map(|c| c.name().to_string()).collect();
        tracing::info!("DEBUG: Incoming Cookies: {:?}", all_cookies);
        if let Some(c) = request.cookies().get(ACCESS_TOKEN_COOKIE) {
             tracing::info!("DEBUG: Found access_token cookie: {}...", &c.value()[..10.min(c.value().len())]);
        } else {
             tracing::info!("DEBUG: access_token cookie NOT found in request");
        }

        let token = match auth_header {
             Some(header) => match extract_token_from_header(header) {
                Some(token) => Some(token.to_string()),
                None => {
                    tracing::debug!("Invalid Authorization header format");
                    return Outcome::Error((Status::Unauthorized, ApiError::InvalidToken));
                }
            },
            None => request
                .cookies()
                .get(ACCESS_TOKEN_COOKIE)
                .map(|cookie| cookie.value().to_string()),
        };

        match token {
            Some(token) => match validate_access_token(&token) {
                Ok(claims) => Outcome::Success(AuthenticatedUser::from(claims)),
                Err(e) => {
                    tracing::debug!("Token validation failed: {:?}", e);
                    Outcome::Error((Status::Unauthorized, ApiError::InvalidToken))
                }
            },
            None => {
                tracing::debug!("Missing Authorization header and access token cookie");
                Outcome::Error((Status::Unauthorized, ApiError::MissingToken))
            }
        }
    }
}

/// Admin-only guard
///
/// Guard ini hanya mengizinkan user dengan role Admin.
///
/// # Example
/// ```rust
/// #[get("/admin/users")]
/// async fn list_all_users(admin: AdminUser) -> Json<Vec<User>> {
///     // Hanya admin yang bisa akses
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthenticatedUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // First, authenticate as regular user
        let user = match AuthenticatedUser::from_request(request).await {
            Outcome::Success(user) => user,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        // Check if user is admin
        if user.is_admin() {
            Outcome::Success(AdminUser(user))
        } else {
            tracing::debug!("User {} attempted admin access", user.username);
            Outcome::Error((Status::Forbidden, ApiError::Forbidden))
        }
    }
}

/// Reseller or Admin guard
///
/// Guard ini mengizinkan user dengan role Reseller atau Admin.
#[derive(Debug, Clone)]
pub struct ResellerOrAdmin(pub AuthenticatedUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ResellerOrAdmin {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = match AuthenticatedUser::from_request(request).await {
            Outcome::Success(user) => user,
            Outcome::Error(e) => return Outcome::Error(e),
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        if user.is_reseller_or_above() {
            Outcome::Success(ResellerOrAdmin(user))
        } else {
            tracing::debug!("User {} attempted reseller access", user.username);
            Outcome::Error((Status::Forbidden, ApiError::Forbidden))
        }
    }
}

/// Optional authentication guard
///
/// Guard ini tidak memerlukan autentikasi, tapi akan menyediakan
/// informasi user jika token valid. Menggunakan wrapper type alih-alih
/// Option langsung untuk menghindari orphan rules.
///
/// # Example
/// ```rust
/// #[get("/public")]
/// async fn public_endpoint(user: MaybeUser) -> Json<Response> {
///     if let Some(user) = user.0 {
///         // User terautentikasi
///     } else {
///         // User anonymous
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MaybeUser(pub Option<AuthenticatedUser>);

impl MaybeUser {
    /// Cek apakah user terautentikasi
    pub fn is_authenticated(&self) -> bool {
        self.0.is_some()
    }

    /// Mendapatkan user jika terautentikasi
    pub fn user(&self) -> Option<&AuthenticatedUser> {
        self.0.as_ref()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MaybeUser {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthenticatedUser::from_request(request).await {
            Outcome::Success(user) => Outcome::Success(MaybeUser(Some(user))),
            _ => Outcome::Success(MaybeUser(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user_roles() {
        let admin = AuthenticatedUser {
            id: "1".to_string(),
            username: "admin".to_string(),
            email: "admin@test.com".to_string(),
            role: "admin".to_string(),
        };
        assert!(admin.is_admin());
        assert!(admin.is_reseller_or_above());

        let reseller = AuthenticatedUser {
            id: "2".to_string(),
            username: "reseller".to_string(),
            email: "reseller@test.com".to_string(),
            role: "reseller".to_string(),
        };
        assert!(!reseller.is_admin());
        assert!(reseller.is_reseller_or_above());

        let user = AuthenticatedUser {
            id: "3".to_string(),
            username: "user".to_string(),
            email: "user@test.com".to_string(),
            role: "user".to_string(),
        };
        assert!(!user.is_admin());
        assert!(!user.is_reseller_or_above());
    }

    #[test]
    fn test_maybe_user() {
        let authenticated = MaybeUser(Some(AuthenticatedUser {
            id: "1".to_string(),
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            role: "user".to_string(),
        }));
        assert!(authenticated.is_authenticated());
        assert!(authenticated.user().is_some());

        let anonymous = MaybeUser(None);
        assert!(!anonymous.is_authenticated());
        assert!(anonymous.user().is_none());
    }
}
