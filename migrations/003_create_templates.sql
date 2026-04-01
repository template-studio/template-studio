-- 创建模板主表
CREATE TABLE templates (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL COMMENT '模板名称',
    description TEXT NOT NULL COMMENT '模板描述',
    category_id BIGINT NOT NULL COMMENT '分类ID',
    is_featured INT DEFAULT 0 COMMENT '是否推荐',
    logo VARCHAR(500) COMMENT '模板logo',
    introduction TEXT COMMENT '模板介绍',
    icon VARCHAR(100) COMMENT '模板图标',
    template_type VARCHAR(50) DEFAULT 'basic' COMMENT '模板类型',
    type_config JSON COMMENT '类型配置',
    git_repo_path VARCHAR(500) NOT NULL COMMENT 'Git仓库路径',
    current_version VARCHAR(100) DEFAULT 'main' COMMENT '当前版本',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_category_id (category_id),
    INDEX idx_template_type (template_type),
    INDEX idx_created_at (created_at),
    INDEX idx_is_featured (is_featured)
);