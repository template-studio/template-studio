-- 项目表规范配置表
CREATE TABLE table_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,

    -- 主键规范
    pk_enabled BOOLEAN DEFAULT 1,
    pk_field_name TEXT DEFAULT 'id',
    pk_field_type TEXT DEFAULT 'BIGINT',
    pk_auto_increment BOOLEAN DEFAULT 1,
    pk_comment TEXT,

    -- 审计字段配置
    audit_enabled BOOLEAN DEFAULT 1,
    audit_fields TEXT, -- JSON 数组

    -- 软删除字段配置
    soft_delete_enabled BOOLEAN DEFAULT 0,
    soft_delete_field TEXT DEFAULT 'deleted_at',
    soft_delete_field_type TEXT DEFAULT 'TIMESTAMP',
    soft_delete_nullable BOOLEAN DEFAULT 1,
    soft_delete_default TEXT,
    soft_delete_comment TEXT,

    -- 命名规范
    boolean_prefix TEXT DEFAULT 'is_',
    datetime_suffix TEXT DEFAULT '_at',

    -- 其他配置
    engine_type TEXT DEFAULT 'InnoDB',
    charset TEXT DEFAULT 'utf8mb4',
    collation TEXT DEFAULT 'utf8mb4_unicode_ci',

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 插入默认配置（对于新项目）
INSERT INTO table_preferences (
    project_id,
    pk_enabled,
    pk_field_name,
    pk_field_type,
    pk_auto_increment,
    pk_comment,
    audit_enabled,
    audit_fields,
    soft_delete_enabled
) SELECT
    id,
    1,
    'id',
    'BIGINT',
    1,
    '主键',
    1,
    '[{"field":"created_at","type":"TIMESTAMP","default":"CURRENT_TIMESTAMP","nullable":false,"comment":"创建时间"},{"field":"updated_at","type":"TIMESTAMP","default":"CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP","nullable":false,"comment":"更新时间"}]',
    0
FROM projects;
