-- 创建模板版本表
CREATE TABLE IF NOT EXISTS template_versions (
    id BIGINT PRIMARY KEY AUTO_INCREMENT COMMENT '版本ID',
    template_id BIGINT NOT NULL COMMENT '模板ID',
    version VARCHAR(50) NOT NULL COMMENT '版本号（如 v1.0.0）',
    commit_hash VARCHAR(100) COMMENT 'Git commit hash',
    commit_message TEXT COMMENT 'Git提交信息',
    changelog TEXT COMMENT '发布日志',
    is_latest BOOLEAN DEFAULT FALSE COMMENT '是否为最新版本',
    is_deprecated BOOLEAN DEFAULT FALSE COMMENT '是否已弃用',
    creator_id BIGINT COMMENT '创建者ID',
    creator_name VARCHAR(100) COMMENT '创建者名称',
    file_count INT DEFAULT 0 COMMENT '文件数量',
    total_size BIGINT DEFAULT 0 COMMENT '总大小（字节）',
    storage_path VARCHAR(500) COMMENT '存储路径（如 releases/123/v1.0.0）',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',

    UNIQUE KEY uk_template_version (template_id, version),
    KEY idx_template_id (template_id),
    KEY idx_is_latest (is_latest),
    KEY idx_created_at (created_at),
    KEY idx_is_deprecated (is_deprecated)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='模板版本表';

-- 添加索引注释
ALTER TABLE template_versions COMMENT = '模板版本表：存储模板的发布版本信息，支持版本管理和快速回滚';
