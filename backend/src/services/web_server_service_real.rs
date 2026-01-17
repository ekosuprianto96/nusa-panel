//! # Web Server Service (REAL IMPLEMENTATION)
//!
//! Implementasi nyata untuk manajemen Nginx/Apache dan SSL Certbot.
//! Gunakan file ini menggantikan `web_server_service.rs` di production.

use chrono::Utc;
use sqlx::MySqlPool;
use std::fs;
use std::process::Command;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{ApiError, ApiResult};
use crate::models::{
    CreateVirtualHostRequest, Domain, PackageSpecs, PhpVersion, RequestSslRequest,
    SslCertificate, SslCertificateResponse, VirtualHost, VirtualHostResponse,
};

pub struct WebServerServiceReal;

impl WebServerServiceReal {
    // ==========================================
    // VIRTUAL HOST OPERATIONS (REAL)
    // ==========================================

    pub async fn create_vhost(
        pool: &MySqlPool,
        user_id: &str,
        request: CreateVirtualHostRequest,
    ) -> ApiResult<VirtualHostResponse> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 1. Fetch domain info
        let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
            .bind(&request.domain_id)
            .fetch_one(pool)
            .await?;

        if domain.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        // 2. Prepare Config Variables
        let php_ver = request.php_version.clone().unwrap_or_default().to_string();
        let server_type = request.web_server_type;
        let vhost_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // 3. Generate Config File based on Type
        match server_type {
            
            // --- NGINX ---
            crate::models::WebServerType::Nginx => {
                let config_path = format!("/etc/nginx/sites-available/{}", domain.domain_name);
                let link_path = format!("/etc/nginx/sites-enabled/{}", domain.domain_name);
                
                let nginx_config = format!(
                    r#"
server {{
    listen 80;
    server_name {0} www.{0};
    root {1};
    index index.php index.html;

    location / {{
        try_files $uri $uri/ =404;
    }}

    location ~ \.php$ {{
        include snippets/fastcgi-php.conf;
        fastcgi_pass unix:/run/php/php{2}-fpm.sock;
    }}
}}
                    "#,
                    domain.domain_name,
                    domain.document_root,
                    php_ver
                );

                fs::write(&config_path, nginx_config)
                    .map_err(|e| ApiError::InternalError(format!("Nginx write error: {}", e)))?;

                if !std::path::Path::new(&link_path).exists() {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&config_path, &link_path)
                        .map_err(|e| ApiError::InternalError(format!("Nginx symlink error: {}", e)))?;
                    
                    #[cfg(not(unix))]
                    tracing::warn!("Symlink creation skipped on non-unix system: {} -> {}", config_path, link_path);
                }

                Command::new("systemctl").arg("reload").arg("nginx").output()?;
            },

            // --- APACHE ---
            crate::models::WebServerType::Apache => {
                let config_path = format!("/etc/apache2/sites-available/{}.conf", domain.domain_name);
                
                let apache_config = format!(
                    r#"
<VirtualHost *:80>
    ServerName {0}
    ServerAlias www.{0}
    DocumentRoot {1}

    <Directory {1}>
        Options Indexes FollowSymLinks
        AllowOverride All
        Require all granted
    </Directory>

    <FilesMatch \.php$>
        SetHandler "proxy:unix:/run/php/php{2}-fpm.sock|fcgi://localhost"
    </FilesMatch>

    ErrorLog ${{APACHE_LOG_DIR}}/{0}_error.log
    CustomLog ${{APACHE_LOG_DIR}}/{0}_access.log combined
</VirtualHost>
                    "#,
                    domain.domain_name,
                    domain.document_root,
                    php_ver
                );

                fs::write(&config_path, apache_config)
                    .map_err(|e| ApiError::InternalError(format!("Apache write error: {}", e)))?;

                // a2ensite domain.com.conf
                Command::new("a2ensite").arg(format!("{}.conf", domain.domain_name)).output()?;
                
                Command::new("systemctl").arg("reload").arg("apache2").output()?;
            },

