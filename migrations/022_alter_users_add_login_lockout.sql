-- 账号级登录失败锁定：失败计数与锁定截止时间
-- 配合 auth_service 的锁定策略（连续失败 5 次锁 15 分钟）
ALTER TABLE users
    ADD COLUMN failed_login_count INT NOT NULL DEFAULT 0 COMMENT '连续登录失败次数',
    ADD COLUMN locked_until DATETIME DEFAULT NULL COMMENT '锁定截止时间（NULL 未锁定）';
