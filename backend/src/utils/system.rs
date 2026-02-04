use std::process::{Command, Stdio};
use std::io::Write;
use crate::errors::{ApiError, ApiResult};

/// Ensure Linux system user exists for the given panel user.
/// If not, it creates it.
/// 
/// # Arguments
/// * `username` - The system username (e.g., u_123456)
/// * `password` - The user's password (for setting system password)
pub fn ensure_system_user(username: &str, password: &str) -> ApiResult<()> {
    // 1. Check if user exists
    let check_output = Command::new("id")
        .arg(username)
        .output()
        .map_err(|e| ApiError::InternalError(format!("Failed to execute id command: {}", e)))?;

    if check_output.status.success() {
        return Ok(()); // User already exists
    }

    // 2. Create user if not exists
    // useradd -m -s /bin/bash -d /home/username username
    let output = Command::new("sudo")
        .arg("useradd")
        .arg("-m")
        .arg("-s").arg("/bin/bash")
        .arg("-d").arg(format!("/home/{}", username))
        .arg(username)
        .output()
        .map_err(|e| ApiError::InternalError(format!("Failed to execute useradd: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::InternalError(format!("Failed to create system user '{}': {}. Make sure backend is running with sudo privileges or passwordless sudo is configured.", username, error_msg.trim())));
    }

    // 3. Set Password
    // echo "username:password" | sudo chpasswd
    let mut child = Command::new("sudo")
        .arg("chpasswd")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ApiError::InternalError(format!("Failed to spawn chpasswd: {}", e)))?;

    if let Some(mut stdin) = child.stdin.take() {
        let input = format!("{}:{}", username, password);
        stdin.write_all(input.as_bytes())
            .map_err(|e| ApiError::InternalError(format!("Failed to write to chpasswd: {}", e)))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| ApiError::InternalError(format!("Failed to wait for chpasswd: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
         return Err(ApiError::InternalError(format!("Failed to set system password: {}", error_msg)));
    }

    // 4. Create Directory Skeleton
    let directories = vec![
        ("public_html", "755"),
        ("logs", "700"),
        ("backups", "700"),
        ("tmp", "700"),
    ];

    for (dir, perms) in directories {
        let path = format!("/home/{}/{}", username, dir);
        
        // mkdir -p path
        let output = Command::new("sudo")
            .arg("mkdir")
            .arg("-p")
            .arg(&path)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to spawn mkdir for {}: {}", path, e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
             return Err(ApiError::InternalError(format!("Failed to create directory {}: {}", path, error_msg.trim())));
        }

        // chown username:username path
        let output = Command::new("sudo")
            .arg("chown")
            .arg(format!("{}:{}", username, username))
            .arg(&path)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to spawn chown for {}: {}", path, e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
             return Err(ApiError::InternalError(format!("Failed to set ownership for {}: {}", path, error_msg.trim())));
        }

        // chmod perms path
        let output = Command::new("sudo")
            .arg("chmod")
            .arg(perms)
            .arg(&path)
            .output()
            .map_err(|e| ApiError::InternalError(format!("Failed to spawn chmod for {}: {}", path, e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
             return Err(ApiError::InternalError(format!("Failed to set permissions for {}: {}", path, error_msg.trim())));
        }
    }

    // 5. Ensure Home Directory Permissions (755 for Nginx access)
    // chmod 755 /home/username
    let output = Command::new("sudo")
        .arg("chmod")
        .arg("755")
        .arg(format!("/home/{}", username))
        .output()
        .map_err(|e| ApiError::InternalError(format!("Failed to spawn chmod for home: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::InternalError(format!("Failed to set home permissions: {}", error_msg.trim())));
    }

    Ok(())
}

/// Update system user password using chpasswd
pub fn update_system_password(username: &str, password: &str) -> ApiResult<()> {
    // echo "username:password" | sudo chpasswd
    let mut child = Command::new("sudo")
        .arg("chpasswd")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ApiError::InternalError(format!("Failed to spawn chpasswd: {}", e)))?;

    if let Some(mut stdin) = child.stdin.take() {
        let input = format!("{}:{}", username, password);
        stdin.write_all(input.as_bytes())
            .map_err(|e| ApiError::InternalError(format!("Failed to write to chpasswd: {}", e)))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| ApiError::InternalError(format!("Failed to wait for chpasswd: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
         return Err(ApiError::InternalError(format!("Failed to set system password: {}", error_msg)));
    }

    Ok(())
}

/// Ensure directory exists with correct ownership using sudo
pub fn ensure_directory(path: &str, owner: &str) -> ApiResult<()> {
    // mkdir -p path
    let output = Command::new("sudo")
        .arg("mkdir")
        .arg("-p")
        .arg(path)
        .output()
        .map_err(|e| ApiError::InternalError(format!("Failed to execute mkdir: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::InternalError(format!("Failed to create directory '{}': {}", path, error_msg.trim())));
    }

    // chown -R owner:owner path
    let output = Command::new("sudo")
        .arg("chown")
        .arg("-R")
        .arg(format!("{}:{}", owner, owner))
        .arg(path)
        .output()
        .map_err(|e| ApiError::InternalError(format!("Failed to execute chown: {}", e)))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::InternalError(format!("Failed to set ownership for '{}': {}", path, error_msg.trim())));
    }

    Ok(())
}
