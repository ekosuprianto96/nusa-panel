//! # Domain Service
//!
//! Business logic untuk domain management: CRUD domains, subdomains, dan DNS records.

use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;

use crate::config::CONFIG;
use crate::errors::{ApiError, ApiResult};
use crate::models::{
    CreateDnsRecordRequest, CreateDomainRequest, CreateSubdomainRequest, DnsRecord,
    DnsRecordResponse, Domain, DomainResponse, PaginatedDomains, Subdomain, SubdomainResponse, UpdateDnsRecordRequest,
    UpdateDomainRequest,
};

/// Service untuk operasi domain
pub struct DomainService;

impl DomainService {
    // ==========================================
    // DOMAIN OPERATIONS
    // ==========================================

    /// Get all domains untuk user
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user pemilik domains
    /// * `page` - Halaman (1-indexed)
    /// * `per_page` - Items per halaman
    ///
    /// # Returns
    /// Vector of DomainResponse dengan total count
    pub async fn get_user_domains(
        pool: &MySqlPool,
        user_id: &str,
        page: i64,
        per_page: i64,
    ) -> ApiResult<PaginatedDomains> {
        let offset = (page - 1) * per_page;

        // Get total count
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM domains WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        // Get paginated domains
        let domains = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        // Convert to response with counts
        let mut items = Vec::with_capacity(domains.len());
        for domain in domains {
            let subdomains_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM subdomains WHERE domain_id = ?",
            )
            .bind(&domain.id)
            .fetch_one(pool)
            .await? as i32;

            let dns_records_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM dns_records WHERE domain_id = ?",
            )
            .bind(&domain.id)
            .fetch_one(pool)
            .await? as i32;

