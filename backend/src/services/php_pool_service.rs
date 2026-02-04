//! # PHP-FPM Pool Service
//!
//! Membuat pool per user untuk isolasi PHP-FPM.

use std::process::{Command, Stdio};
use std::io::Write;

use crate::errors::{ApiError, ApiResult};

pub struct PhpPoolService;

impl PhpPoolService {
    /// Generate pool config content
    fn pool_config(version: &str, system_username: &str) -> String {
        format!(
            r#"[{user}]
user = {user}
group = {user}

listen = /run/php/php-fpm-{version}-{user}.sock
listen.owner = {user}
listen.group = www-data
listen.mode = 0660

pm = ondemand
pm.max_children = 10
pm.process_idle_timeout = 10s
pm.max_requests = 500

; Security
chdir = /
"#,
            user = system_username,
            version = version
        )
    }

    /// Ensure pool file exists for user & php version
    pub fn ensure_user_pool(version: &str, system_username: &str) -> ApiResult<()> {
        let allowed_versions = ["7.4", "8.0", "8.1", "8.2", "8.3"];
        if !allowed_versions.contains(&version) {
            return Err(ApiError::ValidationError(format!(
                "PHP version {} tidak valid",
                version
            )));
        }

        let pool_path = format!("/etc/php/{}/fpm/pool.d/{}.conf", version, system_username);
        let content = Self::pool_config(version, system_username);

        // Write pool config with sudo tee
        let mut child = Command::new("sudo")
            .arg("tee")
            .arg(&pool_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| ApiError::InternalError(format!("Failed to write pool file: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| ApiError::InternalError(format!("Failed to write pool file: {}", e)))?;
        }

        let status = child
            .wait()
            .map_err(|e| ApiError::InternalError(format!("Failed to wait pool write: {}", e)))?;

        if !status.success() {
            return Err(ApiError::InternalError(
                "Failed to create PHP-FPM pool (sudo)".to_string(),
            ));
        }

        // Reload PHP-FPM service for this version
        let service_name = format!("php{}-fpm", version);
        let output = Command::new("sudo")
            .arg("systemctl")
            .arg("reload")
            .arg(&service_name)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to reload {}: {}", service_name, e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!(
                "Failed to reload {}: {}",
                service_name, err
            )));
        }

        Ok(())
    }
}
