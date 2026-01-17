-- Create redis_instances table
-- Jalankan: mysql -u root nusa_panel < fix_redis_table.sql

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
