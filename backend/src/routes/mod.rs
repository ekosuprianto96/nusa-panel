//! # Routes Module
//!
//! API route handlers untuk NusaPanel.

pub mod apps;
pub mod auth;
pub mod databases;
pub mod domains;
pub mod emails;
pub mod files;
pub mod ftp;
pub mod health;
pub mod nodejs;
pub mod packages;
pub mod phpmyadmin;
pub mod redis;
pub mod security;
pub mod system;
pub mod users;
pub mod web_server;

use rocket::catch;

pub use apps::*;
pub use auth::*;
pub use databases::*;
pub use domains::*;
pub use emails::*;
pub use files::*;
pub use ftp::*;
pub use health::*;
pub use nodejs::*;
pub use packages::*;
pub use phpmyadmin::*;
pub use redis::*;
pub use security::*;
pub use system::*;
pub use users::*;
pub use web_server::*;

#[catch(403)]
pub fn forbidden_catcher() -> rocket::serde::json::Value {
    rocket::serde::json::json!({
        "success": false,
        "error_code": "FORBIDDEN",
        "message": "The server refused to authorize the request (Catch-all 403)."
    })
}
