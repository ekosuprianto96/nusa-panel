//! # Domain Routes
//!
//! Route handlers untuk domain management.

use rocket::form::FromForm;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route, State};
use serde::Deserialize;

use crate::database::Database;
use crate::errors::ApiResult;
use crate::guards::AuthenticatedUser;
use crate::models::{
    CreateDnsRecordRequest, CreateDomainRequest, CreateSubdomainRequest, DnsRecordResponse,
    DomainResponse, SubdomainResponse, UpdateDnsRecordRequest, UpdateDomainRequest,
};
use crate::services::DomainService;
use crate::utils::response::{paginated, success, success_message, ApiResponse, PaginatedResponse};

/// Query parameters untuk pagination
#[derive(Debug, Deserialize, FromForm)]
pub struct DomainPaginationParams {
    /// Halaman (default: 1)
    #[field(default = 1)]
    pub page: i64,
    /// Items per halaman (default: 10, max: 50)
    #[field(default = 10)]
    pub per_page: i64,
}

// ==========================================
// DOMAIN ENDPOINTS
// ==========================================

/// List user's domains
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Query Parameters
/// - page: Page number (default: 1)
/// - per_page: Items per page (default: 10, max: 50)
#[get("/?<params..>")]
pub async fn list_domains(
    db: &State<Database>,
    user: AuthenticatedUser,
    params: DomainPaginationParams,
) -> ApiResult<Json<PaginatedResponse<DomainResponse>>> {
    let per_page = params.per_page.min(50).max(1);
    let page = params.page.max(1);

    let result =
        DomainService::get_user_domains(db.get_pool(), &user.id, page, per_page).await?;
    Ok(paginated(result.items, result.total, page, per_page))
}

