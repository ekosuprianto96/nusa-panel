use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use crate::config::CONFIG;
use crate::errors::{ApiError, ApiResult};
use crate::models::{VueFinderResponse, VueFinderFile};

pub struct VueFinderService;

impl VueFinderService {
    /// Sandbox: Get user home directory
    fn get_user_base_path(username: &str) -> PathBuf {
        let base = &CONFIG.file.user_home_base;
        PathBuf::from(base).join(username)
    }

    /// Resolve and validate path (Sandbox Check)
    pub fn resolve_path(username: &str, relative_path: &str) -> ApiResult<PathBuf> {
        let base_path = Self::get_user_base_path(username);

        // Ensure user home exists
        if !base_path.exists() {
             // Logic to create home is complex (needs sudo), assume it exists or use system helper
             // For now, fail if not exists to be safe, or try to create like FileService
             // We'll mimic FileService's auto-create logic if possible, 
             // but simpler: just return error if not exists for now unless critical.
             // Actually, FileService creates it. Let's assume FileService or SystemService handles creation on user init.
             // But if we want robust, we should create it.
        }

        let clean_path = relative_path
            .trim()
            .trim_start_matches('/')
            .trim_start_matches('\\');

        if clean_path.contains("..") {
            return Err(ApiError::Forbidden);
        }

        let full_path = if clean_path.is_empty() {
            base_path.clone()
        } else {
            base_path.join(clean_path)
        };

        // Canonicalization check
        // If path exists, canonicalize and check prefix
         if full_path.exists() {
            let canonical = full_path.canonicalize().unwrap_or(full_path.clone());
            let canonical_base = base_path.canonicalize().unwrap_or(base_path.clone());
            if !canonical.starts_with(&canonical_base) {
                return Err(ApiError::Forbidden);
            }
            Ok(canonical)
        } else {
            // If strictly new path, just return full_path, but check parent
            if let Some(parent) = full_path.parent() {
                 if parent.exists() {
                     let canonical_parent = parent.canonicalize().unwrap_or(parent.to_path_buf());
                     let canonical_base = base_path.canonicalize().unwrap_or(base_path.clone());
                     if !canonical_parent.starts_with(&canonical_base) {
                         return Err(ApiError::Forbidden);
                     }
                 }
            }
            Ok(full_path)
        }
    }

    /// Handle "index" action (List files)
    pub async fn index(username: &str, path: Option<String>) -> ApiResult<VueFinderResponse<Vec<VueFinderFile>>> {
        let target_path = Self::resolve_path(username, path.as_deref().unwrap_or(""))?;
        
        if !target_path.exists() || !target_path.is_dir() {
             // If root not found, try to create or return empty
             // For VueFinder, strict error might be better?
             // Or just return empty list.
             // Let's err if path specified but not found.
             if path.is_some() {
                 return Err(ApiError::NotFound("Directory".to_string()));
             }
             // Valid case: user has no home yet?
        }

        let mut files = Vec::new();
        if target_path.exists() {
            let entries = fs::read_dir(&target_path).map_err(|e| ApiError::InternalError(e.to_string()))?;
            for entry in entries {
                let entry = entry.map_err(|_| ApiError::InternalError("Read entry failed".to_string()))?;
                let meta = entry.metadata().map_err(|_| ApiError::InternalError("Metadata failed".to_string()))?;
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Skip hidden files if preferred? VueFinder usually shows them.
                
                let file_type = if meta.is_dir() { "dir" } else { "file" };
                let mime = if meta.is_dir() { None } else { Some(Self::guess_mime(&name)) };
                let _url = format!("/api/vuefinder/preview?path={}/{}", path.as_deref().unwrap_or(""), name); // Correct logic needed for URL

                // Fix URL: relative path for next actions
               // Actually VueFinder expects "path" in node to be the RELATIVE path from root.
                let rel_path = if let Some(p) = &path {
                    if p.is_empty() { name.clone() } else { format!("{}/{}", p, name) }
                } else {
                    name.clone()
                };

                files.push(VueFinderFile {
                    path: rel_path,
                    basename: name.clone(),
                    dirname: path.clone().unwrap_or_default(), // Parent dir
                    r#type: file_type.to_string(),
                    size: meta.len(),
                    timestamp: meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH).duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                    mime,
                    visibility: "public".to_string(),
                    url: "".to_string(), // Optional usually
                });
            }
        }
        
