//! # File Model
//!
//! Model dan DTO untuk File Management operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use validator::Validate;

/// Tipe file entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
}

/// Info tentang file/directory
#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    /// Nama file/directory
    pub name: String,

    /// Path relatif dari user home
    pub path: String,

    /// Tipe (file/directory/symlink)
    pub file_type: FileType,

    /// Ukuran dalam bytes
    pub size: u64,

    /// Permissions (format Unix: rwxr-xr-x)
    pub permissions: String,

    /// Permissions dalam format octal (e.g., 755)
    pub permissions_octal: String,

    /// Owner username
    pub owner: String,

    /// Group name
    pub group: String,

    /// Waktu modifikasi terakhir
    pub modified_at: DateTime<Utc>,

    /// Waktu akses terakhir
    pub accessed_at: Option<DateTime<Utc>>,

    /// Waktu pembuatan
    pub created_at: Option<DateTime<Utc>>,

    /// Extension file (untuk files)
    pub extension: Option<String>,

    /// MIME type (untuk files)
    pub mime_type: Option<String>,

    /// Apakah hidden file (dimulai dengan .)
    pub is_hidden: bool,
}

/// Response untuk list files
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    /// Path saat ini
    pub current_path: String,

    /// Path parent (null jika di root)
    pub parent_path: Option<String>,

    /// Daftar files dan directories
    pub items: Vec<FileInfo>,

    /// Total items
    pub total_items: usize,

    /// Total ukuran semua items dalam bytes
    pub total_size: u64,
}

/// Request untuk list files
#[derive(Debug, Deserialize, Validate)]
pub struct ListFilesRequest {
    /// Path yang akan di-list (relatif dari user home)
    #[validate(length(max = 500, message = "Path maksimal 500 karakter"))]
    pub path: Option<String>,

    /// Include hidden files
    pub show_hidden: Option<bool>,

    /// Sort by: name, size, modified
    pub sort_by: Option<String>,

    /// Sort order: asc, desc
    pub sort_order: Option<String>,
}

/// Request untuk create file atau directory
#[derive(Debug, Deserialize, Validate)]
pub struct CreateFileRequest {
    /// Path parent
    #[validate(length(max = 500, message = "Path maksimal 500 karakter"))]
    pub path: String,

    /// Nama file/directory baru
    #[validate(length(min = 1, max = 255, message = "Nama harus 1-255 karakter"))]
    pub name: String,

    /// Tipe: file atau directory
    pub file_type: String,

    /// Konten file (untuk file biasa)
    pub content: Option<String>,
}

/// Request untuk rename file/directory
#[derive(Debug, Deserialize, Validate)]
pub struct RenameRequest {
    /// Path file/directory yang akan di-rename
    #[validate(length(min = 1, max = 500, message = "Path harus 1-500 karakter"))]
    pub path: String,

    /// Nama baru
    #[validate(length(min = 1, max = 255, message = "Nama baru harus 1-255 karakter"))]
    pub new_name: String,
}

/// Request untuk copy file/directory
#[derive(Debug, Deserialize, Validate)]
pub struct CopyRequest {
    /// Path sumber
    #[validate(length(min = 1, max = 500, message = "Source path harus 1-500 karakter"))]
    pub source: String,

    /// Path tujuan
    #[validate(length(min = 1, max = 500, message = "Destination path harus 1-500 karakter"))]
    pub destination: String,

    /// Overwrite jika sudah ada
    pub overwrite: Option<bool>,
}

/// Request untuk move file/directory
#[derive(Debug, Deserialize, Validate)]
pub struct MoveRequest {
    /// Path sumber
    #[validate(length(min = 1, max = 500, message = "Source path harus 1-500 karakter"))]
    pub source: String,

    /// Path tujuan
    #[validate(length(min = 1, max = 500, message = "Destination path harus 1-500 karakter"))]
    pub destination: String,

    /// Overwrite jika sudah ada
    pub overwrite: Option<bool>,
}

/// Request untuk delete file/directory
#[derive(Debug, Deserialize, Validate)]
pub struct DeleteRequest {
    /// Path file/directory yang akan dihapus
    #[validate(length(min = 1, max = 500, message = "Path harus 1-500 karakter"))]
    pub path: String,

    /// Recursive delete untuk directory
    pub recursive: Option<bool>,
}

/// Request untuk read file content
#[derive(Debug, Deserialize, Validate)]
pub struct ReadFileRequest {
    /// Path file
    #[validate(length(min = 1, max = 500, message = "Path harus 1-500 karakter"))]
    pub path: String,

    /// Encoding (utf-8, base64)
    pub encoding: Option<String>,
}

/// Response untuk file content
#[derive(Debug, Serialize)]
pub struct FileContentResponse {
    /// Path file
    pub path: String,

