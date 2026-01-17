-- ========================================================
-- NusaPanel COMPLETE Database Schema (FIXED VERSION)
-- ========================================================
-- File ini adalah schema lengkap yang sudah disesuaikan dengan model backend.
-- Gunakan file ini untuk fresh install, hapus semua migration file lainnya.
-- 
-- Cara penggunaan:
-- 1. Drop database: DROP DATABASE IF EXISTS nusa_panel;
-- 2. Create database: CREATE DATABASE nusa_panel;
-- 3. Jalankan file ini: mysql -u root nusa_panel < 001_complete_schema_fixed.sql
-- ========================================================

-- ============================================
-- 1. USERS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(30) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(50),
    last_name VARCHAR(50),
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP NULL,
    
    INDEX idx_users_username (username),
    INDEX idx_users_email (email),
    INDEX idx_users_role (role),
    INDEX idx_users_status (status),
    INDEX idx_users_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 2. DOMAINS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS domains (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    domain_name VARCHAR(255) NOT NULL UNIQUE,
    document_root VARCHAR(500) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    ssl_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_domains_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_domains_user_id (user_id),
    INDEX idx_domains_domain_name (domain_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 3. SUBDOMAINS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS subdomains (
    id VARCHAR(36) PRIMARY KEY,
    domain_id VARCHAR(36) NOT NULL,
    subdomain_name VARCHAR(63) NOT NULL,
    document_root VARCHAR(500) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE KEY unique_subdomain (domain_id, subdomain_name),
    CONSTRAINT fk_subdomains_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    INDEX idx_subdomains_domain_id (domain_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 4. DNS_RECORDS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS dns_records (
    id VARCHAR(36) PRIMARY KEY,
    domain_id VARCHAR(36) NOT NULL,
    record_type VARCHAR(10) NOT NULL,
    name VARCHAR(255) NOT NULL,
    value VARCHAR(500) NOT NULL,
    ttl INT NOT NULL DEFAULT 3600,
    priority INT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_dns_records_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    INDEX idx_dns_records_domain_id (domain_id),
    INDEX idx_dns_records_type (record_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 5. MANAGED_DATABASES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS managed_databases (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    db_name VARCHAR(64) NOT NULL UNIQUE,
    description VARCHAR(255),
    size_bytes BIGINT NOT NULL DEFAULT 0,
    charset VARCHAR(32) NOT NULL DEFAULT 'utf8mb4',
    collation VARCHAR(64) NOT NULL DEFAULT 'utf8mb4_unicode_ci',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_managed_databases_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_managed_databases_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 6. DATABASE_USERS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS database_users (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    database_id VARCHAR(36) NULL,
    db_username VARCHAR(32) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL DEFAULT '%',
    privileges VARCHAR(255) DEFAULT 'ALL',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_database_users_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_database_users_db FOREIGN KEY (database_id) REFERENCES managed_databases(id) ON DELETE SET NULL,
    INDEX idx_database_users_user_id (user_id),
    INDEX idx_database_users_database_id (database_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 7. EMAIL_ACCOUNTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS email_accounts (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    domain_id VARCHAR(36) NOT NULL,
    email_address VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    quota_bytes BIGINT NOT NULL DEFAULT 1073741824,
    used_bytes BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    last_login TIMESTAMP NULL,
    
    CONSTRAINT fk_email_accounts_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_email_accounts_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    INDEX idx_email_accounts_user_id (user_id),
    INDEX idx_email_accounts_domain_id (domain_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 8. EMAIL_FORWARDERS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS email_forwarders (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    domain_id VARCHAR(36) NOT NULL,
    source_email VARCHAR(255) NOT NULL,
    destination_email VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_email_forwarders_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_email_forwarders_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    INDEX idx_email_forwarders_user_id (user_id),
    INDEX idx_email_forwarders_domain_id (domain_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 9. AUTORESPONDERS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS autoresponders (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    email_account_id VARCHAR(36) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    start_date TIMESTAMP NULL,
    end_date TIMESTAMP NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_autoresponders_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_autoresponders_account FOREIGN KEY (email_account_id) REFERENCES email_accounts(id) ON DELETE CASCADE,
    INDEX idx_autoresponders_user_id (user_id),
    INDEX idx_autoresponders_account_id (email_account_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 10. FTP_ACCOUNTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS ftp_accounts (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    ftp_username VARCHAR(64) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    home_directory VARCHAR(500) NOT NULL,
    quota_bytes BIGINT DEFAULT 0,
    used_bytes BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    last_login TIMESTAMP NULL,
    
    CONSTRAINT fk_ftp_accounts_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_ftp_accounts_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 11. CRON_JOBS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS cron_jobs (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    schedule VARCHAR(100) NOT NULL,
    command VARCHAR(1000) NOT NULL,
    description VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    email_notification VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_cron_jobs_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_cron_jobs_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 12. SYSTEM_BACKUPS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS system_backups (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    filename VARCHAR(500),
    size_bytes BIGINT DEFAULT 0,
    backup_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP NULL,
    
    CONSTRAINT fk_system_backups_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_system_backups_user_id (user_id),
    INDEX idx_system_backups_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 13. VIRTUAL_HOSTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS virtual_hosts (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    domain_id VARCHAR(36) NOT NULL,
    document_root VARCHAR(500) NOT NULL,
    admin_email VARCHAR(255) NOT NULL,
    php_version VARCHAR(20) DEFAULT '8.2',
    ssl_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    force_https BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_virtual_hosts_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_virtual_hosts_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    INDEX idx_virtual_hosts_user_id (user_id),
    INDEX idx_virtual_hosts_domain_id (domain_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 14. SSL_CERTIFICATES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS ssl_certificates (
    id VARCHAR(36) PRIMARY KEY,
    vhost_id VARCHAR(36) NULL,
    user_id VARCHAR(36) NOT NULL,
    provider VARCHAR(50) DEFAULT 'custom',
    cert_pem TEXT NOT NULL,
    key_pem TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_ssl_certificates_vhost FOREIGN KEY (vhost_id) REFERENCES virtual_hosts(id) ON DELETE CASCADE,
    CONSTRAINT fk_ssl_certificates_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_ssl_certificates_vhost_id (vhost_id),
    INDEX idx_ssl_certificates_user_id (user_id),
    INDEX idx_ssl_certificates_expires_at (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 15. BLOCKED_IPS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS blocked_ips (
    id VARCHAR(36) PRIMARY KEY,
    ip_address VARCHAR(45) NOT NULL,
    reason VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_blocked_ips_ip_address (ip_address)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 16. SSH_KEYS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS ssh_keys (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    public_key TEXT NOT NULL,
    label VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_ssh_keys_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_ssh_keys_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 17. REDIS_INSTANCES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS redis_instances (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    password VARCHAR(255) NOT NULL,
    max_memory VARCHAR(50) NOT NULL DEFAULT '64mb',
    socket_path VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_redis_instances_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_redis_instances_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 18. ACTIVITY_LOGS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS activity_logs (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id VARCHAR(36),
    ip_address VARCHAR(45),
    user_agent VARCHAR(500),
    details JSON,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_activity_logs_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_activity_logs_user_id (user_id),
    INDEX idx_activity_logs_action (action),
    INDEX idx_activity_logs_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- DEFAULT ADMIN USER
-- ============================================
-- Password: Admin@123 (Ganti hash ini di production!)
INSERT INTO users (id, username, email, password_hash, first_name, last_name, role, status, created_at, updated_at)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'admin',
    'admin@localhost',
    '$argon2id$v=19$m=19456,t=2,p=1$mTjaFovuWQ6+RcUwhEy7Xg$YrO7sjXIHdDrDO4A9wVPKBnCjhhDDsMZpNEWpTv7/n+o',
    'System',
    'Administrator',
    'admin',
    'active',
    NOW(),
    NOW()
) ON DUPLICATE KEY UPDATE id = id;
