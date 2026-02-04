-- ========================================================
-- NusaPanel Security Center Schema Updates
-- ========================================================
-- Menambahkan 2FA, audit logs, SSL metadata, dan ModSecurity settings
-- ========================================================

-- ============================================
-- 1. ALTER DOMAINS TABLE
-- ============================================
SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'domains' 
               AND COLUMN_NAME = 'ssl_status');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE domains ADD COLUMN ssl_status VARCHAR(20) NOT NULL DEFAULT ''inactive'' AFTER ssl_enabled',
    'SELECT "Column ssl_status already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'domains' 
               AND COLUMN_NAME = 'ssl_provider');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE domains ADD COLUMN ssl_provider VARCHAR(50) NULL AFTER ssl_status',
    'SELECT "Column ssl_provider already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'domains' 
               AND COLUMN_NAME = 'ssl_expiry_at');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE domains ADD COLUMN ssl_expiry_at DATETIME NULL AFTER ssl_provider',
    'SELECT "Column ssl_expiry_at already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'domains' 
               AND COLUMN_NAME = 'modsecurity_enabled');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE domains ADD COLUMN modsecurity_enabled BOOLEAN NOT NULL DEFAULT TRUE AFTER ssl_expiry_at',
    'SELECT "Column modsecurity_enabled already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

UPDATE domains
SET ssl_status = CASE WHEN ssl_enabled THEN 'active' ELSE 'inactive' END
WHERE ssl_status IS NULL OR ssl_status = '';

UPDATE domains
SET modsecurity_enabled = TRUE
WHERE modsecurity_enabled IS NULL;

-- ============================================
-- 2. USER 2FA TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS user_2fa (
    user_id VARCHAR(36) PRIMARY KEY,
    secret VARCHAR(128) NOT NULL,
    backup_codes TEXT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    enabled_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT fk_user_2fa_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_2fa_enabled (is_enabled)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 3. SECURITY ACCESS LOGS
-- ============================================
CREATE TABLE IF NOT EXISTS security_access_logs (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NULL,
    event_type VARCHAR(50) NOT NULL,
    ip_address VARCHAR(64) NULL,
    target VARCHAR(255) NULL,
    status VARCHAR(20) NULL,
    user_agent VARCHAR(255) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_security_logs_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_security_logs_user (user_id),
    INDEX idx_security_logs_event (event_type),
    INDEX idx_security_logs_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 4. MODSECURITY SETTINGS
-- ============================================
CREATE TABLE IF NOT EXISTS modsecurity_settings (
    id INT PRIMARY KEY,
    main_engine BOOLEAN NOT NULL DEFAULT TRUE,
    paranoia_level INT NOT NULL DEFAULT 2,
    anomaly_threshold INT NOT NULL DEFAULT 5,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO modsecurity_settings (id, main_engine, paranoia_level, anomaly_threshold)
SELECT 1, TRUE, 2, 5
WHERE NOT EXISTS (SELECT 1 FROM modsecurity_settings WHERE id = 1);

-- ============================================
-- 5. MODSECURITY RULE SETS
-- ============================================
CREATE TABLE IF NOT EXISTS modsecurity_rule_sets (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(120) NOT NULL,
    description TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO modsecurity_rule_sets (id, name, description, enabled) VALUES
('sqli', 'SQL Injection (SQLi) Protection', 'Rules to detect and prevent database manipulation.', TRUE),
('xss', 'Cross-Site Scripting (XSS)', 'Prevents malicious client-side script injection.', TRUE),
('lfi', 'Local File Inclusion (LFI)', 'Stops unauthorized local file access.', TRUE),
('scanner', 'Scanner Detection', 'Blocks vulnerability scanners and probes.', FALSE)
ON DUPLICATE KEY UPDATE id = id;
