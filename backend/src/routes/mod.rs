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
pub mod redis;
pub mod security;
pub mod system;
pub mod users;
pub mod web_server;

pub use apps::*;
pub use auth::*;
pub use databases::*;
pub use domains::*;
pub use emails::*;
pub use files::*;
pub use ftp::*;
pub use health::*;
pub use redis::*;
pub use security::*;
pub use system::*;
pub use users::*;
pub use web_server::*;
