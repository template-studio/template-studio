-- 为 PAT 令牌增加权限范围字段
ALTER TABLE personal_access_tokens
    ADD COLUMN scopes TEXT NOT NULL COMMENT '权限范围列表，JSON数组格式，如 ["template:read","template:write"]';
