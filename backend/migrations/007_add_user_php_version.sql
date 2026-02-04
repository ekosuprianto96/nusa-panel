-- ========================================================
-- Add per-user PHP version preference
-- ========================================================

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS 
               WHERE TABLE_SCHEMA = DATABASE() 
               AND TABLE_NAME = 'users' 
               AND COLUMN_NAME = 'php_version');

SET @sql := IF(@exist = 0, 
    'ALTER TABLE users ADD COLUMN php_version VARCHAR(10) NULL AFTER status',
    'SELECT "Column php_version already exists"');

PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
