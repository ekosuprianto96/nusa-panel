//! # Email Routes
//!
//! Route handlers untuk email management.

use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{
    AutoresponderResponse, CreateAutoresponderRequest, CreateEmailAccountRequest,
    CreateEmailForwarderRequest, EmailAccountResponse, EmailForwarderResponse,
    UpdateAutoresponderRequest, UpdateEmailAccountRequest,
};
use crate::services::EmailService;
use crate::utils::response::{success, success_message, ApiResponse};

// ==========================================
// EMAIL ACCOUNT ENDPOINTS
// ==========================================

/// List user's email accounts
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/")]
pub async fn list_email_accounts(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<EmailAccountResponse>>>> {
    let accounts = EmailService::get_user_accounts(db.get_pool(), &user.id).await?;
    Ok(success(accounts))
}

/// Get email account by ID
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Email Account ID
#[get("/<id>")]
pub async fn get_email_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<EmailAccountResponse>>> {
    let account = EmailService::get_account_by_id(db.get_pool(), id, &user.id).await?;
    Ok(success(account))
}

/// Create email account
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "domain_id": "domain-uuid",
///   "username": "info",
///   "password": "SecureP@ss123",
///   "quota_mb": 1024  // optional, default 1GB
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn create_email_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateEmailAccountRequest>,
) -> ApiResult<Json<ApiResponse<EmailAccountResponse>>> {
    let account =
        EmailService::create_account(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(account))
}

/// Update email account
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Email Account ID
///
/// # Request Body
/// ```json
/// {
///   "password": "NewP@ssword",  // optional
///   "quota_mb": 2048,  // optional
///   "is_active": true  // optional
/// }
/// ```
#[put("/<id>", format = "json", data = "<request>")]
pub async fn update_email_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateEmailAccountRequest>,
) -> ApiResult<Json<ApiResponse<EmailAccountResponse>>> {
    let account =
        EmailService::update_account(db.get_pool(), id, &user.id, request.into_inner()).await?;
    Ok(success(account))
}

/// Delete email account
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Email Account ID
#[delete("/<id>")]
pub async fn delete_email_account(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    EmailService::delete_account(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Email account berhasil dihapus"))
}

// ==========================================
// EMAIL FORWARDER ENDPOINTS
// ==========================================

/// List user's email forwarders
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/forwarders")]
pub async fn list_email_forwarders(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<EmailForwarderResponse>>>> {
    let forwarders = EmailService::get_user_forwarders(db.get_pool(), &user.id).await?;
    Ok(success(forwarders))
}

/// Create email forwarder
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "domain_id": "domain-uuid",
///   "source_username": "sales",
///   "destination_email": "john@gmail.com"
/// }
/// ```
#[post("/forwarders", format = "json", data = "<request>")]
pub async fn create_email_forwarder(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateEmailForwarderRequest>,
) -> ApiResult<Json<ApiResponse<EmailForwarderResponse>>> {
    let forwarder =
        EmailService::create_forwarder(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(forwarder))
}

/// Delete email forwarder
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Forwarder ID
#[delete("/forwarders/<id>")]
pub async fn delete_email_forwarder(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    EmailService::delete_forwarder(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Email forwarder berhasil dihapus"))
}

// ==========================================
// AUTORESPONDER ENDPOINTS
// ==========================================

/// List user's autoresponders
///
/// # Headers
/// - Authorization: Bearer <access_token>
#[get("/autoresponders")]
pub async fn list_autoresponders(
    db: &State<Database>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<AutoresponderResponse>>>> {
    let autoresponders = EmailService::get_user_autoresponders(db.get_pool(), &user.id).await?;
    Ok(success(autoresponders))
}

/// Create autoresponder
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "email_account_id": "email-uuid",
///   "subject": "Out of Office",
///   "body": "Terima kasih atas email Anda. Saat ini saya sedang...",
///   "start_date": "2024-01-15T00:00:00Z",  // optional
///   "end_date": "2024-01-20T00:00:00Z"  // optional
/// }
/// ```
#[post("/autoresponders", format = "json", data = "<request>")]
pub async fn create_autoresponder(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateAutoresponderRequest>,
) -> ApiResult<Json<ApiResponse<AutoresponderResponse>>> {
    let autoresponder =
        EmailService::create_autoresponder(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(autoresponder))
}

/// Update autoresponder
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Autoresponder ID
///
/// # Request Body
/// ```json
/// {
///   "subject": "Updated Subject",
///   "body": "Updated body...",
///   "is_active": false
/// }
/// ```
#[put("/autoresponders/<id>", format = "json", data = "<request>")]
pub async fn update_autoresponder(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateAutoresponderRequest>,
) -> ApiResult<Json<ApiResponse<AutoresponderResponse>>> {
    let autoresponder = EmailService::update_autoresponder(
        db.get_pool(),
        id,
        &user.id,
        request.into_inner(),
    )
    .await?;
    Ok(success(autoresponder))
}

/// Delete autoresponder
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Autoresponder ID
#[delete("/autoresponders/<id>")]
pub async fn delete_autoresponder(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    EmailService::delete_autoresponder(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Autoresponder berhasil dihapus"))
}

/// Mendapatkan routes untuk emails
pub fn email_routes() -> Vec<Route> {
    routes![
        // Email Accounts
        list_email_accounts,
        get_email_account,
        create_email_account,
        update_email_account,
        delete_email_account,
        // Forwarders
        list_email_forwarders,
        create_email_forwarder,
        delete_email_forwarder,
        // Autoresponders
        list_autoresponders,
        create_autoresponder,
        update_autoresponder,
        delete_autoresponder
    ]
}
