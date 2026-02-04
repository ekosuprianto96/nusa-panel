-- Migration: Add password_encrypted column for phpMyAdmin SSO
-- This column stores encrypted (not hashed) password for SSO purposes

-- Check if column exists before adding (MySQL 8.0+ compatible)
SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'database_users' 
               AND COLUMN_NAME = 'password_encrypted');

SET @sql := IF(@exist = 0, 
    'ALTER TABLE database_users ADD COLUMN password_encrypted VARCHAR(512) NULL AFTER password_hash',
    'SELECT "Column already exists"');

PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