        Ok(VueFinderResponse {
            adapter: "local".to_string(),
            storages: vec!["local".to_string()],
            results: files,
        })
    }

    /// Handle "new_folder"
    pub async fn new_folder(username: &str, path: String, name: String) -> ApiResult<VueFinderResponse<VueFinderFile>> {
        let parent = Self::resolve_path(username, &path)?;
        let new_path = parent.join(&name);
        
        // Security check done in resolve_path(parent) + basic join
        // We should verify new_path didn't escape (e.g. name="..")
        if name.contains("..") || name.contains("/") || name.contains("\\") {
            return Err(ApiError::ValidationError("Invalid name".to_string()));
        }

        if new_path.exists() {
             return Err(ApiError::AlreadyExists("Folder".to_string()));
        }

        fs::create_dir(&new_path).map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        let _meta = fs::metadata(&new_path).unwrap();
        
        let rel_path = if path.is_empty() { name.clone() } else { format!("{}/{}", path, name) };

        Ok(VueFinderResponse {
            adapter: "local".to_string(),
            storages: vec!["local".to_string()],
            results: VueFinderFile {
                path: rel_path,
                basename: name.clone(),
                dirname: path,
                r#type: "dir".to_string(),
                size: 0,
                timestamp: Utc::now().timestamp(),
                mime: None,
                visibility: "public".to_string(),
                url: "".to_string(),
            },
        })
    }

    // MIME guesser
    fn guess_mime(filename: &str) -> String {
        let ext = Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "txt" => "text/plain",
            "html" => "text/html",
            "php" => "text/x-php",
            "css" => "text/css",
            "js" => "text/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "zip" => "application/zip",
            "pdf" => "application/pdf",
            _ => "application/octet-stream",
        }.to_string()
    }

    /// Handle "delete"
    pub async fn delete(username: &str, path: String) -> ApiResult<VueFinderResponse<()>> {
        let target_path = Self::resolve_path(username, &path)?;
        
        if !target_path.exists() {
            return Err(ApiError::NotFound("File/Directory".to_string()));
        }
        
        if target_path.is_dir() {
            fs::remove_dir_all(&target_path).map_err(|e| ApiError::InternalError(e.to_string()))?;
        } else {
            fs::remove_file(&target_path).map_err(|e| ApiError::InternalError(e.to_string()))?;
        }
        
        Ok(VueFinderResponse {
            adapter: "local".to_string(),
            storages: vec!["local".to_string()],
            results: (),
        })
    }

    /// Handle "rename"
    pub async fn rename(username: &str, path: String, name: String) -> ApiResult<VueFinderResponse<VueFinderFile>> {
        let source_path = Self::resolve_path(username, &path)?;
        
        if !source_path.exists() {
            return Err(ApiError::NotFound("File".to_string()));
        }
        
        let parent = source_path.parent().ok_or(ApiError::Forbidden)?;
        let new_path = parent.join(&name);
        
        if new_path.exists() {
            return Err(ApiError::AlreadyExists("Destination".to_string()));
        }
        
        fs::rename(&source_path, &new_path).map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        let meta = fs::metadata(&new_path).unwrap();
        let dirname = Path::new(&path).parent().unwrap().to_string_lossy().to_string();
        let rel_path = if dirname.is_empty() { name.clone() } else { format!("{}/{}", dirname, name) };

        Ok(VueFinderResponse {
            adapter: "local".to_string(),
            storages: vec!["local".to_string()],
            results: VueFinderFile {
                path: rel_path,
                basename: name.clone(),
                dirname,
                r#type: if meta.is_dir() { "dir".to_string() } else { "file".to_string() },
                size: meta.len(),
                timestamp: Utc::now().timestamp(),
                mime: None,
                visibility: "public".to_string(),
                url: "".to_string(),
            },
        })
    }
}