    /// Nama file
    pub name: String,

    /// Konten file
    pub content: String,

    /// Encoding yang digunakan
    pub encoding: String,

    /// Ukuran file
    pub size: u64,

    /// MIME type
    pub mime_type: Option<String>,
}

/// Request untuk write file content
#[derive(Debug, Deserialize, Validate)]
pub struct WriteFileRequest {
    /// Path file
    #[validate(length(min = 1, max = 500, message = "Path harus 1-500 karakter"))]
    pub path: String,

    /// Konten baru
    pub content: String,

    /// Encoding (utf-8, base64)
    pub encoding: Option<String>,

    /// Create jika tidak ada
    pub create_if_not_exists: Option<bool>,
}

/// Request untuk change permissions
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePermissionsRequest {
    /// Path file/directory
    #[validate(length(min = 1, max = 500, message = "Path harus 1-500 karakter"))]
    pub path: String,

    /// Permissions dalam format octal (e.g., "755", "644")
    #[validate(regex(
        path = "crate::models::file::PERMISSIONS_REGEX",
        message = "Format permissions tidak valid (gunakan format octal seperti 755)"
    ))]
    pub permissions: String,

    /// Recursive untuk directory
    pub recursive: Option<bool>,
}

/// Request untuk compress files
#[derive(Debug, Deserialize, Validate)]
pub struct CompressRequest {
    /// Files/directories yang akan di-compress
    pub paths: Vec<String>,

    /// Nama file archive output
    #[validate(length(min = 1, max = 255, message = "Nama archive harus 1-255 karakter"))]
    pub archive_name: String,

    /// Format: zip, tar.gz, tar.bz2
    pub format: Option<String>,
}

/// Request untuk extract archive
#[derive(Debug, Deserialize, Validate)]
pub struct ExtractRequest {
    /// Path archive file
    #[validate(length(min = 1, max = 500, message = "Path harus 1-500 karakter"))]
    pub archive_path: String,

    /// Destination directory
    #[validate(length(min = 1, max = 500, message = "Destination harus 1-500 karakter"))]
    pub destination: String,

    /// Overwrite existing files
    pub overwrite: Option<bool>,
}

/// Regex untuk validasi permissions octal
pub static PERMISSIONS_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[0-7]{3,4}$").unwrap());

/// Helper untuk konversi Unix permissions ke string
pub fn permissions_to_string(mode: u32) -> String {
    let mut result = String::with_capacity(10);

    // File type
    if mode & 0o40000 != 0 {
        result.push('d');
    } else if mode & 0o120000 != 0 {
        result.push('l');
    } else {
        result.push('-');
    }

    // Owner permissions
    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Other permissions
    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}

/// Helper untuk konversi permissions ke octal string
pub fn permissions_to_octal(mode: u32) -> String {
    format!("{:03o}", mode & 0o777)
}

/// Daftar ekstensi file yang dianggap text
pub const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "html", "htm", "css", "js", "ts", "json", "xml", "yaml", "yml", "toml", "ini",
    "cfg", "conf", "log", "sh", "bash", "zsh", "fish", "py", "rb", "php", "pl", "rs", "go", "java",
    "c", "cpp", "h", "hpp", "cs", "swift", "kt", "scala", "sql", "graphql", "vue", "jsx", "tsx",
    "svelte", "htaccess", "gitignore", "dockerfile", "makefile", "env",
];

/// Daftar ekstensi file berbahaya yang dilarang
pub const DANGEROUS_EXTENSIONS: &[&str] = &[
    "exe", "bat", "cmd", "com", "scr", "pif", "vbs", "vbe", "js", "jse", "ws", "wsf", "wsc", "wsh",
    "ps1", "ps1xml", "ps2", "ps2xml", "psc1", "psc2", "msh", "msh1", "msh2", "mshxml", "msh1xml",
    "msh2xml", "reg", "dll", "sys",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions_to_string() {
        // rwxr-xr-x (755)
        assert_eq!(permissions_to_string(0o100755), "-rwxr-xr-x");
        // rw-r--r-- (644)
        assert_eq!(permissions_to_string(0o100644), "-rw-r--r--");
    }

    #[test]
    fn test_permissions_to_octal() {
        assert_eq!(permissions_to_octal(0o755), "755");
        assert_eq!(permissions_to_octal(0o644), "644");
    }

    #[test]
    fn test_permissions_regex() {
        assert!(PERMISSIONS_REGEX.is_match("755"));
        assert!(PERMISSIONS_REGEX.is_match("644"));
        assert!(PERMISSIONS_REGEX.is_match("0755"));
        assert!(!PERMISSIONS_REGEX.is_match("999"));
        assert!(!PERMISSIONS_REGEX.is_match("75"));
    }
}