            let mut response = DomainResponse::from(domain);
            response.subdomains_count = subdomains_count;
            response.dns_records_count = dns_records_count;
            items.push(response);
        }

        Ok(PaginatedDomains { items, total })
    }

    /// Get domain by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `domain_id` - ID domain
    /// * `user_id` - ID user untuk authorization check
    pub async fn get_by_id(
        pool: &MySqlPool,
        domain_id: &str,
        user_id: &str,
    ) -> ApiResult<DomainResponse> {
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        // Check authorization
        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let subdomains_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM subdomains WHERE domain_id = ?",
        )
        .bind(&domain.id)
        .fetch_one(pool)
        .await? as i32;

        let dns_records_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dns_records WHERE domain_id = ?",
        )
        .bind(&domain.id)
        .fetch_one(pool)
        .await? as i32;

        let mut response = DomainResponse::from(domain);
        response.subdomains_count = subdomains_count;
        response.dns_records_count = dns_records_count;

        Ok(response)
    }

    /// Create new domain
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - ID user pemilik
    /// * `request` - Data domain baru
    pub async fn create(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateDomainRequest,
    ) -> ApiResult<DomainResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Normalize domain name to lowercase
        let domain_name = request.domain_name.to_lowercase();

        // Check if domain already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM domains WHERE domain_name = ?",
        )
        .bind(&domain_name)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Domain".to_string()));
        }

        // Generate document root if not provided
        let document_root = request.document_root.unwrap_or_else(|| {
            format!(
                "{}/{}/public_html",
                CONFIG.file.user_home_base,
                domain_name.replace('.', "_")
            )
        });

        // Generate UUID
        let domain_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Insert domain
        sqlx::query(
            r#"
            INSERT INTO domains (id, user_id, domain_name, document_root, is_active, ssl_enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, TRUE, FALSE, ?, ?)
            "#,
        )
        .bind(&domain_id)
        .bind(user_id)
        .bind(&domain_name)
        .bind(&document_root)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Create default DNS records
        Self::create_default_dns_records(pool, &domain_id).await?;

        tracing::info!("Domain created: {} for user {}", domain_name, user_id);

        // Fetch and return created domain
        Self::get_by_id(pool, &domain_id, user_id).await
    }

    /// Create default DNS records for a new domain
    async fn create_default_dns_records(pool: &MySqlPool, domain_id: &str) -> ApiResult<()> {
        let now = Utc::now();

        // Default A record pointing to localhost (user should update this)
        let records = vec![
            ("A", "@", "127.0.0.1", 3600, None::<i32>),
            ("A", "www", "127.0.0.1", 3600, None),
            ("MX", "@", "mail", 3600, Some(10)),
        ];

        for (record_type, name, value, ttl, priority) in records {
            let record_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO dns_records (id, domain_id, record_type, name, value, ttl, priority, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&record_id)
            .bind(domain_id)
            .bind(record_type)
            .bind(name)
            .bind(value)
            .bind(ttl)
            .bind(priority)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Update domain
    pub async fn update(
        pool: &MySqlPool,
        domain_id: &str,
        user_id: &str,
        request: UpdateDomainRequest,
    ) -> ApiResult<DomainResponse> {
        // Get existing domain
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        // Check authorization
        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build update values
        let document_root = request.document_root.unwrap_or(domain.document_root);
        let is_active = request.is_active.unwrap_or(domain.is_active);

        // Update domain
        sqlx::query(
            "UPDATE domains SET document_root = ?, is_active = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&document_root)
        .bind(is_active)
        .bind(Utc::now())
        .bind(domain_id)
        .execute(pool)
        .await?;

        tracing::info!("Domain updated: {}", domain.domain_name);

        Self::get_by_id(pool, domain_id, user_id).await
    }

    /// Delete domain
    pub async fn delete(pool: &MySqlPool, domain_id: &str, user_id: &str) -> ApiResult<()> {
        // Get domain
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        // Check authorization
        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Delete domain (cascade akan menghapus subdomains dan dns_records)
        sqlx::query("DELETE FROM domains WHERE id = ?")
            .bind(domain_id)
            .execute(pool)
            .await?;

        tracing::info!("Domain deleted: {}", domain.domain_name);

        Ok(())
    }

    // ==========================================
    // SUBDOMAIN OPERATIONS
    // ==========================================

    /// Get all subdomains untuk domain
    pub async fn get_subdomains(
        pool: &MySqlPool,
        domain_id: &str,
        user_id: &str,
    ) -> ApiResult<Vec<SubdomainResponse>> {
        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Get subdomains
        let subdomains = sqlx::query_as::<_, Subdomain>(
            "SELECT * FROM subdomains WHERE domain_id = ? ORDER BY subdomain_name",
        )
        .bind(domain_id)
        .fetch_all(pool)
        .await?;

        // Convert to responses
        let responses: Vec<SubdomainResponse> = subdomains
            .into_iter()
            .map(|s| SubdomainResponse {
                id: s.id,
                domain_id: s.domain_id,
                subdomain_name: s.subdomain_name.clone(),
                full_name: format!("{}.{}", s.subdomain_name, domain.domain_name),
                document_root: s.document_root,
                is_active: s.is_active,
                created_at: s.created_at,
            })
            .collect();

        Ok(responses)
    }

    /// Create subdomain
    pub async fn create_subdomain(
        pool: &MySqlPool,
        domain_id: &str,
        user_id: &str,
        request: CreateSubdomainRequest,
    ) -> ApiResult<SubdomainResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Normalize subdomain name
        let subdomain_name = request.subdomain_name.to_lowercase();

        // Check if subdomain already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM subdomains WHERE domain_id = ? AND subdomain_name = ?",
        )
        .bind(domain_id)
        .bind(&subdomain_name)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Subdomain".to_string()));
        }

        // Generate document root
        let document_root = request.document_root.unwrap_or_else(|| {
            format!(
                "{}/{}/{}/public_html",
                CONFIG.file.user_home_base,
                domain.domain_name.replace('.', "_"),
                subdomain_name
            )
        });

        // Insert subdomain
        let subdomain_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO subdomains (id, domain_id, subdomain_name, document_root, is_active, created_at)
            VALUES (?, ?, ?, ?, TRUE, ?)
            "#,
        )
        .bind(&subdomain_id)
        .bind(domain_id)
        .bind(&subdomain_name)
        .bind(&document_root)
        .bind(now)
        .execute(pool)
        .await?;

        // Create A record for subdomain
        let dns_record_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO dns_records (id, domain_id, record_type, name, value, ttl, created_at, updated_at)
            VALUES (?, ?, 'A', ?, '127.0.0.1', 3600, ?, ?)
            "#,
        )
        .bind(&dns_record_id)
        .bind(domain_id)
        .bind(&subdomain_name)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!(
            "Subdomain created: {}.{}",
            subdomain_name,
            domain.domain_name
        );

        Ok(SubdomainResponse {
            id: subdomain_id,
            domain_id: domain_id.to_string(),
            subdomain_name: subdomain_name.clone(),
            full_name: format!("{}.{}", subdomain_name, domain.domain_name),
            document_root,
            is_active: true,
            created_at: now,
        })
    }

    /// Delete subdomain
    pub async fn delete_subdomain(
        pool: &MySqlPool,
        domain_id: &str,
        subdomain_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Get subdomain
        let subdomain = sqlx::query_as::<_, Subdomain>(
            "SELECT * FROM subdomains WHERE id = ? AND domain_id = ?",
        )
        .bind(subdomain_id)
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Subdomain".to_string()))?;

        // Delete subdomain
        sqlx::query("DELETE FROM subdomains WHERE id = ?")
            .bind(subdomain_id)
            .execute(pool)
            .await?;

        // Delete associated DNS record
        sqlx::query("DELETE FROM dns_records WHERE domain_id = ? AND name = ?")
            .bind(domain_id)
            .bind(&subdomain.subdomain_name)
            .execute(pool)
            .await?;

        tracing::info!(
            "Subdomain deleted: {}.{}",
            subdomain.subdomain_name,
            domain.domain_name
        );

        Ok(())
    }

    // ==========================================
    // DNS RECORDS OPERATIONS
    // ==========================================

    /// Get all DNS records untuk domain
    pub async fn get_dns_records(
        pool: &MySqlPool,
        domain_id: &str,
        user_id: &str,
    ) -> ApiResult<Vec<DnsRecordResponse>> {
        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Get DNS records
        let records = sqlx::query_as::<_, DnsRecord>(
            "SELECT * FROM dns_records WHERE domain_id = ? ORDER BY record_type, name",
        )
        .bind(domain_id)
        .fetch_all(pool)
        .await?;

        let responses: Vec<DnsRecordResponse> =
            records.into_iter().map(DnsRecordResponse::from).collect();

        Ok(responses)
    }

    /// Create DNS record
    pub async fn create_dns_record(
        pool: &MySqlPool,
        domain_id: &str,
        user_id: &str,
        request: CreateDnsRecordRequest,
    ) -> ApiResult<DnsRecordResponse> {
        // Validate input
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Validate record type
        let record_type = request.record_type.to_uppercase();
        let valid_types = ["A", "AAAA", "CNAME", "MX", "TXT", "NS", "SOA", "SRV", "CAA"];
        if !valid_types.contains(&record_type.as_str()) {
            return Err(ApiError::ValidationError(format!(
                "Tipe record tidak valid. Harus salah satu dari: {:?}",
                valid_types
            )));
        }

        // Insert record
        let record_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let ttl = request.ttl.unwrap_or(3600);

        sqlx::query(
            r#"
            INSERT INTO dns_records (id, domain_id, record_type, name, value, ttl, priority, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record_id)
        .bind(domain_id)
        .bind(&record_type)
        .bind(&request.name)
        .bind(&request.value)
        .bind(ttl)
        .bind(request.priority)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        tracing::info!(
            "DNS record created: {} {} for {}",
            record_type,
            request.name,
            domain.domain_name
        );

        Ok(DnsRecordResponse {
            id: record_id,
            domain_id: domain_id.to_string(),
            record_type,
            name: request.name,
            value: request.value,
            ttl,
            priority: request.priority,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update DNS record
    pub async fn update_dns_record(
        pool: &MySqlPool,
        domain_id: &str,
        record_id: &str,
        user_id: &str,
        request: UpdateDnsRecordRequest,
    ) -> ApiResult<DnsRecordResponse> {
        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Get existing record
        let record = sqlx::query_as::<_, DnsRecord>(
            "SELECT * FROM dns_records WHERE id = ? AND domain_id = ?",
        )
        .bind(record_id)
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("DNS Record".to_string()))?;

        // Build update values
        let value = request.value.unwrap_or(record.value);
        let ttl = request.ttl.unwrap_or(record.ttl);
        let priority = request.priority.or(record.priority);
        let now = Utc::now();

        // Update record
        sqlx::query(
            "UPDATE dns_records SET value = ?, ttl = ?, priority = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&value)
        .bind(ttl)
        .bind(priority)
        .bind(now)
        .bind(record_id)
        .execute(pool)
        .await?;

        tracing::info!(
            "DNS record updated: {} {} for {}",
            record.record_type,
            record.name,
            domain.domain_name
        );

        Ok(DnsRecordResponse {
            id: record_id.to_string(),
            domain_id: domain_id.to_string(),
            record_type: record.record_type,
            name: record.name,
            value,
            ttl,
            priority,
            created_at: record.created_at,
            updated_at: now,
        })
    }

    /// Delete DNS record
    pub async fn delete_dns_record(
        pool: &MySqlPool,
        domain_id: &str,
        record_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE id = ?",
        )
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Get record
        let record = sqlx::query_as::<_, DnsRecord>(
            "SELECT * FROM dns_records WHERE id = ? AND domain_id = ?",
        )
        .bind(record_id)
        .bind(domain_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("DNS Record".to_string()))?;

        // Delete record
        sqlx::query("DELETE FROM dns_records WHERE id = ?")
            .bind(record_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "DNS record deleted: {} {} for {}",
            record.record_type,
            record.name,
            domain.domain_name
        );

        Ok(())
    }
}
