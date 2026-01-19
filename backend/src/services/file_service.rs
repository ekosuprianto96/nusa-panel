//! # File Service
//!
//! Business logic untuk file management operations.
//! WARNING: File operations harus selalu di-sandbox ke user home directory!

use chrono::{TimeZone, Utc};
use std::fs::{self, Metadata};
use std::io::{Read, Write, Seek};
use std::path::{Path, PathBuf};
use validator::Validate;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::config::CONFIG;
use crate::errors::{ApiError, ApiResult};
use crate::models::{
    permissions_to_octal, permissions_to_string,
    CompressRequest, CopyRequest, CreateFileRequest, DeleteRequest, ExtractRequest,
    FileContentResponse, FileInfo, FileListResponse, FileType, MoveRequest, RenameRequest,
    WriteFileRequest, DANGEROUS_EXTENSIONS, TEXT_EXTENSIONS,
};
use crate::utils::system::ensure_directory;

/// Service untuk file operations
pub struct FileService;

impl FileService {
    /// Mendapatkan base path untuk user
    ///
    /// Path ini digunakan sebagai sandbox - user tidak bisa keluar dari direktori ini.
    /// Untuk development, menggunakan /tmp/nusa-panel-users untuk menghindari permission issues.
    fn get_user_base_path(username: &str) -> PathBuf {
        // Development: gunakan /tmp yang pasti writable
        // Production: gunakan CONFIG.file.user_home_base
        // #[cfg(debug_assertions)]
        // let base = "/tmp/nusa-panel-users";
        
        // #[cfg(not(debug_assertions))]
        let base = &CONFIG.file.user_home_base;
        
        PathBuf::from(base).join(format!("user_{}", username))
    }

    /// Resolve dan validasi path
    ///
    /// Memastikan path yang diberikan:
    /// 1. Berada di dalam user home directory (sandbox)
    /// 2. Tidak mengandung path traversal (../)
    /// 3. Merupakan path yang valid
    ///
    /// # Arguments
    /// * `user_id` - ID user
    /// * `relative_path` - Path relatif dari user home
    ///
    /// # Returns
    /// Absolute path yang sudah divalidasi
    fn resolve_path(username: &str, relative_path: &str) -> ApiResult<PathBuf> {
        let base_path = Self::get_user_base_path(username);

        tracing::debug!("Resolving path for user {}: base_path={:?}, relative_path={}", username, base_path, relative_path);

        // Ensure user home directory exists
        if !base_path.exists() {
            tracing::info!("User home directory does not exist, creating: {:?}", base_path);
            
            // Convert PathBuf to str for command
            let path_str = base_path.to_string_lossy();
            
            // Create user home with correct ownership (using system user)
            // Function ensure_directory uses sudo mkdir
            ensure_directory(&path_str, username).map_err(|e| {
                tracing::error!("Failed to create user home directory {:?} for user {}: {}", base_path, username, e);
                e
            })?;
            
            tracing::info!("Created user home directory: {:?}", base_path);
        }


        // Normalize path dan hapus leading/trailing slashes
        let clean_path = relative_path
            .trim()
            .trim_start_matches('/')
            .trim_start_matches('\\');

        // Check for path traversal attempts
        if clean_path.contains("..") {
            tracing::warn!(
                "Path traversal attempt detected for user {}: {}",
                username,
                relative_path
            );
            return Err(ApiError::FilePermissionDenied);
        }

        // Construct full path
        let full_path = if clean_path.is_empty() {
            base_path.clone()
        } else {
            base_path.join(clean_path)
        };

        // Canonicalize and verify it's within base path
        // Note: canonicalize akan fail jika path tidak exist, jadi kita perlu handle ini
        let canonical_base = base_path.canonicalize().unwrap_or(base_path.clone());

        // Untuk path yang belum exist, kita check parent
        if full_path.exists() {
            let canonical_full = full_path.canonicalize().map_err(|_| {
                ApiError::FileNotFound(relative_path.to_string())
            })?;

            if !canonical_full.starts_with(&canonical_base) {
                tracing::warn!(
                    "Path escape attempt for user {}: {}",
                    username,
                    relative_path
                );
                return Err(ApiError::FilePermissionDenied);
            }

            Ok(canonical_full)
        } else {
            // Path belum exist, validasi parent
            if let Some(parent) = full_path.parent() {
                if parent.exists() {
                    let canonical_parent = parent.canonicalize().map_err(|_| {
                        ApiError::FileNotFound(relative_path.to_string())
                    })?;

                    if !canonical_parent.starts_with(&canonical_base) {
                        return Err(ApiError::FilePermissionDenied);
                    }
                }
            }

            Ok(full_path)
        }
    }

