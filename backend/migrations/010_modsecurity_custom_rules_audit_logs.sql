-- ========================================================
-- NusaPanel ModSecurity Custom Rules & Audit Logs
-- ========================================================

-- ============================================
-- 1. CUSTOM RULES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS modsecurity_custom_rules (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(120) NOT NULL,
    description TEXT,
    rule_content TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    INDEX idx_modsec_custom_rules_enabled (enabled),
    INDEX idx_modsec_custom_rules_name (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 2. AUDIT LOGS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS modsecurity_audit_logs (
    id VARCHAR(36) PRIMARY KEY,
    domain_id VARCHAR(36) NULL,
    rule_id VARCHAR(50) NULL,
    custom_rule_id VARCHAR(36) NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    message VARCHAR(255) NOT NULL,
    ip_address VARCHAR(64) NULL,
    uri VARCHAR(255) NULL,
    user_agent VARCHAR(255) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_modsec_audit_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE SET NULL,
    CONSTRAINT fk_modsec_audit_rule FOREIGN KEY (rule_id) REFERENCES modsecurity_rule_sets(id) ON DELETE SET NULL,
    CONSTRAINT fk_modsec_audit_custom_rule FOREIGN KEY (custom_rule_id) REFERENCES modsecurity_custom_rules(id) ON DELETE SET NULL,
    INDEX idx_modsec_audit_domain (domain_id),
    INDEX idx_modsec_audit_created (created_at),
    INDEX idx_modsec_audit_severity (severity)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
