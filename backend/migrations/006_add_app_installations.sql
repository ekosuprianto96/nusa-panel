-- ========================================================
-- NusaPanel App Installations Schema
-- ========================================================

CREATE TABLE IF NOT EXISTS app_installations (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    domain_id VARCHAR(36) NOT NULL,
    app_type VARCHAR(50) NOT NULL,
    version VARCHAR(50) NOT NULL DEFAULT 'latest',
    install_path VARCHAR(255) NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    CONSTRAINT fk_app_installations_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_app_installations_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,

    INDEX idx_app_installations_user_id (user_id),
    INDEX idx_app_installations_domain_id (domain_id),
    INDEX idx_app_installations_app_type (app_type),
    INDEX idx_app_installations_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
