//! # Response Utilities
//!
//! Helper untuk membuat response API yang konsisten.

use rocket::serde::json::Json;
use serde::Serialize;

/// Struktur response sukses standar
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    /// Apakah request berhasil
    pub success: bool,
    /// Data response (opsional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Pesan (opsional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Struktur response dengan pagination
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    /// Apakah request berhasil
    pub success: bool,
    /// Data items
    pub data: Vec<T>,
    /// Metadata pagination
    pub pagination: Pagination,
}

/// Metadata pagination
#[derive(Debug, Serialize)]
pub struct Pagination {
    /// Total items
    pub total: i64,
    /// Halaman saat ini (1-indexed)
    pub page: i64,
    /// Items per halaman
    pub per_page: i64,
    /// Total halaman
    pub total_pages: i64,
    /// Apakah ada halaman sebelumnya
    pub has_prev: bool,
    /// Apakah ada halaman selanjutnya
    pub has_next: bool,
}

impl Pagination {
    /// Membuat pagination baru
    ///
    /// # Arguments
    /// * `total` - Total items
    /// * `page` - Halaman saat ini (1-indexed)
    /// * `per_page` - Items per halaman
    pub fn new(total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            total,
            page,
            per_page,
            total_pages,
            has_prev: page > 1,
            has_next: page < total_pages,
        }
    }
}

/// Membuat response sukses dengan data
///
/// # Arguments
/// * `data` - Data yang akan dikembalikan
///
/// # Returns
/// Json wrapped ApiResponse
///
/// # Example
/// ```rust
/// success(user)
/// ```
pub fn success<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        message: None,
    })
}

/// Membuat response sukses dengan pesan
///
/// # Arguments
/// * `message` - Pesan sukses
///
/// # Returns
/// Json wrapped ApiResponse
pub fn success_message(message: impl Into<String>) -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: true,
        data: None,
        message: Some(message.into()),
    })
}

/// Membuat response sukses dengan data dan pesan
///
/// # Arguments
/// * `data` - Data yang akan dikembalikan
/// * `message` - Pesan sukses
///
/// # Returns
/// Json wrapped ApiResponse
pub fn success_with_message<T: Serialize>(
    data: T,
    message: impl Into<String>,
) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        message: Some(message.into()),
    })
}

/// Membuat response dengan pagination
///
/// # Arguments
/// * `data` - Vector items
/// * `total` - Total items di database
/// * `page` - Halaman saat ini
/// * `per_page` - Items per halaman
///
/// # Returns
/// Json wrapped PaginatedResponse
pub fn paginated<T: Serialize>(
    data: Vec<T>,
    total: i64,
    page: i64,
    per_page: i64,
) -> Json<PaginatedResponse<T>> {
    Json(PaginatedResponse {
        success: true,
        data,
        pagination: Pagination::new(total, page, per_page),
    })
}

/// Struktur untuk created response
#[derive(Debug, Serialize)]
pub struct CreatedResponse<T: Serialize> {
    /// Apakah request berhasil
    pub success: bool,
    /// Data yang dibuat
    pub data: T,
    /// Pesan
    pub message: String,
}

/// Membuat response untuk resource yang baru dibuat
///
/// # Arguments
/// * `data` - Data resource yang dibuat
/// * `resource_name` - Nama resource (e.g., "User", "Domain")
///
/// # Returns
/// Json wrapped CreatedResponse
pub fn created<T: Serialize>(data: T, resource_name: &str) -> Json<CreatedResponse<T>> {
    Json(CreatedResponse {
        success: true,
        data,
        message: format!("{} berhasil dibuat", resource_name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestData {
        id: i32,
        name: String,
    }

    #[test]
    fn test_success_response() {
        let data = TestData {
            id: 1,
            name: "Test".to_string(),
        };
        let response = success(data);
        assert!(response.success);
        assert!(response.data.is_some());
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(100, 2, 10);
        assert_eq!(pagination.total, 100);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.total_pages, 10);
        assert!(pagination.has_prev);
        assert!(pagination.has_next);
    }
}