            // --- OPENLITESPEED ---
            crate::models::WebServerType::OpenLiteSpeed => {
                // OLS uses a single XML config or included files. Logic ini lebih kompleks di real world.
                // Untuk MVP, kita asumsikan OLS dikonfigurasi utk include file dari folder vhosts
                
                let config_path = format!("/usr/local/lsws/conf/vhosts/{}.conf", domain.domain_name);
                
                let ols_config = format!(
                    r#"
virtualhost {0} {{
  vhRoot                  {1}
  configFile              $SERVER_ROOT/conf/vhosts/{0}/vhconf.conf
  allowSymbolLink         1
  enableScript            1
  restrained              1

  scripthandler  {{
    add                     lsapi:lsphp{2} php
  }}
}}
                    "#,
                    domain.domain_name,
                    domain.document_root,
                    php_ver.replace(".", "") // lsphp82
                );
                
                // Note: OLS butuh mapping listener juga. Ini simplifikasi.
                fs::write(&config_path, ols_config)
                    .map_err(|e| ApiError::InternalError(format!("OLS write error: {}", e)))?;

                // Graceful restart OLS
                Command::new("/usr/local/lsws/bin/lswsctrl").arg("restart").output()?;
            }
        }

        // 4. Save to Database
        sqlx::query(
            r#"
            INSERT INTO virtual_hosts (id, user_id, domain_id, document_root, admin_email, php_version, ssl_enabled, force_https, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'admin@example.com', ?, FALSE, FALSE, TRUE, ?, ?)
            "#,
        )
        .bind(&vhost_id)
        .bind(user_id)
        .bind(&request.domain_id)
        .bind(&domain.document_root)
        // .bind(web_server_type) // TODO: Add column 'server_type' to DB schema if needed permanent persistence
        .bind(&php_ver)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(VirtualHostResponse {
            id: vhost_id,
            user_id: user_id.to_string(),
            domain_id: request.domain_id,
            domain_name: domain.domain_name,
            document_root: domain.document_root,
            php_version: php_ver,
            ssl_enabled: false,
            force_https: false,
            is_active: true,
            created_at: now,
            package_specs: PackageSpecs::default(),
        })
    }

    /// Get all vhosts for user
    pub async fn get_user_vhosts(pool: &MySqlPool, user_id: &str) -> ApiResult<Vec<VirtualHostResponse>> {
        let vhosts = sqlx::query_as::<_, VirtualHost>(
            "SELECT * FROM virtual_hosts WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut responses = Vec::new();
        for v in vhosts {
            let domain_name = sqlx::query_scalar::<_, String>("SELECT domain_name FROM domains WHERE id = ?")
                .bind(&v.domain_id)
                .fetch_one(pool)
                .await?;
                
            responses.push(VirtualHostResponse {
                id: v.id,
                user_id: v.user_id,
                domain_id: v.domain_id,
                domain_name,
                document_root: v.document_root,
                php_version: v.php_version,
                ssl_enabled: v.ssl_enabled,
                force_https: v.force_https,
                is_active: v.is_active,
                created_at: v.created_at,
                package_specs: PackageSpecs::default(),
            });
        }
        Ok(responses)
    }

    /// Get vhost by ID
    pub async fn get_vhost_by_id(pool: &MySqlPool, id: &str, user_id: &str) -> ApiResult<VirtualHostResponse> {
        let v = sqlx::query_as::<_, VirtualHost>("SELECT * FROM virtual_hosts WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if v.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let domain_name = sqlx::query_scalar::<_, String>("SELECT domain_name FROM domains WHERE id = ?")
            .bind(&v.domain_id)
            .fetch_one(pool)
            .await?;

        Ok(VirtualHostResponse {
            id: v.id,
            user_id: v.user_id,
            domain_id: v.domain_id,
            domain_name,
            document_root: v.document_root,
            php_version: v.php_version,
            ssl_enabled: v.ssl_enabled,
            force_https: v.force_https,
            is_active: v.is_active,
            created_at: v.created_at,
            package_specs: PackageSpecs::default(),
        })
    }

    /// Update vhost
    pub async fn update_vhost(
        pool: &MySqlPool,
        id: &str,
        user_id: &str,
        request: crate::models::UpdateVirtualHostRequest,
    ) -> ApiResult<VirtualHostResponse> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;
        
        let v = sqlx::query_as::<_, VirtualHost>("SELECT * FROM virtual_hosts WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if v.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let php_ver = request.php_version.map(|v| v.to_string()).unwrap_or(v.php_version);
        let force_https = request.force_https.unwrap_or(v.force_https);
        let is_active = request.is_active.unwrap_or(v.is_active);

        sqlx::query(
            "UPDATE virtual_hosts SET php_version = ?, force_https = ?, is_active = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&php_ver).bind(force_https).bind(is_active).bind(Utc::now()).bind(id)
        .execute(pool)
        .await?;

        // Re-generate config if needed
        // ... Logic re-generate Nginx/Apache config ...

        Self::get_vhost_by_id(pool, id, user_id).await
    }

    /// Delete vhost
    pub async fn delete_vhost(pool: &MySqlPool, id: &str, user_id: &str) -> ApiResult<()> {
        let v = sqlx::query_as::<_, VirtualHost>("SELECT * FROM virtual_hosts WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if v.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let domain_name = sqlx::query_scalar::<_, String>("SELECT domain_name FROM domains WHERE id = ?")
            .bind(&v.domain_id)
            .fetch_one(pool)
            .await?;

        // 1. Remove Configs
        let _ = fs::remove_file(format!("/etc/nginx/sites-available/{}", domain_name));
        let _ = fs::remove_file(format!("/etc/nginx/sites-enabled/{}", domain_name));
        let _ = Command::new("systemctl").arg("reload").arg("nginx").output();

        // 2. Delete from DB
        sqlx::query("DELETE FROM virtual_hosts WHERE id = ?").bind(id).execute(pool).await?;

        Ok(())
    }
    
    // Helper untuk Install PHP Version (Placeholder)
    // Di Ubuntu bisa pakai: apt install php8.x-fpm
    pub async fn install_php_version(_version: PhpVersion) -> ApiResult<()> {
        Ok(()) // Placeholder implementation logic apt-get
    }

    // ==========================================
    // SSL CERTIFICATE OPERATIONS (REAL)
    // ==========================================

    pub async fn request_ssl(
        pool: &MySqlPool,
        user_id: &str,
        request: RequestSslRequest,
    ) -> ApiResult<SslCertificateResponse> {
        request.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

        // 1. Fetch vhost & domain details
        let vhost = sqlx::query_as::<_, VirtualHost>("SELECT * FROM virtual_hosts WHERE id = ?")
            .bind(&request.vhost_id)
            .fetch_optional(pool)
            .await?
            .ok_or(ApiError::NotFound("Virtual Host".to_string()))?;

        if vhost.user_id != user_id {
            return Err(ApiError::Forbidden);
        }

        let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
            .bind(&vhost.domain_id)
            .fetch_one(pool)
            .await?;

        // 2. Jalankan Certbot
        // certbot certonly --webroot -w /var/www/html/user/domain.com -d domain.com --non-interactive --agree-tos -m admin@example.com
        let email_arg = request.email.unwrap_or_else(|| vhost.admin_email.clone());
        
        let output = Command::new("certbot")
            .arg("certonly")
            .arg("--webroot")
            .arg("-w").arg(&vhost.document_root)
            .arg("-d").arg(&domain.domain_name)
            .arg("--non-interactive")
            .arg("--agree-tos")
            .arg("-m").arg(&email_arg)
            .output()?;
        
        if output.status.success() {
            // 3. Read Certificates
            let cert_path = format!("/etc/letsencrypt/live/{}/fullchain.pem", domain.domain_name);
            let key_path = format!("/etc/letsencrypt/live/{}/privkey.pem", domain.domain_name);

            let cert_pem = fs::read_to_string(&cert_path)
                .map_err(|e| ApiError::InternalError(format!("Failed to read cert: {}", e)))?;
            let key_pem = fs::read_to_string(&key_path)
                .map_err(|e| ApiError::InternalError(format!("Failed to read key: {}", e)))?;

            // 4. Update Database
            let cert_id = Uuid::new_v4().to_string();
            let now = Utc::now();
            let expires_at = now + chrono::Duration::days(90);

            // Delete existing active certs for this vhost
            sqlx::query("DELETE FROM ssl_certificates WHERE vhost_id = ?")
                .bind(&vhost.id)
                .execute(pool)
                .await?;

            sqlx::query(
                r#"
                INSERT INTO ssl_certificates (id, vhost_id, user_id, provider, cert_pem, key_pem, expires_at, auto_renew, created_at)
                VALUES (?, ?, ?, 'LetsEncrypt', ?, ?, ?, TRUE, ?)
                "#
            )
            .bind(&cert_id)
            .bind(&vhost.id)
            .bind(user_id)
            .bind(&cert_pem)
            .bind(&key_pem)
            .bind(expires_at)
            .bind(now)
            .execute(pool)
            .await?;

            // Update vhost status
            sqlx::query("UPDATE virtual_hosts SET ssl_enabled = TRUE WHERE id = ?")
                .bind(&vhost.id)
                .execute(pool)
                .await?;

            // 5. Update Nginx Config untuk support SSL
            // Kita perlu inject blok 'listen 443 ssl' dan path sertifikat
            let config_path = format!("/etc/nginx/sites-available/{}", domain.domain_name);
            let nginx_ssl_config = format!(
                r#"
server {{
    listen 80;
    server_name {0} www.{0};
    root {1};
    index index.php index.html;

    # Redirect HTTP to HTTPS
    location / {{
        return 301 https://$host$request_uri;
    }}
}}

server {{
    listen 443 ssl http2;
    server_name {0} www.{0};
    root {1};
    index index.php index.html;

    ssl_certificate {2};
    ssl_certificate_key {3};
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {{
        try_files $uri $uri/ =404;
    }}

    location ~ \.php$ {{
        include snippets/fastcgi-php.conf;
        fastcgi_pass unix:/run/php/php{4}-fpm.sock;
    }}
}}
                "#,
                domain.domain_name,
                vhost.document_root,
                cert_path,
                key_path,
                vhost.php_version
            );

            fs::write(&config_path, nginx_ssl_config)
                .map_err(|e| ApiError::InternalError(format!("Failed to update nginx config: {}", e)))?;

            // 6. Reload Nginx
            Command::new("systemctl").arg("reload").arg("nginx").output()?;
            
            // Return success
            Ok(SslCertificateResponse {
                id: cert_id,
                vhost_id: vhost.id,
                provider: "LetsEncrypt".to_string(),
                expires_at,
                auto_renew: true,
                days_remaining: 90
            })
        } else {
             Err(ApiError::InternalError(format!("Certbot failed: {}", String::from_utf8_lossy(&output.stderr))))
        }
    }

    /// Get SSL status for vhost
    pub async fn get_ssl_status(pool: &MySqlPool, vhost_id: &str, user_id: &str) -> ApiResult<SslCertificateResponse> {
        let cert = sqlx::query_as::<_, SslCertificate>(
            "SELECT * FROM ssl_certificates WHERE vhost_id = ? AND user_id = ?"
        )
        .bind(vhost_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound("SSL Certificate".to_string()))?;

        Ok(SslCertificateResponse {
            id: cert.id,
            vhost_id: cert.vhost_id,
            provider: cert.provider,
            expires_at: cert.expires_at,
            auto_renew: cert.auto_renew,
            days_remaining: (cert.expires_at - Utc::now()).num_days(),
        })
    }
}
