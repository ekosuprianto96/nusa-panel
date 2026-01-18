//! # Health Check Routes
//!
//! Endpoint untuk health check dan status server.

use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use serde::Serialize;

/// Response health check
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Status server
    pub status: String,
    /// Versi API
    pub version: String,
    /// Nama aplikasi
    pub name: String,
}

/// Health check endpoint
#[get("/health")]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "NusaPanel API".to_string(),
    })
}

/// Mendapatkan routes untuk health check
pub fn health_routes() -> Vec<Route> {
    routes![health_check]
}
