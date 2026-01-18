-- ============================================
-- DOMAIN FEATURES: REDIRECTS & ALIASES
-- ============================================

-- ============================================
-- 19. REDIRECTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS redirects (
    id VARCHAR(36) PRIMARY KEY,
    domain_id VARCHAR(36) NOT NULL,
    source_path VARCHAR(500) NOT NULL,
    destination_url VARCHAR(500) NOT NULL,
    type VARCHAR(10) NOT NULL DEFAULT '301',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_redirects_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    INDEX idx_redirects_domain_id (domain_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================
-- 20. DOMAIN_ALIASES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS domain_aliases (
    id VARCHAR(36) PRIMARY KEY,
    domain_id VARCHAR(36) NOT NULL,
    alias_domain VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT fk_domain_aliases_domain FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    INDEX idx_domain_aliases_domain_id (domain_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
