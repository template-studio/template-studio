-- 创建编程语言表
CREATE TABLE languages (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL COMMENT '语言名称',
    display_name VARCHAR(100) NOT NULL COMMENT '显示名称',
    code VARCHAR(50) NOT NULL COMMENT '语言代码',
    icon VARCHAR(500) COMMENT '语言图标',
    color VARCHAR(20) COMMENT '颜色',
    is_popular INT DEFAULT 0 COMMENT '是否热门',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_is_popular (is_popular)
);