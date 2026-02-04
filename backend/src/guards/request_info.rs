//! # Request Info Guard
//!
//! Menyediakan informasi request sederhana seperti IP dan User-Agent.

use rocket::request::{FromRequest, Outcome, Request};

/// Informasi request yang sering dipakai di audit log
#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestInfo {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let ip_address = request.client_ip().map(|ip| ip.to_string());
        let user_agent = request.headers().get_one("User-Agent").map(|s| s.to_string());
        Outcome::Success(RequestInfo { ip_address, user_agent })
    }
}
