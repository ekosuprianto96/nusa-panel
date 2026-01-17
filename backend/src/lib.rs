//! # NusaPanel - Web Hosting Control Panel API
//!
//! Panel kontrol web hosting yang dibangun dengan Rust dan Rocket framework.
//! Menyediakan API mirip cPanel untuk manajemen hosting multi-OS.
//!
//! ## Fitur Utama
//! - User Management dengan Role-Based Access Control
//! - Domain & DNS Management
//! - File Management
//! - Database Management (MySQL)
//! - Email Management
//!
//! ## Keamanan
//! - JWT Authentication
//! - Password hashing dengan Argon2
//! - Rate limiting
//! - Input validation

pub mod config;
pub mod database;
pub mod errors;
pub mod guards;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;