/// Get domain by ID
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Domain ID
#[get("/<id>")]
pub async fn get_domain(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<DomainResponse>>> {
    let domain = DomainService::get_by_id(db.get_pool(), id, &user.id).await?;
    Ok(success(domain))
}

/// Create new domain
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Request Body
/// ```json
/// {
///   "domain_name": "example.com",
///   "document_root": "/home/example_com/public_html"  // optional
/// }
/// ```
#[post("/", format = "json", data = "<request>")]
pub async fn create_domain(
    db: &State<Database>,
    user: AuthenticatedUser,
    request: Json<CreateDomainRequest>,
) -> ApiResult<Json<ApiResponse<DomainResponse>>> {
    let domain = DomainService::create(db.get_pool(), &user.id, request.into_inner()).await?;
    Ok(success(domain))
}

/// Update domain
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Domain ID
///
/// # Request Body
/// ```json
/// {
///   "document_root": "/new/path",
///   "is_active": true
/// }
/// ```
#[put("/<id>", format = "json", data = "<request>")]
pub async fn update_domain(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
    request: Json<UpdateDomainRequest>,
) -> ApiResult<Json<ApiResponse<DomainResponse>>> {
    let domain =
        DomainService::update(db.get_pool(), id, &user.id, request.into_inner()).await?;
    Ok(success(domain))
}

/// Delete domain
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - id: Domain ID
#[delete("/<id>")]
pub async fn delete_domain(
    db: &State<Database>,
    user: AuthenticatedUser,
    id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    DomainService::delete(db.get_pool(), id, &user.id).await?;
    Ok(success_message("Domain berhasil dihapus"))
}

// ==========================================
// SUBDOMAIN ENDPOINTS
// ==========================================

/// List subdomains for a domain
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - domain_id: Domain ID
#[get("/<domain_id>/subdomains")]
pub async fn list_subdomains(
    db: &State<Database>,
    user: AuthenticatedUser,
    domain_id: &str,
) -> ApiResult<Json<ApiResponse<Vec<SubdomainResponse>>>> {
    let subdomains = DomainService::get_subdomains(db.get_pool(), domain_id, &user.id).await?;
    Ok(success(subdomains))
}

/// Create subdomain
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - domain_id: Domain ID
///
/// # Request Body
/// ```json
/// {
///   "subdomain_name": "blog",
///   "document_root": "/home/example_com/blog/public_html"  // optional
/// }
/// ```
#[post("/<domain_id>/subdomains", format = "json", data = "<request>")]
pub async fn create_subdomain(
    db: &State<Database>,
    user: AuthenticatedUser,
    domain_id: &str,
    request: Json<CreateSubdomainRequest>,
) -> ApiResult<Json<ApiResponse<SubdomainResponse>>> {
    let subdomain = DomainService::create_subdomain(
        db.get_pool(),
        domain_id,
        &user.id,
        request.into_inner(),
    )
    .await?;
    Ok(success(subdomain))
}

/// Delete subdomain
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - domain_id: Domain ID
/// - subdomain_id: Subdomain ID
#[delete("/<domain_id>/subdomains/<subdomain_id>")]
pub async fn delete_subdomain(
    db: &State<Database>,
    user: AuthenticatedUser,
    domain_id: &str,
    subdomain_id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    DomainService::delete_subdomain(db.get_pool(), domain_id, subdomain_id, &user.id).await?;
    Ok(success_message("Subdomain berhasil dihapus"))
}

// ==========================================
// DNS RECORDS ENDPOINTS
// ==========================================

/// List DNS records for a domain
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - domain_id: Domain ID
#[get("/<domain_id>/dns")]
pub async fn list_dns_records(
    db: &State<Database>,
    user: AuthenticatedUser,
    domain_id: &str,
) -> ApiResult<Json<ApiResponse<Vec<DnsRecordResponse>>>> {
    let records = DomainService::get_dns_records(db.get_pool(), domain_id, &user.id).await?;
    Ok(success(records))
}

/// Create DNS record
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - domain_id: Domain ID
///
/// # Request Body
/// ```json
/// {
///   "record_type": "A",
///   "name": "@",
///   "value": "192.168.1.1",
///   "ttl": 3600,
///   "priority": null
/// }
/// ```
#[post("/<domain_id>/dns", format = "json", data = "<request>")]
pub async fn create_dns_record(
    db: &State<Database>,
    user: AuthenticatedUser,
    domain_id: &str,
    request: Json<CreateDnsRecordRequest>,
) -> ApiResult<Json<ApiResponse<DnsRecordResponse>>> {
    let record = DomainService::create_dns_record(
        db.get_pool(),
        domain_id,
        &user.id,
        request.into_inner(),
    )
    .await?;
    Ok(success(record))
}

/// Update DNS record
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - domain_id: Domain ID
/// - record_id: DNS Record ID
///
/// # Request Body
/// ```json
/// {
///   "value": "192.168.1.2",
///   "ttl": 7200
/// }
/// ```
#[put("/<domain_id>/dns/<record_id>", format = "json", data = "<request>")]
pub async fn update_dns_record(
    db: &State<Database>,
    user: AuthenticatedUser,
    domain_id: &str,
    record_id: &str,
    request: Json<UpdateDnsRecordRequest>,
) -> ApiResult<Json<ApiResponse<DnsRecordResponse>>> {
    let record = DomainService::update_dns_record(
        db.get_pool(),
        domain_id,
        record_id,
        &user.id,
        request.into_inner(),
    )
    .await?;
    Ok(success(record))
}

/// Delete DNS record
///
/// # Headers
/// - Authorization: Bearer <access_token>
///
/// # Path Parameters
/// - domain_id: Domain ID
/// - record_id: DNS Record ID
#[delete("/<domain_id>/dns/<record_id>")]
pub async fn delete_dns_record(
    db: &State<Database>,
    user: AuthenticatedUser,
    domain_id: &str,
    record_id: &str,
) -> ApiResult<Json<ApiResponse<()>>> {
    DomainService::delete_dns_record(db.get_pool(), domain_id, record_id, &user.id).await?;
    Ok(success_message("DNS record berhasil dihapus"))
}

/// Mendapatkan routes untuk domains
pub fn domain_routes() -> Vec<Route> {
    routes![
        // Domain CRUD
        list_domains,
        get_domain,
        create_domain,
        update_domain,
        delete_domain,
        // Subdomain
        list_subdomains,
        create_subdomain,
        delete_subdomain,
        // DNS Records
        list_dns_records,
        create_dns_record,
        update_dns_record,
        delete_dns_record
    ]
}
