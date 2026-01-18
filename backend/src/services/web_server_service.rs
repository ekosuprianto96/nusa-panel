//! # Web Server Service
//!
//! Business logic untuk Web Server configuration (Nginx/VirtualHost).
//! Menangani pembuatan konfigurasi, SSL request, dan PHP selection.

use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;
use validator::Validate;
use crate::errors::{ApiError, ApiResult};
use crate::models::{
    CreateVirtualHostRequest, Domain, PackageSpecs, PhpVersion, RequestSslRequest,
    SslCertificate, SslCertificateResponse, UpdateVirtualHostRequest, VirtualHost,
    VirtualHostResponse,
};

/// Service untuk web server operations
pub struct WebServerService;

impl WebServerService {
    // ==========================================
    // VIRTUAL HOST OPERATIONS
    // ==========================================

    /// Get all virtual hosts untuk user
    pub async fn get_user_vhosts(
        pool: &MySqlPool,
        user_id: &str,
    ) -> ApiResult<Vec<VirtualHostResponse>> {
        let vhosts = sqlx::query_as::<_, VirtualHost>(
            "SELECT * FROM virtual_hosts WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut responses = Vec::with_capacity(vhosts.len());
        for vhost in vhosts {
            // Get domain name
            let domain = sqlx::query_as::<_, Domain>(
                "SELECT * FROM domains WHERE id = ?",
            )
            .bind(&vhost.domain_id)
            .fetch_optional(pool)
            .await?;

            let domain_name = domain
                .map(|d| d.domain_name)
                .unwrap_or_else(|| "unknown".to_string());

            responses.push(VirtualHostResponse {
                id: vhost.id,
                user_id: vhost.user_id,
                domain_id: vhost.domain_id,
                domain_name,
                document_root: vhost.document_root,
                php_version: vhost.php_version,
                ssl_enabled: vhost.ssl_enabled,
                force_https: vhost.force_https,
                is_active: vhost.is_active,
                created_at: vhost.created_at,
                package_specs: PackageSpecs::default(), // Use default values
            });
        }

        Ok(responses)
    }

    /// Get virtual host by ID
    pub async fn get_vhost_by_id(
        pool: &MySqlPool,
        vhost_id: &str,
        user_id: &str,
    ) -> ApiResult<VirtualHostResponse> {
        let vhost = sqlx::query_as::<_, VirtualHost>(
            "SELECT * FROM virtual_hosts WHERE id = ?",
        )
        .bind(vhost_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if vhost.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Get domain name
        let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
            .bind(&vhost.domain_id)
            .fetch_optional(pool)
            .await?;

        let domain_name = domain
            .map(|d| d.domain_name)
            .unwrap_or_else(|| "unknown".to_string());

        Ok(VirtualHostResponse {
            id: vhost.id,
            user_id: vhost.user_id,
            domain_id: vhost.domain_id,
            domain_name,
            document_root: vhost.document_root,
            php_version: vhost.php_version,
            ssl_enabled: vhost.ssl_enabled,
            force_https: vhost.force_https,
            is_active: vhost.is_active,
            created_at: vhost.created_at,
            package_specs: PackageSpecs::default(), // Use default values
        })
    }

    /// Create virtual host
    ///
    /// Secara otomatis akan men-generate konfigurasi Nginx/Apache.
    pub async fn create_vhost(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateVirtualHostRequest,
    ) -> ApiResult<VirtualHostResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // Verify domain ownership
        let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
            .bind(&request.domain_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Domain".to_string()))?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Check if vhost already exists for this domain
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM virtual_hosts WHERE domain_id = ?",
        )
        .bind(&request.domain_id)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ApiError::AlreadyExists("Virtual Host".to_string()));
        }

        // Set parameters
        let php_version = request
            .php_version
            .unwrap_or(PhpVersion::default())
            .to_string();
        
        let ssl_enabled = request.ssl_enabled.unwrap_or(false);
        let document_root = domain.document_root;
        let vhost_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Get admin email form user email (placeholder logic)
        // In reality we should fetch user email
        let admin_email = format!("webmaster@{}", domain.domain_name);

        // Insert virtual host
        sqlx::query(
            r#"
            INSERT INTO virtual_hosts (id, user_id, domain_id, document_root, admin_email, php_version, ssl_enabled, force_https, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, FALSE, TRUE, ?, ?)
            "#,
        )
        .bind(&vhost_id)
        .bind(user_id)
        .bind(&request.domain_id)
        .bind(&document_root)
        .bind(&admin_email)
        .bind(&php_version)
        .bind(ssl_enabled)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // TODO: Generate Nginx/Apache config file
        tracing::info!(
            "Virtual Host created for domain: {}",
            domain.domain_name
        );

