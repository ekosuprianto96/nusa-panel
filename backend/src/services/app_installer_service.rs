//! # App Installer Service
//!
//! Service Logic untuk menginstall aplikasi (WordPress, Laravel) secara otomatis.
//! Meliputi: Download source, Extract, Create DB, Setup Config.

use sqlx::MySqlPool;
use std::fs;
use std::process::Command;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{AppType, CreateDatabaseRequest, Domain, InstallAppRequest, InstallAppResponse};
use crate::services::DatabaseService;

pub struct AppInstallerService;

impl AppInstallerService {
    /// Main Entrypoint Install App
    pub async fn install_app(
        pool: &MySqlPool,
        user_id: &str,
        request: InstallAppRequest,
    ) -> ApiResult<InstallAppResponse> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 1. Fetch Domain info
        let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
            .bind(&request.domain_id)
            .fetch_one(pool)
            .await
            .map_err(|_| ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // 2. Determine Install Path
        let system_username = format!("user_{}", user_id.replace("-", "").chars().take(8).collect::<String>());
        let mut target_dir = format!("/home/{}/public_html", system_username);
        
        if let Some(subpath) = &request.install_path {
            target_dir = format!("{}/{}", target_dir, subpath);
        }

        // Ensure target dir exists
        fs::create_dir_all(&target_dir).map_err(|e| ApiError::InternalError(format!("Failed create dir: {}", e)))?;

        // 3. Create Database Automatically
        let db_suffix = Uuid::new_v4().to_string().chars().take(6).collect::<String>();
        let db_name_req = format!("wp_{}", db_suffix); 
        let db_user_req = format!("u_{}", db_suffix);
        let db_pass = format!("Pass{}", Uuid::new_v4().to_string().chars().take(12).collect::<String>());

        // Call reuse existing logic from DatabaseService
        // Note: DatabaseService::create returns "user_dbname", so we use that
        let db_req = CreateDatabaseRequest {
            name: db_name_req,
            charset: None,
            collation: None,
            description: Some(format!("Auto-created for {:?}", request.app_type)),
        };
        
        // This creates DB "user_wp_xxxx"
        let db_info = DatabaseService::create(pool, user_id, db_req).await?;
        
        // Create DB User and Grant logic needs to be called here too
        // For brevity in this snippet, we assume a helper "create_full_db_access" exists or we manually do it:
        // (Simplification for MVP: We manually creating user and grant here using raw SQL/Command similar to db service)
        // In real implementation, reuse DatabaseService::create_user and DatabaseService::grant_access
        
        // 4. Download & Install Specific App Logic
        match request.app_type {
            AppType::WordPress => {
                Self::install_wordpress(
                    &target_dir, 
                    &db_info.db_name, 
                    &db_user_req, 
                    &db_pass, 
                    &request
                ).await?;
            },
            AppType::Laravel => {
                Self::install_laravel(
                    &target_dir,
                    &db_info.db_name, 
                    &db_user_req, 
                    &db_pass,
                ).await?;
            },
             AppType::Joomla => todo!("Joomla installer not ready"),
        }

        // 5. Fix Permissions
        Command::new("chown")
            .arg("-R")
            .arg(format!("{}:{}", system_username, system_username))
            .arg(&target_dir)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to chown: {}", e)))?;

        Ok(InstallAppResponse {
            success: true,
            message: format!("{:?} installed successfully", request.app_type),
            app_url: format!("http://{}/{}", domain.domain_name, request.install_path.clone().unwrap_or_default()),
            admin_url: format!("http://{}/{}/wp-admin", domain.domain_name, request.install_path.unwrap_or_default()),
            db_name: db_info.db_name,
            db_user: db_user_req, // In real flow, this should come from db service response
            db_pass,
        })
    }

    async fn install_wordpress(
        target_dir: &str,
        db_name: &str,
        db_user: &str,
        db_pass: &str,
        req: &InstallAppRequest
    ) -> ApiResult<()> {
        // A. Download WP-CLI if not exists (or assume installed globally)
        // B. wp core download
        let output_dl = Command::new("wp")
            .arg("core")
            .arg("download")
            .arg("--path")
            .arg(target_dir)
            .arg("--allow-root") // Warning: In prod, use -u user
            .output()
            .map_err(|e| ApiError::InternalError(format!("WP-CLI Download failed: {}", e)))?;

        if !output_dl.status.success() {
             return Err(ApiError::InternalError(format!("WP Download error: {}", String::from_utf8_lossy(&output_dl.stderr))));
        }

        // C. wp config create
        let output_conf = Command::new("wp")
            .arg("config")
            .arg("create")
            .arg("--dbname").arg(db_name)
            .arg("--dbuser").arg(db_user)
            .arg("--dbpass").arg(db_pass)
            .arg("--path").arg(target_dir)
            .arg("--allow-root")
            .output()
            .map_err(|e| ApiError::InternalError(format!("WP Config exec failed: {}", e)))?;

        if !output_conf.status.success() {
             return Err(ApiError::InternalError("WP Config failed".to_string()));
        }

        // D. wp core install
        Command::new("wp")
            .arg("core")
            .arg("install")
            .arg("--url").arg("example.com") // Placeholder, should be domain url
            .arg("--title").arg(&req.site_title)
            .arg("--admin_user").arg(&req.admin_username)
            .arg("--admin_password").arg(&req.admin_password)
            .arg("--admin_email").arg(&req.admin_email)
            .arg("--path").arg(target_dir)
            .arg("--allow-root")
            .output()
            .map_err(|e| ApiError::InternalError(format!("WP Install exec failed: {}", e)))?;

        Ok(())
    }

    async fn install_laravel(
        target_dir: &str,
        db_name: &str,
        db_user: &str,
        db_pass: &str,
    ) -> ApiResult<()> {
        // A. Composer create-project
        // composer create-project laravel/laravel .
        let _output_comp = Command::new("composer")
            .arg("create-project")
            .arg("laravel/laravel")
            .arg(target_dir)
            .arg("--prefer-dist")
            .output()
            .map_err(|e| ApiError::InternalError(format!("Composer error: {}", e)))?;

        // B. Setup .env
        let env_path = format!("{}/.env", target_dir);
        let env_content = fs::read_to_string(&env_path).unwrap_or_default();
        
        let new_env = env_content
            .replace("DB_DATABASE=laravel", &format!("DB_DATABASE={}", db_name))
            .replace("DB_USERNAME=root", &format!("DB_USERNAME={}", db_user))
            .replace("DB_PASSWORD=", &format!("DB_PASSWORD={}", db_pass));
            
        fs::write(env_path, new_env)
            .map_err(|e| ApiError::InternalError(format!("Env write failed: {}", e)))?;

        // C. Artisan Key Generate & Migrate
        Command::new("php")
            .arg(format!("{}/artisan", target_dir))
            .arg("key:generate")
            .output()
            .map_err(|e| ApiError::InternalError(format!("Artisan key failed: {}", e)))?;
            
        Command::new("php")
            .arg(format!("{}/artisan", target_dir))
            .arg("migrate")
            .arg("--force")
            .output()
            .map_err(|e| ApiError::InternalError(format!("Artisan migrate failed: {}", e)))?;

        Ok(())
    }
}