    /// Check apakah ekstensi file diizinkan
    fn is_extension_allowed(filename: &str) -> bool {
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match extension {
            Some(ext) => !DANGEROUS_EXTENSIONS.contains(&ext.as_str()),
            None => true,
        }
    }

    /// Check apakah file adalah text file
    fn is_text_file(filename: &str) -> bool {
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match extension {
            Some(ext) => TEXT_EXTENSIONS.contains(&ext.as_str()),
            None => true, // Files tanpa extension dianggap text
        }
    }

    /// Build FileInfo dari path dan metadata
    fn build_file_info(path: &Path, metadata: &Metadata, base_path: &Path) -> FileInfo {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let relative_path = path
            .strip_prefix(base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| name.clone());

        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.file_type().is_symlink() {
            FileType::Symlink
        } else {
            FileType::File
        };

        let extension = if file_type == FileType::File {
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_string())
        } else {
            None
        };

        // Get permissions (platform specific)
        #[cfg(unix)]
        let (permissions, permissions_octal_str) = {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            (permissions_to_string(mode), permissions_to_octal(mode))
        };

        #[cfg(windows)]
        let (permissions, permissions_octal_str) = {
            if metadata.permissions().readonly() {
                ("r--r--r--".to_string(), "444".to_string())
            } else {
                ("rw-rw-rw-".to_string(), "666".to_string())
            }
        };

