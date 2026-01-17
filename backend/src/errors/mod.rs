//! # Error Module
//!
//! Custom error types untuk NusaPanel API.
//! Menggunakan thiserror untuk derive Error trait dengan proper error messages.

use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use rocket::Request;
use serde::Serialize;
use thiserror::Error;

/// Struktur response error API
///
/// Format standar untuk semua error response.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Apakah request berhasil
    pub success: bool,
    /// Kode error
    pub error_code: String,
    /// Pesan error yang user-friendly
    pub message: String,
    /// Detail tambahan (opsional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Custom error types untuk aplikasi
///
/// Semua error di aplikasi harus menggunakan tipe ini
/// untuk konsistensi error handling.
#[derive(Error, Debug)]
pub enum ApiError {
    // ==========================================
    // Authentication Errors (401, 403)
    // ==========================================
    
    /// Token JWT tidak valid atau expired
    #[error("Token tidak valid atau sudah kadaluarsa")]
    InvalidToken,

    /// Token JWT tidak ditemukan di header
    #[error("Token autentikasi tidak ditemukan")]
    MissingToken,

    /// Credentials tidak valid (username/password salah)
    #[error("Username atau password salah")]
    InvalidCredentials,

    /// User tidak memiliki akses ke resource
    #[error("Anda tidak memiliki akses ke resource ini")]
    Unauthorized,

    /// User tidak memiliki permission yang diperlukan
    #[error("Anda tidak memiliki izin untuk melakukan aksi ini")]
    Forbidden,

    // ==========================================
    // Validation Errors (400)
    // ==========================================
    
    /// Input validation gagal
    #[error("Validasi input gagal: {0}")]
    ValidationError(String),

    /// Request body tidak valid
    #[error("Request body tidak valid")]
    InvalidRequestBody,

    /// Parameter required tidak ditemukan
    #[error("Parameter required tidak ditemukan: {0}")]
    MissingParameter(String),

    /// Password tidak memenuhi requirement
    #[error("Password harus minimal {0} karakter dan mengandung huruf besar, angka, dan karakter spesial")]
    WeakPassword(usize),

    // ==========================================
    // Resource Errors (404, 409)
    // ==========================================
    
    /// Resource tidak ditemukan
    #[error("{0} tidak ditemukan")]
    NotFound(String),

    /// Resource sudah ada (conflict)
    #[error("{0} sudah ada")]
    AlreadyExists(String),

    /// Resource sedang digunakan dan tidak bisa dihapus
    #[error("{0} sedang digunakan dan tidak bisa dihapus")]
    InUse(String),

    // ==========================================
    // Server Errors (500, 503)
    // ==========================================
    
    /// Database error
    #[error("Terjadi kesalahan pada database")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    InternalError(String),

    /// IO Error
    #[error("System IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Service tidak tersedia
    #[error("Service sedang tidak tersedia, coba lagi nanti")]
    ServiceUnavailable,
    // ==========================================
    // Rate Limiting (429)
    // ==========================================
    
    /// Rate limit exceeded
    #[error("Terlalu banyak request. Coba lagi dalam {0} detik")]
    RateLimitExceeded(u64),

    // ==========================================
    // File Errors
    // ==========================================
    
    /// File tidak ditemukan
    #[error("File tidak ditemukan: {0}")]
    FileNotFound(String),

    /// File terlalu besar
    #[error("Ukuran file melebihi batas maksimum ({0} bytes)")]
    FileTooLarge(u64),

    /// Tipe file tidak diizinkan
    #[error("Tipe file tidak diizinkan: {0}")]
    FileTypeNotAllowed(String),

    /// Permission denied untuk file operation
    #[error("Tidak memiliki izin untuk mengakses file/folder ini")]
    FilePermissionDenied,
}

impl From<String> for ApiError {
    fn from(s: String) -> Self {
        ApiError::InternalError(s)
    }
}

impl ApiError {
    /// Mendapatkan HTTP status code untuk error
    pub fn status_code(&self) -> Status {
        match self {
            // 400 Bad Request
            Self::ValidationError(_)
            | Self::InvalidRequestBody
            | Self::MissingParameter(_)
            | Self::WeakPassword(_) => Status::BadRequest,

            // 401 Unauthorized
            Self::InvalidToken
            | Self::MissingToken
            | Self::InvalidCredentials => Status::Unauthorized,

            // 403 Forbidden
            Self::Unauthorized
            | Self::Forbidden
            | Self::FilePermissionDenied => Status::Forbidden,

            // 404 Not Found
            Self::NotFound(_) | Self::FileNotFound(_) => Status::NotFound,

            // 409 Conflict
            Self::AlreadyExists(_) | Self::InUse(_) => Status::Conflict,

            // 413 Payload Too Large
            Self::FileTooLarge(_) => Status::PayloadTooLarge,

            // 415 Unsupported Media Type
            Self::FileTypeNotAllowed(_) => Status::UnsupportedMediaType,

            // 429 Too Many Requests
            Self::RateLimitExceeded(_) => Status::TooManyRequests,

            // 500 Internal Server Error
            Self::DatabaseError(_) | Self::InternalError(_) | Self::IoError(_) => Status::InternalServerError,

            // 503 Service Unavailable
            Self::ServiceUnavailable => Status::ServiceUnavailable,
        }
    }

    /// Mendapatkan error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidToken => "INVALID_TOKEN",
            Self::MissingToken => "MISSING_TOKEN",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::InvalidRequestBody => "INVALID_REQUEST_BODY",
            Self::MissingParameter(_) => "MISSING_PARAMETER",
            Self::WeakPassword(_) => "WEAK_PASSWORD",
            Self::NotFound(_) => "NOT_FOUND",
            Self::AlreadyExists(_) => "ALREADY_EXISTS",
            Self::InUse(_) => "IN_USE",
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::IoError(_) => "IO_ERROR",
            Self::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            Self::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            Self::FileNotFound(_) => "FILE_NOT_FOUND",
            Self::FileTooLarge(_) => "FILE_TOO_LARGE",
            Self::FileTypeNotAllowed(_) => "FILE_TYPE_NOT_ALLOWED",
            Self::FilePermissionDenied => "FILE_PERMISSION_DENIED",
        }
    }
}

/// Implementasi Responder untuk ApiError
///
/// Memungkinkan ApiError untuk langsung dikembalikan dari route handlers.
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status_code();
        let error_response = ErrorResponse {
            success: false,
            error_code: self.error_code().to_string(),
            message: self.to_string(),
            details: if cfg!(debug_assertions) {
                // Tampilkan detail error hanya di development
                Some(format!("{:?}", self))
            } else {
                None
            },
        };

        // Log error
        tracing::error!(
            error_code = %error_response.error_code,
            message = %error_response.message,
            "API Error occurred"
        );

        Json(error_response).respond_to(request).map(|mut res| {
            res.set_status(status);
            res
        })
    }
}

/// Type alias untuk Result dengan ApiError
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ApiError::InvalidToken.status_code(), Status::Unauthorized);
        assert_eq!(ApiError::Forbidden.status_code(), Status::Forbidden);
        assert_eq!(ApiError::NotFound("User".to_string()).status_code(), Status::NotFound);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(ApiError::InvalidToken.error_code(), "INVALID_TOKEN");
        assert_eq!(ApiError::Forbidden.error_code(), "FORBIDDEN");
    }
}
