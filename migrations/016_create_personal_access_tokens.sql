CREATE TABLE IF NOT EXISTS personal_access_tokens (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id BIGINT NOT NULL,
    name VARCHAR(100) NOT NULL COMMENT '令牌名称，如 CLI、桌面端',
    token_hash VARCHAR(255) NOT NULL UNIQUE COMMENT '令牌哈希',
    token_prefix VARCHAR(20) NOT NULL COMMENT '令牌前缀，用于展示',
    last_used_at DATETIME NULL,
    expires_at DATETIME NULL COMMENT '过期时间，NULL 表示永不过期',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id),
    INDEX idx_token_hash (token_hash)
);
