CREATE TABLE IF NOT EXISTS system_settings (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    `group` VARCHAR(100) NOT NULL DEFAULT 'general' COMMENT '设置分组',
    `key` VARCHAR(200) NOT NULL COMMENT '设置键名',
    value TEXT COMMENT '设置值',
    description VARCHAR(500) COMMENT '设置项描述',
    sort INT DEFAULT 0 COMMENT '排序',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_group_key (`group`, `key`),
    INDEX idx_group (`group`)
);
