-- 操作审计日志：关键管理操作的记录
-- 记录谁在何时对什么资源做了什么（IP 与 UA 供安全追溯）
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id BIGINT NOT NULL COMMENT '操作人用户ID',
    username VARCHAR(100) NOT NULL COMMENT '操作人用户名（冗余，防用户删除后无法追溯）',
    action VARCHAR(50) NOT NULL COMMENT '动作标识（template.delete / release.publish / user.delete ...）',
    resource_type VARCHAR(50) NOT NULL COMMENT '资源类型（template / release / user / role / setting）',
    resource_id VARCHAR(64) DEFAULT NULL COMMENT '资源ID（字符串兼容非数字ID）',
    detail VARCHAR(1000) DEFAULT NULL COMMENT '补充信息（JSON：变更摘要/原因等）',
    ip VARCHAR(45) DEFAULT NULL COMMENT '来源IP（兼容IPv6）',
    user_agent VARCHAR(300) DEFAULT NULL COMMENT '请求User-Agent',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_user_id (user_id),
    INDEX idx_action (action),
    INDEX idx_resource (resource_type, resource_id),
    INDEX idx_created_at (created_at)
);