        // Get modification time
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| Utc.timestamp_opt(d.as_secs() as i64, 0).unwrap())
            .unwrap_or_else(Utc::now);

        let accessed_at = metadata
            .accessed()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| Utc.timestamp_opt(d.as_secs() as i64, 0).unwrap());

        let created_at = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| Utc.timestamp_opt(d.as_secs() as i64, 0).unwrap());

        FileInfo {
            name: name.clone(),
            path: relative_path,
            file_type,
            size: metadata.len(),
            permissions,
            permissions_octal: permissions_octal_str,
            owner: "user".to_string(), // Simplified
            group: "user".to_string(), // Simplified
            modified_at,
            accessed_at,
            created_at,
            extension,
            mime_type: None, // TODO: Implement MIME type detection
            is_hidden: name.starts_with('.'),
        }
    }

    // ==========================================
    // FILE OPERATIONS
    // ==========================================

    /// List files in directory
    pub async fn list_files(
        user_id: &str,
        path: Option<&str>,
        show_hidden: bool,
    ) -> ApiResult<FileListResponse> {
        let base_path = Self::get_user_base_path(user_id);
        let target_path = Self::resolve_path(user_id, path.unwrap_or(""))?;

        // Ensure directory exists
        if !target_path.exists() {
            // Create user home if it doesn't exist
            if target_path == base_path {
                fs::create_dir_all(&target_path).map_err(|e| {
                    tracing::error!("Failed to create user home: {}", e);
                    ApiError::InternalError("Failed to create directory".to_string())
                })?;
            } else {
                return Err(ApiError::FileNotFound(
                    path.unwrap_or("").to_string(),
                ));
            }
        }

        if !target_path.is_dir() {
            return Err(ApiError::ValidationError(
                "Path bukan direktori".to_string(),
            ));
        }

        let mut items = Vec::new();
        let mut total_size = 0u64;

        let entries = fs::read_dir(&target_path).map_err(|e| {
            tracing::error!("Failed to read directory: {}", e);
            ApiError::FilePermissionDenied
        })?;

        for entry in entries {
            let entry = entry.map_err(|_| ApiError::FilePermissionDenied)?;
            let metadata = entry.metadata().map_err(|_| ApiError::FilePermissionDenied)?;

            // Canonicalize base_path to match entry.path() specificities (like \\?\ prefix on Windows)
            let canonical_base = base_path.canonicalize().unwrap_or(base_path.clone());
            let file_info = Self::build_file_info(&entry.path(), &metadata, &canonical_base);

            // Skip hidden files if not requested
            if !show_hidden && file_info.is_hidden {
                continue;
            }

            total_size += file_info.size;
            items.push(file_info);
        }

        // Sort by type (directories first) then by name
        items.sort_by(|a, b| {
            match (&a.file_type, &b.file_type) {
                (FileType::Directory, FileType::Directory) => a.name.cmp(&b.name),
                (FileType::Directory, _) => std::cmp::Ordering::Less,
                (_, FileType::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        let current_path = target_path
            .strip_prefix(&base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let parent_path = if target_path != base_path {
            target_path
                .parent()
                .and_then(|p| p.strip_prefix(&base_path).ok())
                .map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        Ok(FileListResponse {
            current_path,
            parent_path,
            total_items: items.len(),
            total_size,
            items,
        })
    }

    /// Create file or directory
    pub async fn create(user_id: &str, request: CreateFileRequest) -> ApiResult<FileInfo> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Check dangerous extensions
        if !Self::is_extension_allowed(&request.name) {
            return Err(ApiError::FileTypeNotAllowed(request.name));
        }

        let base_path = Self::get_user_base_path(user_id);
        let parent_path = Self::resolve_path(user_id, &request.path)?;
        let full_path = parent_path.join(&request.name);

        // Check if already exists
        if full_path.exists() {
            return Err(ApiError::AlreadyExists("File/Directory".to_string()));
        }

        // Ensure parent exists
        if !parent_path.exists() {
            fs::create_dir_all(&parent_path).map_err(|e| {
                tracing::error!("Failed to create parent directory: {}", e);
                ApiError::InternalError("Failed to create directory".to_string())
            })?;
        }

        match request.file_type.to_lowercase().as_str() {
            "directory" | "dir" | "folder" => {
                fs::create_dir(&full_path).map_err(|e| {
                    tracing::error!("Failed to create directory: {}", e);
                    ApiError::InternalError("Failed to create directory".to_string())
                })?;
            }
            "file" | _ => {
                let mut file = fs::File::create(&full_path).map_err(|e| {
                    tracing::error!("Failed to create file: {}", e);
                    ApiError::InternalError("Failed to create file".to_string())
                })?;

                if let Some(content) = request.content {
                    file.write_all(content.as_bytes()).map_err(|e| {
                        tracing::error!("Failed to write file content: {}", e);
                        ApiError::InternalError("Failed to write file".to_string())
                    })?;
                }
            }
        }

        let metadata = fs::metadata(&full_path).map_err(|_| {
            ApiError::InternalError("Failed to get file metadata".to_string())
        })?;

        tracing::info!("File created: {} by user {}", request.name, user_id);

        Ok(Self::build_file_info(&full_path, &metadata, &base_path))
    }

    /// Read file content
    pub async fn read_file(user_id: &str, path: &str) -> ApiResult<FileContentResponse> {
        let _base_path = Self::get_user_base_path(user_id);
        let full_path = Self::resolve_path(user_id, path)?;

        if !full_path.exists() {
            return Err(ApiError::FileNotFound(path.to_string()));
        }

        if !full_path.is_file() {
            return Err(ApiError::ValidationError(
                "Path bukan file".to_string(),
            ));
        }

        let metadata = fs::metadata(&full_path).map_err(|_| {
            ApiError::FileNotFound(path.to_string())
        })?;

        // Check file size limit (10MB for reading)
        if metadata.len() > 10 * 1024 * 1024 {
            return Err(ApiError::ValidationError(
                "File terlalu besar untuk dibaca (max 10MB)".to_string(),
            ));
        }

        let mut file = fs::File::open(&full_path).map_err(|_| {
            ApiError::FilePermissionDenied
        })?;

        let mut content = Vec::new();
        file.read_to_end(&mut content).map_err(|_| {
            ApiError::FilePermissionDenied
        })?;

        let name = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Check if text file or binary
        let (content_string, encoding) = if Self::is_text_file(&name) {
            match String::from_utf8(content.clone()) {
                Ok(s) => (s, "utf-8".to_string()),
                Err(_) => {
                    // Fallback to base64 for binary content
                    use base64::{engine::general_purpose::STANDARD, Engine};
                    (STANDARD.encode(&content), "base64".to_string())
                }
            }
        } else {
            // Binary file - return as base64
            use base64::{engine::general_purpose::STANDARD, Engine};
            (STANDARD.encode(&content), "base64".to_string())
        };

        Ok(FileContentResponse {
            path: path.to_string(),
            name,
            content: content_string,
            encoding,
            size: metadata.len(),
            mime_type: None, // TODO: Detect MIME type
        })
    }

    /// Write file content
    pub async fn write_file(user_id: &str, request: WriteFileRequest) -> ApiResult<FileInfo> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let base_path = Self::get_user_base_path(user_id);
        let full_path = Self::resolve_path(user_id, &request.path)?;

        // Check if parent exists
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                return Err(ApiError::FileNotFound(
                    "Parent directory does not exist".to_string(),
                ));
            }
        }

        // Check if file exists
        if !full_path.exists() && !request.create_if_not_exists.unwrap_or(false) {
            return Err(ApiError::FileNotFound(request.path.clone()));
        }

        // Decode content if base64
        let content_bytes = match request.encoding.as_deref() {
            Some("base64") => {
                use base64::{engine::general_purpose::STANDARD, Engine};
                STANDARD.decode(&request.content).map_err(|_| {
                    ApiError::ValidationError("Invalid base64 content".to_string())
                })?
            }
            _ => request.content.into_bytes(),
        };

        // Check size limit
        if content_bytes.len() > CONFIG.file.max_upload_size as usize {
            return Err(ApiError::FileTooLarge(CONFIG.file.max_upload_size));
        }

        // Write file
        let mut file = fs::File::create(&full_path).map_err(|e| {
            tracing::error!("Failed to create file: {}", e);
            ApiError::FilePermissionDenied
        })?;

        file.write_all(&content_bytes).map_err(|e| {
            tracing::error!("Failed to write file: {}", e);
            ApiError::InternalError("Failed to write file".to_string())
        })?;

        let metadata = fs::metadata(&full_path).map_err(|_| {
            ApiError::InternalError("Failed to get file metadata".to_string())
        })?;

        tracing::info!("File written: {} by user {}", request.path, user_id);

        Ok(Self::build_file_info(&full_path, &metadata, &base_path))
    }

    /// Delete file or directory
    pub async fn delete(user_id: &str, request: DeleteRequest) -> ApiResult<()> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let full_path = Self::resolve_path(user_id, &request.path)?;

        if !full_path.exists() {
            return Err(ApiError::FileNotFound(request.path.clone()));
        }

        // Prevent deleting user home
        let base_path = Self::get_user_base_path(user_id);
        if full_path == base_path {
            return Err(ApiError::ValidationError(
                "Tidak dapat menghapus home directory".to_string(),
            ));
        }

        if full_path.is_dir() {
            if request.recursive.unwrap_or(false) {
                fs::remove_dir_all(&full_path).map_err(|e| {
                    tracing::error!("Failed to delete directory recursively: {}", e);
                    ApiError::FilePermissionDenied
                })?;
            } else {
                fs::remove_dir(&full_path).map_err(|e| {
                    tracing::error!("Failed to delete directory: {}", e);
                    ApiError::ValidationError(
                        "Directory tidak kosong. Gunakan recursive=true".to_string(),
                    )
                })?;
            }
        } else {
            fs::remove_file(&full_path).map_err(|e| {
                tracing::error!("Failed to delete file: {}", e);
                ApiError::FilePermissionDenied
            })?;
        }

        tracing::info!("File/directory deleted: {} by user {}", request.path, user_id);

        Ok(())
    }

    /// Rename file or directory
    pub async fn rename(user_id: &str, request: RenameRequest) -> ApiResult<FileInfo> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Check dangerous extensions
        if !Self::is_extension_allowed(&request.new_name) {
            return Err(ApiError::FileTypeNotAllowed(request.new_name));
        }

        let base_path = Self::get_user_base_path(user_id);
        let source_path = Self::resolve_path(user_id, &request.path)?;

        if !source_path.exists() {
            return Err(ApiError::FileNotFound(request.path.clone()));
        }

        // Build destination path (same parent, new name)
        let dest_path = source_path
            .parent()
            .ok_or_else(|| ApiError::ValidationError("Invalid path".to_string()))?
            .join(&request.new_name);

        // Check if destination exists
        if dest_path.exists() {
            return Err(ApiError::AlreadyExists("File/Directory".to_string()));
        }

        // Rename
        fs::rename(&source_path, &dest_path).map_err(|e| {
            tracing::error!("Failed to rename: {}", e);
            ApiError::FilePermissionDenied
        })?;

        let metadata = fs::metadata(&dest_path).map_err(|_| {
            ApiError::InternalError("Failed to get file metadata".to_string())
        })?;

        tracing::info!(
            "File renamed: {} -> {} by user {}",
            request.path,
            request.new_name,
            user_id
        );

        Ok(Self::build_file_info(&dest_path, &metadata, &base_path))
    }

    /// Copy file or directory
    pub async fn copy(user_id: &str, request: CopyRequest) -> ApiResult<FileInfo> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let base_path = Self::get_user_base_path(user_id);
        let source_path = Self::resolve_path(user_id, &request.source)?;
        let dest_path = Self::resolve_path(user_id, &request.destination)?;

        if !source_path.exists() {
            return Err(ApiError::FileNotFound(request.source.clone()));
        }

        // Check if destination exists
        if dest_path.exists() && !request.overwrite.unwrap_or(false) {
            return Err(ApiError::AlreadyExists("Destination".to_string()));
        }

        if source_path.is_dir() {
            // Copy directory recursively
            Self::copy_dir_recursive(&source_path, &dest_path)?;
        } else {
            // Copy file
            fs::copy(&source_path, &dest_path).map_err(|e| {
                tracing::error!("Failed to copy file: {}", e);
                ApiError::FilePermissionDenied
            })?;
        }

        let metadata = fs::metadata(&dest_path).map_err(|_| {
            ApiError::InternalError("Failed to get file metadata".to_string())
        })?;

        tracing::info!(
            "File copied: {} -> {} by user {}",
            request.source,
            request.destination,
            user_id
        );

        Ok(Self::build_file_info(&dest_path, &metadata, &base_path))
    }

    /// Helper untuk copy directory recursively
    fn copy_dir_recursive(src: &Path, dst: &Path) -> ApiResult<()> {
        fs::create_dir_all(dst).map_err(|e| {
            tracing::error!("Failed to create directory: {}", e);
            ApiError::FilePermissionDenied
        })?;

        for entry in fs::read_dir(src).map_err(|_| ApiError::FilePermissionDenied)? {
            let entry = entry.map_err(|_| ApiError::FilePermissionDenied)?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path).map_err(|e| {
                    tracing::error!("Failed to copy file: {}", e);
                    ApiError::FilePermissionDenied
                })?;
            }
        }

        Ok(())
    }

    /// Move file or directory
    pub async fn move_file(user_id: &str, request: MoveRequest) -> ApiResult<FileInfo> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let base_path = Self::get_user_base_path(user_id);
        let source_path = Self::resolve_path(user_id, &request.source)?;
        let dest_path = Self::resolve_path(user_id, &request.destination)?;

        if !source_path.exists() {
            return Err(ApiError::FileNotFound(request.source.clone()));
        }

        // Check if destination exists
        if dest_path.exists() && !request.overwrite.unwrap_or(false) {
            return Err(ApiError::AlreadyExists("Destination".to_string()));
        }

        // Move
        fs::rename(&source_path, &dest_path).map_err(|e| {
            tracing::error!("Failed to move: {}", e);
            ApiError::FilePermissionDenied
        })?;

        let metadata = fs::metadata(&dest_path).map_err(|_| {
            ApiError::InternalError("Failed to get file metadata".to_string())
        })?;

        tracing::info!(
            "File moved: {} -> {} by user {}",
            request.source,
            request.destination,
            user_id
        );

        Ok(Self::build_file_info(&dest_path, &metadata, &base_path))
    }

    // ==========================================
    // COMPRESS & EXTRACT OPERATIONS
    // ==========================================

    /// Compress files/directories ke zip archive
    ///
    /// # Arguments
    /// * `user_id` - ID user
    /// * `request` - Data files yang akan di-compress
    ///
    /// # Returns
    /// FileInfo dari archive yang dibuat
    pub async fn compress(user_id: &str, request: CompressRequest) -> ApiResult<FileInfo> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        if request.paths.is_empty() {
            return Err(ApiError::ValidationError(
                "Minimal harus ada satu file untuk di-compress".to_string(),
            ));
        }

        let base_path = Self::get_user_base_path(user_id);

        // Determine output path
        let archive_name = if request.archive_name.ends_with(".zip") {
            request.archive_name.clone()
        } else {
            format!("{}.zip", request.archive_name)
        };

        // Use parent of first file as output location
        let first_path = Self::resolve_path(user_id, &request.paths[0])?;
        let output_dir = first_path
            .parent()
            .unwrap_or(&base_path)
            .to_path_buf();
        let archive_path = output_dir.join(&archive_name);

        // Check if archive already exists
        if archive_path.exists() {
            return Err(ApiError::AlreadyExists(archive_name));
        }

        // Create zip file
        let zip_file = fs::File::create(&archive_path).map_err(|e| {
            tracing::error!("Failed to create zip file: {}", e);
            ApiError::InternalError("Gagal membuat file archive".to_string())
        })?;

        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // Add each path to archive
        for path_str in &request.paths {
            let source_path = Self::resolve_path(user_id, path_str)?;

            if !source_path.exists() {
                return Err(ApiError::FileNotFound(path_str.clone()));
            }

            if source_path.is_file() {
                // Add single file
                let file_name = source_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");

                zip.start_file(file_name, options).map_err(|e| {
                    tracing::error!("Failed to add file to zip: {}", e);
                    ApiError::InternalError("Gagal menambahkan file ke archive".to_string())
                })?;

                let mut file = fs::File::open(&source_path).map_err(|_| {
                    ApiError::FilePermissionDenied
                })?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).map_err(|_| {
                    ApiError::FilePermissionDenied
                })?;

                zip.write_all(&buffer).map_err(|e| {
                    tracing::error!("Failed to write to zip: {}", e);
                    ApiError::InternalError("Gagal menulis ke archive".to_string())
                })?;
            } else if source_path.is_dir() {
                // Add directory recursively
                let base_name = source_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("folder");

                for entry in WalkDir::new(&source_path) {
                    let entry = entry.map_err(|e| {
                        tracing::error!("Failed to walk directory: {}", e);
                        ApiError::FilePermissionDenied
                    })?;

                    let entry_path = entry.path();
                    let relative_path = entry_path
                        .strip_prefix(&source_path)
                        .map_err(|_| ApiError::FilePermissionDenied)?;

                    let archive_path_name = if relative_path.as_os_str().is_empty() {
                        base_name.to_string()
                    } else {
                        format!("{}/{}", base_name, relative_path.to_string_lossy())
                    };

                    if entry_path.is_dir() {
                        zip.add_directory(&archive_path_name, options).map_err(|e| {
                            tracing::error!("Failed to add directory to zip: {}", e);
                            ApiError::InternalError("Gagal menambahkan folder ke archive".to_string())
                        })?;
                    } else {
                        zip.start_file(&archive_path_name, options).map_err(|e| {
                            tracing::error!("Failed to start file in zip: {}", e);
                            ApiError::InternalError("Gagal menambahkan file ke archive".to_string())
                        })?;

                        let mut file = fs::File::open(entry_path).map_err(|_| {
                            ApiError::FilePermissionDenied
                        })?;
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).map_err(|_| {
                            ApiError::FilePermissionDenied
                        })?;

                        zip.write_all(&buffer).map_err(|e| {
                            tracing::error!("Failed to write to zip: {}", e);
                            ApiError::InternalError("Gagal menulis ke archive".to_string())
                        })?;
                    }
                }
            }
        }

        zip.finish().map_err(|e| {
            tracing::error!("Failed to finish zip: {}", e);
            ApiError::InternalError("Gagal menyelesaikan archive".to_string())
        })?;

        let metadata = fs::metadata(&archive_path).map_err(|_| {
            ApiError::InternalError("Gagal membaca metadata archive".to_string())
        })?;

        tracing::info!(
            "Files compressed to {} by user {}",
            archive_name,
            user_id
        );

        Ok(Self::build_file_info(&archive_path, &metadata, &base_path))
    }

    /// Extract zip archive
    ///
    /// # Arguments
    /// * `user_id` - ID user
    /// * `request` - Data archive dan destination
    ///
    /// # Returns
    /// FileInfo dari destination directory
    pub async fn extract(user_id: &str, request: ExtractRequest) -> ApiResult<FileInfo> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let base_path = Self::get_user_base_path(user_id);
        let archive_path = Self::resolve_path(user_id, &request.archive_path)?;
        let dest_path = Self::resolve_path(user_id, &request.destination)?;

        // Check archive exists
        if !archive_path.exists() {
            return Err(ApiError::FileNotFound(request.archive_path.clone()));
        }

        // Check it's a file
        if !archive_path.is_file() {
            return Err(ApiError::ValidationError(
                "Path bukan file archive".to_string(),
            ));
        }

        // Create destination directory if needed
        if !dest_path.exists() {
            fs::create_dir_all(&dest_path).map_err(|e| {
                tracing::error!("Failed to create destination directory: {}", e);
                ApiError::InternalError("Gagal membuat direktori tujuan".to_string())
            })?;
        }

        // Open and extract zip
        let zip_file = fs::File::open(&archive_path).map_err(|_| {
            ApiError::FilePermissionDenied
        })?;

        let mut archive = ZipArchive::new(zip_file).map_err(|e| {
            tracing::error!("Failed to open zip archive: {}", e);
            ApiError::ValidationError("File bukan archive zip yang valid".to_string())
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                tracing::error!("Failed to read archive entry: {}", e);
                ApiError::InternalError("Gagal membaca isi archive".to_string())
            })?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest_path.join(path),
                None => continue,
            };

            // Check if extracted path is within destination (security)
            if !outpath.starts_with(&dest_path) {
                tracing::warn!(
                    "Zip extraction attempted path traversal: {:?}",
                    outpath
                );
                continue;
            }

            // Check if should overwrite
            if outpath.exists() && !request.overwrite.unwrap_or(false) {
                continue;
            }

            if file.is_dir() {
                fs::create_dir_all(&outpath).map_err(|e| {
                    tracing::error!("Failed to create directory: {}", e);
                    ApiError::InternalError("Gagal membuat direktori".to_string())
                })?;
            } else {
                // Ensure parent directory exists
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| {
                            tracing::error!("Failed to create parent directory: {}", e);
                            ApiError::InternalError("Gagal membuat direktori".to_string())
                        })?;
                    }
                }

                let mut outfile = fs::File::create(&outpath).map_err(|e| {
                    tracing::error!("Failed to create output file: {}", e);
                    ApiError::FilePermissionDenied
                })?;

                std::io::copy(&mut file, &mut outfile).map_err(|e| {
                    tracing::error!("Failed to write output file: {}", e);
                    ApiError::InternalError("Gagal menulis file".to_string())
                })?;

                // Set permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
                    }
                }
            }
        }

        let metadata = fs::metadata(&dest_path).map_err(|_| {
            ApiError::InternalError("Gagal membaca metadata".to_string())
        })?;

        tracing::info!(
            "Archive extracted: {} -> {} by user {}",
            request.archive_path,
            request.destination,
            user_id
        );

        Ok(Self::build_file_info(&dest_path, &metadata, &base_path))
    }

    // ==========================================
    // SEARCH OPERATIONS
    // ==========================================

    /// Search files by name
    ///
    /// # Arguments
    /// * `user_id` - ID user
    /// * `query` - Search query string
    /// * `path` - Optional path to start search from
    /// * `max_results` - Maximum results to return (default 50)
    ///
    /// # Returns
    /// Vector of matching FileInfo
    pub async fn search(
        user_id: &str,
        query: &str,
        path: Option<&str>,
        max_results: Option<usize>,
    ) -> ApiResult<Vec<FileInfo>> {
        if query.trim().is_empty() {
            return Err(ApiError::ValidationError(
                "Query pencarian tidak boleh kosong".to_string(),
            ));
        }

        let base_path = Self::get_user_base_path(user_id);
        let search_path = Self::resolve_path(user_id, path.unwrap_or(""))?;
        let max = max_results.unwrap_or(50).min(100); // Cap at 100

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for entry in WalkDir::new(&search_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if results.len() >= max {
                break;
            }

            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Match filename (case-insensitive)
            if file_name.to_lowercase().contains(&query_lower) {
                if let Ok(metadata) = entry.metadata() {
                    results.push(Self::build_file_info(entry_path, &metadata, &base_path));
                }
            }
        }

        tracing::info!(
            "Search '{}' found {} results for user {}",
            query,
            results.len(),
            user_id
        );

        Ok(results)
    }
}
