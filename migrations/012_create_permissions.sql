CREATE TABLE permissions (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL UNIQUE COMMENT '如 dashboard, template, category',
    display_name VARCHAR(100) NOT NULL COMMENT '如 仪表盘, 模板管理',
    type VARCHAR(20) NOT NULL DEFAULT 'menu' COMMENT 'menu=菜单 button=按钮',
    parent_id BIGINT DEFAULT NULL,
    sort INT DEFAULT 0,
    status TINYINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_parent (parent_id)
);
