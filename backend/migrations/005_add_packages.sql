-- ========================================================
-- NusaPanel Package Management Schema
-- ========================================================
-- Menambahkan tabel packages dan relasi ke users
-- ========================================================

-- ============================================
-- 1. PACKAGES TABLE
-- ============================================
-- Menyimpan daftar package hosting dengan limits
CREATE TABLE IF NOT EXISTS packages (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    
    -- Resource Limits
    disk_quota_mb BIGINT NOT NULL DEFAULT 1024,           -- Default 1GB
    bandwidth_mb BIGINT NOT NULL DEFAULT 10240,           -- Default 10GB
    
    -- Feature Limits
    max_domains INT NOT NULL DEFAULT 1,
    max_subdomains INT NOT NULL DEFAULT 5,
    max_databases INT NOT NULL DEFAULT 1,
    max_email_accounts INT NOT NULL DEFAULT 5,
    max_ftp_accounts INT NOT NULL DEFAULT 2,
    max_cron_jobs INT NOT NULL DEFAULT 2,
    
    -- Pricing
    price_monthly DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    price_yearly DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    
    -- Status & Metadata
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INT NOT NULL DEFAULT 0,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_packages_name (name),
    INDEX idx_packages_is_active (is_active),
    INDEX idx_packages_sort_order (sort_order)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 2. ALTER USERS TABLE
-- ============================================
-- Menambahkan relasi package dan created_by ke users

-- Tambah kolom package_id (nullable untuk backward compatibility)
-- Kolom-kolom tambahan (idempotent untuk mencegah duplicate column)
SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND COLUMN_NAME = 'package_id');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD COLUMN package_id VARCHAR(36) NULL AFTER status',
    'SELECT "Column package_id already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND COLUMN_NAME = 'created_by');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD COLUMN created_by VARCHAR(36) NULL AFTER package_id',
    'SELECT "Column created_by already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND COLUMN_NAME = 'company');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD COLUMN company VARCHAR(100) NULL AFTER last_name',
    'SELECT "Column company already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND COLUMN_NAME = 'phone');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD COLUMN phone VARCHAR(20) NULL AFTER company',
    'SELECT "Column phone already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND COLUMN_NAME = 'address');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD COLUMN address TEXT NULL AFTER phone',
    'SELECT "Column address already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

-- Constraints (idempotent)
SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND CONSTRAINT_NAME = 'fk_users_package');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD CONSTRAINT fk_users_package FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE SET NULL',
    'SELECT "Constraint fk_users_package already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND CONSTRAINT_NAME = 'fk_users_created_by');
SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD CONSTRAINT fk_users_created_by FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL',
    'SELECT "Constraint fk_users_created_by already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

-- Indexes (idempotent)
SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.STATISTICS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND INDEX_NAME = 'idx_users_package_id');
SET @sql := IF(@exist = 0, 
    'CREATE INDEX idx_users_package_id ON users(package_id)',
    'SELECT "Index idx_users_package_id already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.STATISTICS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND INDEX_NAME = 'idx_users_created_by');
SET @sql := IF(@exist = 0, 
    'CREATE INDEX idx_users_created_by ON users(created_by)',
    'SELECT "Index idx_users_created_by already exists"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

-- ============================================
-- 3. DEFAULT PACKAGES
-- ============================================
-- Insert default hosting packages
INSERT INTO packages (id, name, description, disk_quota_mb, bandwidth_mb, max_domains, max_subdomains, max_databases, max_email_accounts, max_ftp_accounts, max_cron_jobs, price_monthly, price_yearly, is_active, is_default, sort_order) VALUES
(
    'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d',
    'Free',
    'Paket gratis untuk testing dan development',
    512,        -- 512MB disk
    5120,       -- 5GB bandwidth
    1,          -- 1 domain
    2,          -- 2 subdomains
    1,          -- 1 database
    2,          -- 2 email
    1,          -- 1 FTP
    1,          -- 1 cron
    0.00,       -- Free
    0.00,
    TRUE,
    TRUE,       -- Default package
    1
),
(
    'b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e',
    'Basic',
    'Paket dasar untuk website personal dan blog',
    2048,       -- 2GB disk
    20480,      -- 20GB bandwidth
    1,          -- 1 domain
    5,          -- 5 subdomains
    2,          -- 2 databases
    10,         -- 10 emails
    2,          -- 2 FTP
    3,          -- 3 crons
    25000.00,   -- Rp 25.000/bulan
    250000.00,  -- Rp 250.000/tahun
    TRUE,
    FALSE,
    2
),
(
    'c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f',
    'Professional',
    'Paket profesional untuk bisnis kecil menengah',
    10240,      -- 10GB disk
    102400,     -- 100GB bandwidth
    5,          -- 5 domains
    25,         -- 25 subdomains
    10,         -- 10 databases
    50,         -- 50 emails
    10,         -- 10 FTP
    10,         -- 10 crons
    75000.00,   -- Rp 75.000/bulan
    750000.00,  -- Rp 750.000/tahun
    TRUE,
    FALSE,
    3
),
(
    'd4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f8a',
    'Enterprise',
    'Paket enterprise dengan resource unlimited',
    51200,      -- 50GB disk
    0,          -- Unlimited bandwidth (0 = unlimited)
    0,          -- Unlimited domains
    0,          -- Unlimited subdomains
    0,          -- Unlimited databases
    0,          -- Unlimited emails
    0,          -- Unlimited FTP
    0,          -- Unlimited crons
    200000.00,  -- Rp 200.000/bulan
    2000000.00, -- Rp 2.000.000/tahun
    TRUE,
    FALSE,
    4
);
