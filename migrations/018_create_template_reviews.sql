-- 模板审核记录表
CREATE TABLE IF NOT EXISTS template_reviews (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    template_id BIGINT NOT NULL COMMENT '模板ID',
    reviewer_id BIGINT NOT NULL COMMENT '审核人ID',
    action VARCHAR(20) NOT NULL COMMENT 'approve=通过 reject=拒绝',
    reason VARCHAR(500) DEFAULT '' COMMENT '审核备注/拒绝原因',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_template_id (template_id),
    FOREIGN KEY (template_id) REFERENCES templates(id) ON DELETE CASCADE
);
