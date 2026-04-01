-- 创建变量预设表
CREATE TABLE var_presets (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL COMMENT '变量预设名称',
    display_name VARCHAR(100) NOT NULL COMMENT '显示名称',
    description TEXT COMMENT '描述',
    category VARCHAR(50) NOT NULL COMMENT '分类',
    icon VARCHAR(500) COMMENT '图标',
    sort INT DEFAULT 0 COMMENT '排序',
    is_enabled INT DEFAULT 1 COMMENT '是否启用',
    version VARCHAR(20) DEFAULT '1.0' COMMENT '版本',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_category (category),
    INDEX idx_is_enabled (is_enabled),
    INDEX idx_sort (sort)
);