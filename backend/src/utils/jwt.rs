//! # JWT Utilities
//!
//! Modul untuk pembuatan dan validasi JSON Web Tokens.
//! Menggunakan jsonwebtoken crate dengan algoritma HS256.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::CONFIG;
use crate::errors::ApiError;

/// Tipe token JWT
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    /// Access token untuk autentikasi API
    Access,
    /// Refresh token untuk memperpanjang session
    Refresh,
}

/// JWT Claims
///
/// Struktur data yang di-encode ke dalam JWT token.
/// Berisi informasi user dan metadata token.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Username
    pub username: String,

    /// Email user
    pub email: String,

    /// Role user (admin, reseller, user)
    pub role: String,

    /// Tipe token (access atau refresh)
    pub token_type: TokenType,

    /// Issued at (waktu pembuatan)
    pub iat: i64,

    /// Expiration time
    pub exp: i64,

    /// Not before (token valid setelah waktu ini)
    pub nbf: i64,

    /// Issuer
    pub iss: String,

    /// JWT ID (unique identifier untuk token)
    pub jti: String,
}

/// Payload untuk membuat token
#[derive(Debug)]
pub struct TokenPayload {
    /// User ID
    pub user_id: Uuid,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Role
    pub role: String,
}

/// Response setelah login berhasil
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: String,
    /// Tipe token (Bearer)
    pub token_type: String,
    /// Waktu expired access token dalam detik
    pub expires_in: u64,
}

/// Membuat access token baru
///
/// # Arguments
/// * `payload` - Data user untuk di-encode ke token
///
/// # Returns
/// JWT access token string
///
/// # Errors
/// Returns ApiError jika gagal membuat token
pub fn create_access_token(payload: &TokenPayload) -> Result<String, ApiError> {
    let now = Utc::now();
    let expiration = now + Duration::seconds(CONFIG.jwt.expiration as i64);

    let claims = Claims {
        sub: payload.user_id.to_string(),
        username: payload.username.clone(),
        email: payload.email.clone(),
        role: payload.role.clone(),
        token_type: TokenType::Access,
        iat: now.timestamp(),
        exp: expiration.timestamp(),
        nbf: now.timestamp(),
        iss: CONFIG.jwt.issuer.clone(),
        jti: Uuid::new_v4().to_string(),
    };

    encode_token(&claims)
}

/// Membuat refresh token baru
///
/// # Arguments
/// * `payload` - Data user untuk di-encode ke token
///
/// # Returns
/// JWT refresh token string
///
/// # Errors
/// Returns ApiError jika gagal membuat token
pub fn create_refresh_token(payload: &TokenPayload) -> Result<String, ApiError> {
    let now = Utc::now();
    let expiration = now + Duration::seconds(CONFIG.jwt.refresh_expiration as i64);

    let claims = Claims {
        sub: payload.user_id.to_string(),
        username: payload.username.clone(),
        email: payload.email.clone(),
        role: payload.role.clone(),
        token_type: TokenType::Refresh,
        iat: now.timestamp(),
        exp: expiration.timestamp(),
        nbf: now.timestamp(),
        iss: CONFIG.jwt.issuer.clone(),
        jti: Uuid::new_v4().to_string(),
    };

    encode_token(&claims)
}

/// Membuat pasangan access dan refresh token
///
/// # Arguments
/// * `payload` - Data user untuk di-encode
///
/// # Returns
/// TokenPair berisi access dan refresh token
///
/// # Errors
/// Returns ApiError jika gagal membuat token
pub fn create_token_pair(payload: &TokenPayload) -> Result<TokenPair, ApiError> {
    Ok(TokenPair {
        access_token: create_access_token(payload)?,
        refresh_token: create_refresh_token(payload)?,
        token_type: "Bearer".to_string(),
        expires_in: CONFIG.jwt.expiration,
    })
}

/// Encode claims menjadi JWT token
fn encode_token(claims: &Claims) -> Result<String, ApiError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(CONFIG.jwt.secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to encode JWT: {}", e);
        ApiError::InternalError("Failed to create token".to_string())
    })
}

/// Validasi dan decode JWT token
///
/// # Arguments
/// * `token` - JWT token string
///
/// # Returns
/// TokenData berisi Claims jika valid
///
/// # Errors
/// Returns ApiError jika token tidak valid atau expired
pub fn validate_token(token: &str) -> Result<TokenData<Claims>, ApiError> {
    let mut validation = Validation::default();
    validation.set_issuer(&[&CONFIG.jwt.issuer]);
    validation.validate_exp = true;
    validation.validate_nbf = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(CONFIG.jwt.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        tracing::debug!("Token validation failed: {}", e);
        ApiError::InvalidToken
    })
}

/// Validasi access token secara spesifik
///
/// Memastikan token adalah tipe Access, bukan Refresh.
///
/// # Arguments
/// * `token` - JWT token string
///
/// # Returns
/// Claims jika valid
pub fn validate_access_token(token: &str) -> Result<Claims, ApiError> {
    let token_data = validate_token(token)?;

    if token_data.claims.token_type != TokenType::Access {
        return Err(ApiError::InvalidToken);
    }

    Ok(token_data.claims)
}

/// Validasi refresh token secara spesifik
///
/// Memastikan token adalah tipe Refresh, bukan Access.
///
/// # Arguments
/// * `token` - JWT token string
///
/// # Returns
/// Claims jika valid
pub fn validate_refresh_token(token: &str) -> Result<Claims, ApiError> {
    let token_data = validate_token(token)?;

    if token_data.claims.token_type != TokenType::Refresh {
        return Err(ApiError::InvalidToken);
    }

    Ok(token_data.claims)
}

/// Extract token dari Authorization header
///
/// Mengambil token dari format "Bearer <token>".
///
/// # Arguments
/// * `auth_header` - Authorization header value
///
/// # Returns
/// Token string tanpa prefix "Bearer "
pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
    if auth_header.starts_with("Bearer ") {
        Some(&auth_header[7..])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_payload() -> TokenPayload {
        TokenPayload {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
        }
    }

    #[test]
    fn test_create_access_token() {
        let payload = create_test_payload();
        let token = create_access_token(&payload);
        assert!(token.is_ok());
    }

    #[test]
    fn test_create_token_pair() {
        let payload = create_test_payload();
        let pair = create_token_pair(&payload);
        assert!(pair.is_ok());
        let pair = pair.unwrap();
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
    }

    #[test]
    fn test_extract_token_from_header() {
        assert_eq!(
            extract_token_from_header("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(extract_token_from_header("abc123"), None);
    }
}
