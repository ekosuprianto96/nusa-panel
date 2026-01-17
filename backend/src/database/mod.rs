//! # Database Module
//!
//! Modul untuk koneksi dan manajemen database MySQL.
//! Menggunakan SQLx untuk async database operations.

use rocket::fairing::{self, Fairing, Info, Kind};
use rocket::{Build, Rocket};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;

/// Wrapper untuk MySQL connection pool
///
/// Digunakan sebagai Rocket managed state untuk sharing
/// database connections across request handlers.
pub struct Database {
    /// MySQL connection pool
    pub pool: MySqlPool,
}

impl Database {
    /// Membuat database fairing untuk Rocket
    ///
    /// Fairing ini akan menginisialisasi connection pool
    /// saat Rocket startup dan menjalankan migrations.
    ///
    /// # Returns
    /// Database fairing instance
    pub fn fairing() -> DatabaseFairing {
        DatabaseFairing
    }

    /// Mendapatkan reference ke connection pool
    pub fn get_pool(&self) -> &MySqlPool {
        &self.pool
    }
}

/// Rocket Fairing untuk inisialisasi database
pub struct DatabaseFairing;

#[rocket::async_trait]
impl Fairing for DatabaseFairing {
    /// Informasi fairing
    fn info(&self) -> Info {
        Info {
            name: "MySQL Database",
            kind: Kind::Ignite,
        }
    }

    /// Handler saat Rocket ignite
    ///
    /// Menginisialisasi MySQL connection pool dengan konfigurasi
    /// yang dioptimalkan untuk keamanan dan performa.
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        // Ambil DATABASE_URL dari environment
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in environment variables");

        tracing::info!("🔌 Connecting to MySQL database...");

        // Buat connection pool dengan konfigurasi keamanan
        let pool = MySqlPoolOptions::new()
            // Maksimum koneksi dalam pool
            .max_connections(20)
            // Minimum koneksi yang dijaga idle
            .min_connections(5)
            // Timeout untuk mendapatkan koneksi dari pool
            .acquire_timeout(Duration::from_secs(30))
            // Timeout untuk koneksi idle
            .idle_timeout(Duration::from_secs(600))
            // Maximum lifetime sebuah koneksi
            .max_lifetime(Duration::from_secs(1800))
            // Test koneksi sebelum digunakan
            .test_before_acquire(true)
            .connect(&database_url)
            .await;

        match pool {
            Ok(pool) => {
                tracing::info!("✅ Database connection established successfully");

                // Jalankan migrations
                if let Err(e) = run_migrations(&pool).await {
                    tracing::warn!("⚠️  Migration warning (continuing startup): {}", e);
                    // Kita tidak return Err agar server tetap jalan meskipun migrasi "gagal"
                    // (misal karena table sudah ada tapi history migrasi hilang)
                }

                Ok(rocket.manage(Database { pool }))
            }
            Err(e) => {
                tracing::error!("❌ Failed to connect to database: {}", e);
                Err(rocket)
            }
        }
    }
}

/// Menjalankan database migrations
///
/// # Arguments
/// * `pool` - MySQL connection pool
///
/// # Returns
/// Result indicating success or failure
async fn run_migrations(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    tracing::info!("📦 Running database migrations...");

    // Migrations akan dijalankan dari folder migrations/
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    tracing::info!("✅ Migrations completed successfully");
    Ok(())
}

/// Helper trait untuk mendapatkan database dari Rocket state
pub trait DatabaseExt {
    /// Mendapatkan database pool dari request
    fn db(&self) -> &MySqlPool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fairing_info() {
        let fairing = DatabaseFairing;
        let info = fairing.info();
        assert_eq!(info.name, "MySQL Database");
    }
}
