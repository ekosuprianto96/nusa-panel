-- ============================================
-- DDNS HOSTS
-- ============================================

CREATE TABLE IF NOT EXISTS ddns_hosts (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    domain_id VARCHAR(36) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    description VARCHAR(255) NULL,
    api_key VARCHAR(128) NOT NULL,
    last_ip VARCHAR(64) NULL,
    last_updated_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT fk_ddns_hosts_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_ddns_hosts_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    UNIQUE KEY uq_ddns_hosts_hostname (hostname),
    UNIQUE KEY uq_ddns_hosts_api_key (api_key),
    INDEX idx_ddns_hosts_user_id (user_id),
    INDEX idx_ddns_hosts_domain_id (domain_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
