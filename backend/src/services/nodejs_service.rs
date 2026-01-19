use crate::errors::{ApiError, ApiResult};
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodejsStatus {
    pub nvm_installed: bool,
    pub current_version: Option<String>,
    pub installed_versions: Vec<String>,
    pub lts_versions: Vec<String>,
}

pub struct NodejsService;

impl NodejsService {
    /// Get NVM wrapper command string
    /// Sources nvm.sh before running the command
    fn get_nvm_wrapper(username: &str, command: &str) -> String {
        format!(
            "sudo -u {} bash -c 'export NVM_DIR=\"$HOME/.nvm\"; [ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\"; {}'",
            format!("user_{}", username), command
        )
    }

    /// Check if NVM is installed for user
    pub fn is_nvm_installed(username: &str) -> bool {
        let sudo_username = format!("user_{}", username);
        let cmd = format!(
            "sudo -u {} bash -c '[ -d \"$HOME/.nvm\" ] && echo \"yes\" || echo \"no\"'",
            sudo_username
        );
        
        match Command::new("bash").arg("-c").arg(&cmd).output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "yes",
            Err(_) => false,
        }
    }

    /// Check if system user exists
    fn system_user_exists(username: &str) -> bool {
        let system_username = format!("user_{}", username);
        Command::new("id")
            .arg(&system_username)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Create system user if not exists
    fn create_system_user(username: &str) -> ApiResult<()> {
        if Self::system_user_exists(username) {
            return Ok(());
        }

        let system_username = format!("user_{}", username);
        tracing::info!("Creating system user: {}", system_username);

        let output = Command::new("sudo")
            .arg("useradd")
            .arg("-m")
            .arg("-s")
            .arg("/bin/bash")
            .arg(&system_username)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute useradd: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("Failed to create system user {}: {}", system_username, error)));
        }

        // Ensure ownership is correct (just in case directory existed with root)
        let _ = Command::new("chown")
            .arg("-R")
            .arg(format!("{}:{}", system_username, system_username))
            .arg(format!("/home/{}", system_username))
            .output();

        Ok(())
    }

    /// Install NVM for user
    pub fn install_nvm(username: &str) -> ApiResult<()> {
        // Ensure system user exists
        Self::create_system_user(username)?;
        
        let sudo_username = format!("user_{}", username);
        
        // Double check ownership before install (critical fix for Permission Denied)
        let _ = Command::new("chown")
            .arg("-R")
            .arg(format!("{}:{}", sudo_username, sudo_username))
            .arg(format!("/home/{}", sudo_username))
            .output();

        // Installing NVM using the install script
        // We use curl to download and bash to execute
        let sudo_username = format!("user_{}", username);
        let cmd = format!(
            "sudo -u {} bash -c 'curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash'",
            sudo_username
        );

        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute nvm install: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("NVM installation failed: {}", error)));
        }

        Ok(())
    }

    /// Get Node.js status (versions, current, nvm status)
    pub fn get_status(username: &str) -> ApiResult<NodejsStatus> {
        let nvm_installed = Self::is_nvm_installed(username);
        
        if !nvm_installed {
            return Ok(NodejsStatus {
                nvm_installed: false,
                current_version: None,
                installed_versions: vec![],
                lts_versions: vec![],
            });
        }

        // Get installed versions
        let list_cmd = Self::get_nvm_wrapper(username, "nvm list --no-colors");
        let installed_output = Command::new("bash").arg("-c").arg(&list_cmd).output().ok();
        
        let mut installed_versions = Vec::new();
        let mut current_version = None;

        if let Some(output) = installed_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let mut line_trim = line.trim();
                
                if line_trim.is_empty() || line_trim.contains("Date") {
                    continue;
                }

                // Check for current version indicator
                let is_current = line_trim.starts_with("->");
                
                // If it starts with ->, we parse what's after it
                if is_current {
                    line_trim = line_trim.trim_start_matches("->").trim();
                }

                // Filter out aliases.
                // Real versions usually start with 'v' or a number.
                // Aliases start with characters (e.g., 'default', 'lts/*', 'node', 'stable', 'iojs')
                if line_trim.starts_with("default") 
                    || line_trim.starts_with("node") 
                    || line_trim.starts_with("stable") 
                    || line_trim.starts_with("unstable") 
                    || line_trim.starts_with("iojs") 
                    || line_trim.starts_with("lts/") 
                    || line_trim.starts_with("system") {
                    continue;
                }

                // Split by whitespace to get the version part and ignore potential trailing '*' or comments
                let parts: Vec<&str> = line_trim.split_whitespace().collect();
                
                // The version should be the first valid part found
                if let Some(ver_part) = parts.first() {
                     let clean_ver = ver_part.replace("v", "").replace("*", "");
                     
                     // Basic validation: must start with digit
                     if !clean_ver.is_empty() && clean_ver.chars().next().map_or(false, |c| c.is_digit(10)) {
                         if !installed_versions.contains(&clean_ver) {
                             installed_versions.push(clean_ver.clone());
                             
                             if is_current {
                                 current_version = Some(clean_ver);
                             }
                         }
                     }
                }
            }
        }

        // Get LTS versions (simplified for now, ideally we fetch from nvm ls-remote --lts)
        // Since remote fetch can be slow, we might want to cache this or just fetch on demand.
        // For status, let's just return what we have or a specific user-requested list command.
        // For now, let's skip remote fetch in get_status to verify speed, or return empty if separate endpoint needed.
        let lts_versions = vec![]; // We will implement a separate method for remote versions to keep status fast

        Ok(NodejsStatus {
            nvm_installed: true,
            current_version,
            installed_versions,
            lts_versions,
        })
    }

    /// List available LTS versions
    pub fn list_available_lts(username: &str) -> ApiResult<Vec<String>> {
        let cmd = Self::get_nvm_wrapper(username, "nvm ls-remote --lts --no-colors");
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to list remote versions: {}", e)))?;

        if !output.status.success() {
            return Err(ApiError::InternalError("Failed to fetch LTS versions".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut versions = Vec::new();
        
        for line in stdout.lines() {
            let line_trim = line.trim();
            if line_trim.contains("LTS") {
                // "       v20.10.0   (Latest LTS: Iron)"
                // "->      v20.11.0   (Latest LTS: Iron)"
                
                // If line starts with "->", ignore it and take the next part
                let clean_line = if line_trim.starts_with("->") {
                    line_trim.trim_start_matches("->").trim()
                } else {
                    line_trim
                };

                let parts: Vec<&str> = clean_line.split_whitespace().collect();
                if let Some(&ver) = parts.get(0) {
                     let clean_ver = ver.replace("v", "").replace("*", "");
                     // Additional safety check
                     if !clean_ver.is_empty() && clean_ver.chars().next().map_or(false, |c| c.is_digit(10)) {
                        versions.push(clean_ver);
                     }
                }
            }
        }
        
        // Return only last 10 unique major versions or just all? User usually needs latest of each major.
        // Let's reverse to show newest first
        versions.reverse();
        
        Ok(versions)
    }

    /// Install specific Node.js version
    pub fn install_version(username: &str, version: &str) -> ApiResult<()> {
        let cmd = Self::get_nvm_wrapper(username, &format!("nvm install {}", version));
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute install: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("Failed to install version {}: {}", version, error)));
        }
        
        Ok(())
    }

    /// Uninstall specific Node.js version
    pub fn uninstall_version(username: &str, version: &str) -> ApiResult<()> {
        // We need to deactivate before uninstalling if it happens to be the active one
        // Also remove 'v' prefix if present to ensure clean argument
        let clean_version = version.replace("v", "");
        let cmd = Self::get_nvm_wrapper(username, &format!("nvm deactivate && nvm uninstall {}", clean_version));
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute uninstall: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            // NVM uninstall output to stderr even on success sometimes, or specific messages.
            // Check if error contains "N/A: version ... not installed" or similar to ignore?
            // For now, strict check.
            return Err(ApiError::InternalError(format!("Failed to uninstall version {}: {}", version, error)));
        }
        
        Ok(())
    }

    /// Set default Node.js version (alias default)
    pub fn set_default(username: &str, version: &str) -> ApiResult<()> {
        let clean_version = version.replace("v", "");
        let cmd = Self::get_nvm_wrapper(username, &format!("nvm alias default {}", clean_version));
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to set default: {}", e)))?;

        if !output.status.success() {
             let error = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("Failed to set default version {}: {}", version, error)));
        }
        
        Ok(())
    }
}
