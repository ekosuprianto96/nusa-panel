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

    fn user_home(username: &str) -> String {
        format!("/home/user_{}", username)
    }

    fn validate_project_path(username: &str, path: Option<&str>) -> ApiResult<String> {
        let base = Self::user_home(username);
        let candidate = path.unwrap_or(&base);

        if candidate.contains("..") || !candidate.starts_with("/") {
            return Err(ApiError::ValidationError("Invalid path".to_string()));
        }
        if !candidate.starts_with(&base) {
            return Err(ApiError::Forbidden);
        }

        Ok(candidate.to_string())
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

        // If no current version detected, try alias default
        if current_version.is_none() {
            let alias_cmd = Self::get_nvm_wrapper(username, "nvm alias default --no-colors");
            if let Ok(out) = Command::new("bash").arg("-c").arg(&alias_cmd).output() {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // Example: "default -> v20.10.0 (-> v20.10.0)"
                    if let Some(pos) = stdout.find("->") {
                        let rest = stdout[pos + 2..].trim();
                        let ver_part = rest.split_whitespace().next().unwrap_or("");
                        let clean_ver = ver_part.replace("v", "").replace("*", "");
                        if !clean_ver.is_empty() && clean_ver.chars().next().map_or(false, |c| c.is_digit(10)) {
                            current_version = Some(clean_ver.clone());
                            if !installed_versions.contains(&clean_ver) {
                                installed_versions.push(clean_ver);
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
        if !Self::is_nvm_installed(username) {
            return Ok(vec![
                "20.10.0".to_string(),
                "18.18.2".to_string(),
            ]);
        }
        let cmd = Self::get_nvm_wrapper(username, "nvm ls-remote --lts --no-colors");
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to list remote versions: {}", e)))?;

        if !output.status.success() {
            tracing::warn!("Failed to fetch LTS versions, returning defaults");
            return Ok(vec![
                "20.10.0".to_string(),
                "18.18.2".to_string(),
            ]);
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
        
        if versions.is_empty() {
            return Ok(vec![
                "20.10.0".to_string(),
                "18.18.2".to_string(),
            ]);
        }

        // Deduplicate
        let mut uniq: Vec<String> = Vec::new();
        for v in versions {
            if !uniq.contains(&v) {
                uniq.push(v);
            }
        }

        // Sort by semver desc
        uniq.sort_by(|a, b| {
            let pa = a.split('.').map(|s| s.parse::<i32>().unwrap_or(0)).collect::<Vec<_>>();
            let pb = b.split('.').map(|s| s.parse::<i32>().unwrap_or(0)).collect::<Vec<_>>();
            pb.cmp(&pa)
        });

        // Ensure latest per major LTS (18, 20, 22, 24) are included if available
        let mut result: Vec<String> = Vec::new();
        let majors = [24, 22, 20, 18];
        for major in majors {
            if let Some(v) = uniq.iter().find(|v| v.starts_with(&format!("{}.", major))) {
                if !result.contains(v) {
                    result.push(v.clone());
                }
            }
        }

        // Fill with newest remaining up to 10 total
        for v in uniq {
            if result.len() >= 10 {
                break;
            }
            if !result.contains(&v) {
                result.push(v);
            }
        }

        Ok(result)
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
    /// ==========================================
    /// ENV VAR OPERATIONS
    /// ==========================================

    /// Get environment variables from .env file
    pub fn get_env_vars(username: &str, path: &str) -> ApiResult<Vec<String>> {
        // Validate path to prevent directory traversal
        if path.contains("..") || !path.starts_with("/") {
             return Err(ApiError::ValidationError("Invalid path".to_string()));
        }

        let sudo_username = format!("user_{}", username);
        // Check if path belongs to user
        if !path.starts_with(&format!("/home/{}", sudo_username)) {
             return Err(ApiError::Forbidden);
        }

        let cmd = format!(
            "sudo -u {} cat {}/.env",
            sudo_username, path
        );
        
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to read .env: {}", e)))?;

        // If file doesn't exist, return empty
        if !output.status.success() {
            return Ok(Vec::new());
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let mut vars = Vec::new();

        for line in content.lines() {
            if !line.trim().is_empty() && !line.starts_with('#') {
                vars.push(line.to_string());
            }
        }

        Ok(vars)
    }

    /// Save environment variable to .env file
    pub fn save_env_var(username: &str, path: &str, key: &str, value: &str) -> ApiResult<()> {
        // Validate inputs
        if path.contains("..") || !path.starts_with("/") {
             return Err(ApiError::ValidationError("Invalid path".to_string()));
        }
        
        let sudo_username = format!("user_{}", username);
         if !path.starts_with(&format!("/home/{}", sudo_username)) {
             return Err(ApiError::Forbidden);
        }

        // We'll implemented a simple append/replace logic via a shell script helper or manually reading/writing
        // For simplicity and atomic safety, let's use a small python or perline script via bash, or just simple file operations if we were running as the user.
        // Since we are running as root/service, we must use sudo.
        
        // Strategy: Read all, update/add map, write back.
        // Better: Use a helper command (npm install dotenv-cli?) or just sed.
        // Let's use a robust sed or awk approach, or full rewrite.
        
        // Let's go with full rewrite for safety (prevent duplicate keys).
        let current_vars = Self::get_env_vars(username, path)?;
        let mut new_vars_map = std::collections::HashMap::new();
        
        for var in current_vars {
            if let Some((k, v)) = var.split_once('=') {
                new_vars_map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        
        new_vars_map.insert(key.to_string(), value.to_string());
        
        let mut new_content = String::new();
        for (k, v) in new_vars_map {
            new_content.push_str(&format!("{}={}\n", k, v));
        }
        
        // Write back
        // We write to a temp file then move it as the user
        let temp_cmd = format!(
            "echo \"{}\" | sudo -u {} tee {}/.env > /dev/null",
            new_content.replace("\"", "\\\""), sudo_username, path
        );
        
        let output = Command::new("bash")
            .arg("-c")
            .arg(&temp_cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to write .env: {}", e)))?;

        if !output.status.success() {
             return Err(ApiError::InternalError("Failed to save env var".to_string()));
        }

        Ok(())
    }

    /// Delete environment variable
    pub fn delete_env_var(username: &str, path: &str, key: &str) -> ApiResult<()> {
        // Similar logic to save, but remove key
         // Validate paths...
        if path.contains("..") || !path.starts_with("/") {
             return Err(ApiError::ValidationError("Invalid path".to_string()));
        }
         let sudo_username = format!("user_{}", username);
         if !path.starts_with(&format!("/home/{}", sudo_username)) {
             return Err(ApiError::Forbidden);
        }

        let current_vars = Self::get_env_vars(username, path)?;
        let mut new_content = String::new();
        
        for var in current_vars {
            if let Some((k, _)) = var.split_once('=') {
                if k.trim() != key {
                    new_content.push_str(&format!("{}\n", var));
                }
            }
        }

        let temp_cmd = format!(
            "echo \"{}\" | sudo -u {} tee {}/.env > /dev/null",
            new_content.replace("\"", "\\\""), sudo_username, path
        );
        
        let output = Command::new("bash")
            .arg("-c")
            .arg(&temp_cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to write .env: {}", e)))?;

         if !output.status.success() {
             return Err(ApiError::InternalError("Failed to delete env var".to_string()));
        }

        Ok(())
    }

    /// ==========================================
    /// PM2 OPERATIONS
    /// ==========================================

    /// List PM2 processes
    pub fn pm2_list(username: &str) -> ApiResult<Vec<Pm2Process>> {
        let cmd = Self::get_nvm_wrapper(username, "pm2 jlist");
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to list pm2: {}", e)))?;

        if !output.status.success() {
            // Include stderr in error message for better debugging
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If pm2 not found/not running, return empty list? No, probably should allow user to install/setup pm2.
            // But for now, let's error.
            return Err(ApiError::InternalError(format!("PM2 list failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // PM2 jlist returns JSON array
        let processes: Vec<serde_json::Value> = serde_json::from_str(&stdout)
            .map_err(|e| ApiError::InternalError(format!("Failed to parse pm2 json: {}", e)))?;

        let mut result = Vec::new();
        for p in processes {
            let pid = p["pid"].as_u64().unwrap_or(0) as u32;
            let name = p["name"].as_str().unwrap_or("unknown").to_string();
            let pm_id = p["pm_id"].as_u64().unwrap_or(0) as u32;
            let status = p["pm2_env"]["status"].as_str().unwrap_or("stopped").to_string();
            let memory = p["monit"]["memory"].as_u64().unwrap_or(0);
            let cpu = p["monit"]["cpu"].as_f64().unwrap_or(0.0) as f32;
            let uptime = p["pm2_env"]["pm_uptime"].as_u64().unwrap_or(0); // timestamp
            let restarts = p["pm2_env"]["restart_time"].as_u64().unwrap_or(0) as u32;

            result.push(Pm2Process {
                pid,
                name,
                pm_id,
                status,
                memory,
                cpu,
                uptime,
                restarts,
            });
        }

        Ok(result)
    }

    /// PM2 Action (start, stop, restart, delete)
    pub fn pm2_action(username: &str, action: &str, target: &str) -> ApiResult<()> {
        let valid_actions = ["start", "stop", "restart", "delete"];
        if !valid_actions.contains(&action) {
            return Err(ApiError::ValidationError("Invalid action".to_string()));
        }

        // For start, it might be a script path. For others, it's name or id.
        // Assuming target is safe (process name or id).
        // If it's a script, it should be path.
        // This wrapper is simplistic. Real world needs more validation.

        Self::create_system_user(username)?;
        let cmd = format!("pm2 {} {}", action, target);
        let wrapper = Self::get_nvm_wrapper(username, &cmd);
        
        let output = Command::new("bash")
            .arg("-c")
            .arg(&wrapper)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute pm2 action: {}", e)))?;

        if !output.status.success() {
             let error = String::from_utf8_lossy(&output.stderr);
             return Err(ApiError::InternalError(format!("PM2 Action Failed: {}", error)));
        }
        Ok(())
    }

    // ==========================================
    // NPM OPERATIONS
    // ==========================================

    /// List NPM packages (root)
    pub fn npm_list(username: &str) -> ApiResult<Vec<NpmPackage>> {
        Self::create_system_user(username)?;
        Self::npm_list_in_path(username, None)
    }

    pub fn npm_list_in_path(username: &str, path: Option<&str>) -> ApiResult<Vec<NpmPackage>> {
        let path = Self::validate_project_path(username, path)?;
        let cmd = format!("cd {} && npm list --depth=0 --json", path);
        let wrapper = Self::get_nvm_wrapper(username, &cmd);

        let output = Command::new("bash")
            .arg("-c")
            .arg(&wrapper)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute npm list: {}", e)))?;
            
        // NPM list might return non-zero if there are unmet peer deps, but still give JSON
        let json_str = String::from_utf8_lossy(&output.stdout);
        
        // Basic parsing - struct defined below
        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(root) => {
                let mut packages = Vec::new();
                if let Some(deps) = root.get("dependencies").and_then(|d| d.as_object()) {
                    for (name, param) in deps {
                        let version = param.get("version").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                         packages.push(NpmPackage {
                            name: name.clone(),
                            version,
                            r#type: "dependency".to_string(), // Simplified
                        });
                    }
                }
                 // Handle devDependencies if present (though --depth=0 usually mixes them or shows only prod)
                 // This is a simplified implementation.
                Ok(packages)
            }
            Err(_) => {
                // Return empty if parse fails or empty
                Ok(Vec::new())
            }
        }
    }

    /// Install NPM package
    pub fn npm_install(
        username: &str,
        package: Option<&str>,
        version: Option<&str>,
        dev: bool,
        path: Option<&str>,
    ) -> ApiResult<()> {
        Self::create_system_user(username)?;
        let path = Self::validate_project_path(username, path)?;

        let cmd = if let Some(pkg) = package {
            if pkg.trim().is_empty() {
                "npm install".to_string()
            } else {
                let pkg_str = if let Some(v) = version {
                    format!("{}@{}", pkg, v)
                } else {
                    pkg.to_string()
                };
                let flags = if dev { "--save-dev" } else { "--save" };
                format!("npm install {} {}", flags, pkg_str)
            }
        } else {
            "npm install".to_string()
        };

        let wrapper = Self::get_nvm_wrapper(username, &format!("cd {} && {}", path, cmd));
        let output = Command::new("bash")
            .arg("-c")
            .arg(&wrapper)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute npm install: {}", e)))?;

        if !output.status.success() {
             let error = String::from_utf8_lossy(&output.stderr);
             return Err(ApiError::InternalError(format!("NPM Install Failed: {}", error)));
        }
        Ok(())
    }

    /// Uninstall NPM package
    pub fn npm_uninstall(username: &str, package: &str, path: Option<&str>) -> ApiResult<()> {
        Self::create_system_user(username)?;
        let path = Self::validate_project_path(username, path)?;
        let cmd = format!("npm uninstall {}", package);

        let wrapper = Self::get_nvm_wrapper(username, &format!("cd {} && {}", path, cmd));
        let output = Command::new("bash")
            .arg("-c")
            .arg(&wrapper)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute npm uninstall: {}", e)))?;

        if !output.status.success() {
             let error = String::from_utf8_lossy(&output.stderr);
             return Err(ApiError::InternalError(format!("NPM Uninstall Failed: {}", error)));
        }
        Ok(())
    }

    /// Run NPM script (e.g. start, build)
    pub fn npm_run(username: &str, script: &str, path: Option<&str>) -> ApiResult<()> {
        Self::create_system_user(username)?;
        let path = Self::validate_project_path(username, path)?;

        if script.trim().is_empty() {
            return Err(ApiError::ValidationError("Script tidak boleh kosong".to_string()));
        }

        let cmd = format!("npm run {}", script);
        let wrapper = Self::get_nvm_wrapper(username, &format!("cd {} && {}", path, cmd));
        let output = Command::new("bash")
            .arg("-c")
            .arg(&wrapper)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to execute npm run: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::InternalError(format!("NPM Run Failed: {}", error)));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pm2Process {
    pub pid: u32,
    pub name: String,
    pub pm_id: u32,
    pub status: String,
    pub memory: u64,
    pub cpu: f32,
    pub uptime: u64,
    pub restarts: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NpmPackage {
    pub name: String,
    pub version: String,
    pub r#type: String, // dependency, dev-dep
}
