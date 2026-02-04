-- ========================================================
-- NusaPanel ModSecurity Audit Logs - Add user_id
-- ========================================================

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS
               WHERE TABLE_SCHEMA = DATABASE()
               AND TABLE_NAME = 'modsecurity_audit_logs'
               AND COLUMN_NAME = 'user_id');
SET @sql := IF(@exist = 0,
    'ALTER TABLE modsecurity_audit_logs ADD COLUMN user_id VARCHAR(36) NULL AFTER id',
    'SELECT \"Column user_id already exists\"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.STATISTICS
               WHERE TABLE_SCHEMA = DATABASE()
               AND TABLE_NAME = 'modsecurity_audit_logs'
               AND INDEX_NAME = 'idx_modsec_audit_user');
SET @sql := IF(@exist = 0,
    'CREATE INDEX idx_modsec_audit_user ON modsecurity_audit_logs(user_id)',
    'SELECT \"Index idx_modsec_audit_user already exists\"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @exist := (SELECT COUNT(*) FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS
               WHERE TABLE_SCHEMA = DATABASE()
               AND TABLE_NAME = 'modsecurity_audit_logs'
               AND CONSTRAINT_NAME = 'fk_modsec_audit_user');
SET @sql := IF(@exist = 0,
    'ALTER TABLE modsecurity_audit_logs ADD CONSTRAINT fk_modsec_audit_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL',
    'SELECT \"Constraint fk_modsec_audit_user already exists\"');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;
