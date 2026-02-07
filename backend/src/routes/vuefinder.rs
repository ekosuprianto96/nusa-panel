use rocket::{get, routes, Route};
use rocket::serde::json::{Value, serde_json};
use crate::guards::AuthenticatedUser;
use crate::errors::{ApiResult, ApiError};
use crate::services::VueFinderService;


pub fn vuefinder_routes() -> Vec<Route> {
    routes![handle_vuefinder]
}

#[get("/vuefinder?<q>&<adapter>&<path>&<name>")]
pub async fn handle_vuefinder(
    user: AuthenticatedUser,
    q: Option<String>,
    adapter: Option<String>, // 'path' in VueFinder usually refers to the node path
    path: Option<String>,
    name: Option<String>,
) -> ApiResult<Value> {
    let _ = adapter; // Suppress unused variable warning
    let action = q.as_deref().unwrap_or("index");
    let username = &user.username;

    match action {
        "index" => {
            let response = VueFinderService::index(username, path).await?;
            Ok(serde_json::to_value(response).unwrap())
        },
        "new_folder" => {
            let p = path.unwrap_or_default();
            let n = name.ok_or(ApiError::ValidationError("Name required".to_string()))?;
            let response = VueFinderService::new_folder(username, p, n).await?;
            Ok(serde_json::to_value(response).unwrap())
        },
        "delete" => {
            let p = path.ok_or(ApiError::ValidationError("Path required".to_string()))?;
            let response = VueFinderService::delete(username, p).await?;
            Ok(serde_json::to_value(response).unwrap())
        },
        "rename" => {
            let p = path.ok_or(ApiError::ValidationError("Path required".to_string()))?;
            let n = name.ok_or(ApiError::ValidationError("Name required".to_string()))?;
            let response = VueFinderService::rename(username, p, n).await?;
            Ok(serde_json::to_value(response).unwrap())
        },
        _ => Err(ApiError::ValidationError("Unknown action".to_string())),
    }
}
