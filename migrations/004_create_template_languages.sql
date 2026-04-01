-- 创建模板语言关联表
CREATE TABLE template_languages (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    template_id BIGINT NOT NULL COMMENT '模板ID',
    language_id BIGINT NOT NULL COMMENT '语言ID',
    is_primary INT DEFAULT 0 COMMENT '是否主要语言',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_template_language (template_id, language_id),
    INDEX idx_language_id (language_id)
);