use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VueFinderResponse<T> {
    pub adapter: String,
    pub storages: Vec<String>,
    pub results: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VueFinderFile {
    pub path: String,
    pub basename: String,
    pub dirname: String,
    pub r#type: String, // "dir" or "file"
    pub size: u64,
    pub timestamp: i64,
    pub mime: Option<String>,
    pub visibility: String, // "public"
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VueFinderDiskUsage {
    pub used: u64,
    pub total: u64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct VueFinderRequest {
    pub q: String, // Action: index, new_folder, etc.
    pub adapter: Option<String>,
    pub path: Option<String>,
    pub name: Option<String>,
    // Add other fields as needed
}