        Self::get_vhost_by_id(pool, &vhost_id, user_id).await
    }

    /// Update virtual host config
    pub async fn update_vhost(
        pool: &MySqlPool,
        vhost_id: &str,
        user_id: &str,
        request: UpdateVirtualHostRequest,
    ) -> ApiResult<VirtualHostResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let vhost = sqlx::query_as::<_, VirtualHost>(
            "SELECT * FROM virtual_hosts WHERE id = ?",
        )
        .bind(vhost_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if vhost.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Build update values
        let php_version = request
            .php_version
            .map(|v| v.to_string())
            .unwrap_or(vhost.php_version);
        
        let ssl_enabled = request.ssl_enabled.unwrap_or(vhost.ssl_enabled);
        let force_https = request.force_https.unwrap_or(vhost.force_https);
        let is_active = request.is_active.unwrap_or(vhost.is_active);
        let custom_config = request.custom_config.or(vhost.custom_config);

        // Update database
        sqlx::query(
            r#"
            UPDATE virtual_hosts 
            SET php_version = ?, ssl_enabled = ?, force_https = ?, is_active = ?, custom_config = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&php_version)
        .bind(ssl_enabled)
        .bind(force_https)
        .bind(is_active)
        .bind(&custom_config)
        .bind(Utc::now())
        .bind(vhost_id)
        .execute(pool)
        .await?;

        // TODO: Regenerate Nginx/Apache config and reload
        tracing::info!("Virtual Host updated: {}", vhost_id);

        Self::get_vhost_by_id(pool, vhost_id, user_id).await
    }

    /// Delete virtual host
    pub async fn delete_vhost(
        pool: &MySqlPool,
        vhost_id: &str,
        user_id: &str,
    ) -> ApiResult<()> {
        let vhost = sqlx::query_as::<_, VirtualHost>(
            "SELECT * FROM virtual_hosts WHERE id = ?",
        )
        .bind(vhost_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if vhost.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Delete SSL certificates first
        sqlx::query("DELETE FROM ssl_certificates WHERE vhost_id = ?")
            .bind(vhost_id)
            .execute(pool)
            .await?;

        // Delete virtual host record
        sqlx::query("DELETE FROM virtual_hosts WHERE id = ?")
            .bind(vhost_id)
            .execute(pool)
            .await?;

        // TODO: Remove config files and reload web server
        tracing::info!("Virtual Host deleted: {}", vhost_id);

        Ok(())
    }

    // ==========================================
    // SSL CERTIFICATE OPERATIONS
    // ==========================================

    /// Request SSL Certificate (Let's Encrypt simulation)
    pub async fn request_ssl(
        pool: &MySqlPool,
        user_id: &str,
        request: RequestSslRequest,
    ) -> ApiResult<SslCertificateResponse> {
        request
            .validate()
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;

        let vhost = sqlx::query_as::<_, VirtualHost>(
            "SELECT * FROM virtual_hosts WHERE id = ?",
        )
        .bind(&request.vhost_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if vhost.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // Simulation: Generate dummy certificate
        let cert_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(90); // 90 days validity

        let dummy_cert = "-----BEGIN CERTIFICATE-----\n(Simulated Certificate)\n-----END CERTIFICATE-----";
        let dummy_key = "-----BEGIN PRIVATE KEY-----\n(Simulated Key)\n-----END PRIVATE KEY-----";

        // Delete existing cert if any
        sqlx::query("DELETE FROM ssl_certificates WHERE vhost_id = ?")
            .bind(&request.vhost_id)
            .execute(pool)
            .await?;

        // Insert new cert
        sqlx::query(
            r#"
            INSERT INTO ssl_certificates (id, vhost_id, user_id, provider, cert_pem, key_pem, expires_at, auto_renew, created_at)
            VALUES (?, ?, ?, 'LetsEncrypt', ?, ?, ?, TRUE, ?)
            "#
        )
        .bind(&cert_id)
        .bind(&request.vhost_id)
        .bind(user_id)
        .bind(dummy_cert)
        .bind(dummy_key)
        .bind(expires_at)
        .bind(now)
        .execute(pool)
        .await?;

        // Enable SSL on vhost
        sqlx::query("UPDATE virtual_hosts SET ssl_enabled = TRUE WHERE id = ?")
            .bind(&request.vhost_id)
            .execute(pool)
            .await?;

        tracing::info!("SSL Certificate requested/renewed for vhost: {}", request.vhost_id);

        let cert = SslCertificate {
            id: cert_id,
            vhost_id: request.vhost_id,
            user_id: user_id.to_string(),
            provider: "LetsEncrypt".to_string(),
            cert_pem: dummy_cert.to_string(),
            key_pem: dummy_key.to_string(),
            chain_pem: None,
            expires_at,
            auto_renew: true,
            created_at: now,
        };

        Ok(SslCertificateResponse::from(cert))
    }

    /// Get SSL Certificate status
    pub async fn get_ssl_status(
        pool: &MySqlPool,
        vhost_id: &str,
        user_id: &str,
    ) -> ApiResult<SslCertificateResponse> {
        let cert = sqlx::query_as::<_, SslCertificate>(
            "SELECT * FROM ssl_certificates WHERE vhost_id = ?",
        )
        .bind(vhost_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("SSL Certificate".to_string()))?;

        if cert.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        Ok(SslCertificateResponse::from(cert))
    }
}
