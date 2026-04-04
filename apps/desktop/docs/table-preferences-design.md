# 表规范配置设计方案

## 📋 设计目标

**表规范** = 项目级别的表结构规范，在代码生成时自动应用到所有表。

**应用场景**：
- 新建表时自动应用规范
- 代码生成时使用规范配置
- 保持项目表结构一致性

---

## 🎨 UI 设计

```
┌─────────────────────────────────────────────────────────────────┐
│  📋 表规范配置                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 1️⃣ 主键规范                                             │ │
│  │ ────────────────────────────────────────────────────────  │ │
│  │ ☑ 启用主键规范                                           │ │
│  │                                                           │ │
│  │ 字段名:   [id_______________]                            │ │
│  │ 字段类型: [BIGINT__________] ▼                           │ │
│  │           可选: INT, BIGINT, CHAR(36), VARCHAR(32)       │ │
│  │ ☑ 自增 (AUTO_INCREMENT)                                 │ │
│  │ 字段注释: [主键ID________________________________]       │ │
│  │                                                           │ │
│  │ 💡 提示：UUID 使用 CHAR(36)，雪花ID 使用 BIGINT          │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 2️⃣ 审计字段                                             │ │
│  │ ────────────────────────────────────────────────────────  │ │
│  │ ☑ 启用审计字段                                           │ │
│  │                                                           │ │
│  │ 创建时间字段                                              │ │
│  │ ☑ 启用  字段名: [created_at_________]                   │ │
│  │         类型: [TIMESTAMP_________] ▼                    │ │
│  │         默认值: [CURRENT_TIMESTAMP_______]               │ │
│  │         注释: [创建时间________________]                 │ │
│  │                                                           │ │
│  │ 更新时间字段                                              │ │
│  │ ☑ 启用  字段名: [updated_at_________]                   │ │
│  │         类型: [TIMESTAMP_________] ▼                    │ │
│  │         默认值: [CURRENT_TIMESTAMP           │ │
│  │                  ON UPDATE CURRENT_TIMESTAMP]           │ │
│  │         注释: [更新时间________________]                 │ │
│  │                                                           │ │
│  │ 创建人字段（可选）                                        │ │
│  │ ☐ 启用  字段名: [created_by_________]                   │ │
│  │         类型: [BIGINT___________] ▼                     │ │
│  │         注释: [创建人ID________________]                 │ │
│  │                                                           │ │
│  │ 更新人字段（可选）                                        │ │
│  │ ☐ 启用  字段名: [updated_by_________]                   │ │
│  │         类型: [BIGINT___________] ▼                     │ │
│  │         注释: [更新人ID________________]                 │ │
│  │                                                           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 3️⃣ 软删除字段                                           │ │
│  │ ────────────────────────────────────────────────────────  │ │
│  │ ☐ 启用软删除                                             │ │
│  │                                                           │ │
│  │ 字段名:   [deleted_at________]                           │ │
│  │ 字段类型: [TIMESTAMP________] ▼                          │ │
│  │           可选: TIMESTAMP, DATETIME, BIGINT              │ │
│  │ 允许空值: ☑ (建议勾选，NULL 表示未删除)                 │ │
│  │ 默认值:   [NULL_______________]                          │ │
│  │ 字段注释: [删除时间，NULL表示未删除___________]         │ │
│  │                                                           │ │
│  │ 💡 提示：启用软删除后，删除操作会更新此字段而非物理删除  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 4️⃣ 状态字段                                             │ │
│  │ ────────────────────────────────────────────────────────  │ │
│  │ ☐ 启用状态字段                                           │ │
│  │                                                           │ │
│  │ 字段名:   [status__________]                             │ │
│  │ 字段类型: [TINYINT________] ▼                            │ │
│  │           可选: TINYINT, SMALLINT, VARCHAR               │ │
│  │ 默认值:   [1_____________]                                │ │
│  │ 字段注释: [状态 0:禁用 1:正常_______________]           │ │
│  │                                                           │ │
│  │ 状态枚举值（可选，用于生成代码枚举类）                  │ │
│  │ ┌─────────────────────────────────────────────────────┐ │ │
│  │ │ 值  │ 标签      │ 说明                          │  │ │
│  │ ├────┼──────────┼───────────────────────────────┤  │ │
│  │ │ 0  │ 禁用      │ 账号被禁用，无法登录           │  │ │
│  │ │ 1  │ 正常      │ 正常状态                       │  │ │
│  │ │ 2  │ 锁定      │ 账号被锁定，需要解锁           │  │ │
│  │ │    │ [+ 添加] │                               │  │ │
│  │ └────┴──────────┴───────────────────────────────┘  │ │
│  │                                                           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 5️⃣ 命名规范                                             │ │
│  │ ────────────────────────────────────────────────────────  │ │
│  │ 布尔字段前缀                                              │ │
│  │ [is_]          ☑ 启用                                    │ │
│  │ 示例: is_active, is_deleted                              │ │
│  │                                                           │ │
│  │ 时间字段后缀                                              │ │
│  │ [_at]          ☑ 启用                                    │ │
│  │ 示例: created_at, updated_at, deleted_at                 │ │
│  │                                                           │ │
│  │ 枚举字段前缀（可选）                                      │ │
│  │ [___]          ☐ 启用                                    │ │
│  │ 示例: user_type, order_status                            │ │
│  │                                                           │ │
│  │ 金额字段后缀（可选）                                      │ │
│  │ [___]          ☐ 启用                                    │ │
│  │ 示例: total_amount, pay_price                           │ │
│  │                                                           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 6️⃣ 存储配置                                             │ │
│  │ ────────────────────────────────────────────────────────  │ │
│  │ 存储引擎        [InnoDB____________] ▼                   │ │
│  │                  可选: InnoDB, MyISAM, Memory             │ │
│  │                                                           │ │
│  │ 字符集          [utf8mb4____________] ▼                  │ │
│  │                  可选: utf8, utf8mb4, gbk                │ │
│  │                                                           │ │
│  │ 排序规则        [utf8mb4_unicode_ci___] ▼                │ │
│  │                  可选: utf8mb4_unicode_ci, utf8_general_ci│ │
│  │                                                           │ │
│  │ 行格式          [DYNAMIC____________] ▼                   │ │
│  │                  可选: DYNAMIC, FIXED, COMPRESSED         │ │
│  │                                                           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 7️⃣ 字段默认值                                           │ │
│  │ ────────────────────────────────────────────────────────  │ │
│  │ 字符串字段默认值                                          │ │
│  │ ☐ 设置默认值: [''_______________]                        │ │
│  │                                                           │ │
│  │ 数值字段默认值                                            │ │
│  │ ☐ 设置默认值: [0_______________]                         │ │
│  │                                                           │ │
│  │ 布尔字段默认值                                            │ │
│  │ ☐ 设置默认值: [0 (false)_________] ▼                     │ │
│  │                                                           │ │
│  │ 枚举字段默认值                                            │ │
│  │ ☐ 设置默认值: [1_______________]                         │ │
│  │                                                           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  [重置为默认] [保存配置]                                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📊 配置项详解

### 1️⃣ 主键规范

| 配置项 | 数据库字段 | 默认值 | 说明 |
|--------|-----------|--------|------|
| 启用 | `pk_enabled` | 1 | 是否启用主键规范 |
| 字段名 | `pk_field_name` | 'id' | 主键字段名称 |
| 字段类型 | `pk_field_type` | 'BIGINT' | 主键数据类型 |
| 自增 | `pk_auto_increment` | 1 | 是否自增 |
| 字段注释 | `pk_comment` | '主键ID' | 字段注释 |

**可选类型**：
- `INT` - 适用于小型表
- `BIGINT` - 适用于大型表（推荐）
- `CHAR(36)` - UUID 主键
- `VARCHAR(32)` - 短 ID（如雪花ID）

---

### 2️⃣ 审计字段

| 配置项 | 数据库字段 | 默认值 | 说明 |
|--------|-----------|--------|------|
| 启用 | `audit_enabled` | 1 | 是否启用审计字段 |
| 字段配置 | `audit_fields` | JSON | 审计字段配置（JSON） |

**audit_fields JSON 结构**：
```json
{
  "created_at": {
    "enabled": true,
    "field_name": "created_at",
    "field_type": "TIMESTAMP",
    "default_value": "CURRENT_TIMESTAMP",
    "comment": "创建时间"
  },
  "updated_at": {
    "enabled": true,
    "field_name": "updated_at",
    "field_type": "TIMESTAMP",
    "default_value": "CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP",
    "comment": "更新时间"
  },
  "created_by": {
    "enabled": false,
    "field_name": "created_by",
    "field_type": "BIGINT",
    "comment": "创建人ID"
  },
  "updated_by": {
    "enabled": false,
    "field_name": "updated_by",
    "field_type": "BIGINT",
    "comment": "更新人ID"
  }
}
```

---

### 3️⃣ 软删除字段

| 配置项 | 数据库字段 | 默认值 | 说明 |
|--------|-----------|--------|------|
| 启用 | `soft_delete_enabled` | 0 | 是否启用软删除 |
| 字段名 | `soft_delete_field` | 'deleted_at' | 软删除字段名 |
| 字段类型 | `soft_delete_field_type` | 'TIMESTAMP' | 字段数据类型 |
| 允许空值 | `soft_delete_nullable` | 1 | 是否允许NULL |
| 默认值 | `soft_delete_default` | NULL | 默认值 |
| 字段注释 | `soft_delete_comment` | '删除时间' | 字段注释 |

**使用场景**：
- 启用后，删除操作会更新此字段而非物理删除
- 查询时需要过滤 `WHERE deleted_at IS NULL`

---

### 4️⃣ 状态字段（新增）

**目的**：定义项目的状态字段规范

**配置存储**：可扩展到数据库，或存储在 JSON 中

```json
{
  "status": {
    "enabled": true,
    "field_name": "status",
    "field_type": "TINYINT",
    "default_value": "1",
    "comment": "状态 0:禁用 1:正常",
    "enum_values": [
      {"value": 0, "label": "禁用", "description": "账号被禁用，无法登录"},
      {"value": 1, "label": "正常", "description": "正常状态"},
      {"value": 2, "label": "锁定", "description": "账号被锁定，需要解锁"}
    ]
  }
}
```

**代码生成示例**：
```java
// Java 枚举类
public enum UserStatus {
    DISABLED(0, "禁用"),
    NORMAL(1, "正常"),
    LOCKED(2, "锁定");

    private final Integer value;
    private final String label;

    UserStatus(Integer value, String label) {
        this.value = value;
        this.label = label;
    }
}
```

---

### 5️⃣ 命名规范

| 配置项 | 数据库字段 | 默认值 | 说明 |
|--------|-----------|--------|------|
| 布尔字段前缀 | `boolean_prefix` | 'is_' | 布尔字段前缀 |
| 时间字段后缀 | `datetime_suffix` | '_at' | 时间字段后缀 |

**示例**：
- 布尔字段：`is_active`, `is_deleted`, `is_verified`
- 时间字段：`created_at`, `updated_at`, `deleted_at`

**可选扩展**（存储在 JSON）：
- 枚举字段前缀：`user_type`, `order_status`
- 金额字段后缀：`total_amount`, `pay_price`

---

### 6️⃣ 存储配置

| 配置项 | 数据库字段 | 默认值 | 说明 |
|--------|-----------|--------|------|
| 存储引擎 | `engine_type` | 'InnoDB' | 存储引擎 |
| 字符集 | `charset` | 'utf8mb4' | 字符集 |
| 排序规则 | `collation` | 'utf8mb4_unicode_ci' | 排序规则 |

**可选存储引擎**：
- `InnoDB` - 默认，支持事务
- `MyISAM` - 不支持事务，但读取快
- `Memory` - 内存表

---

### 7️⃣ 字段默认值（新增）

**目的**：为不同类型的字段设置默认值

**配置存储**：JSON

```json
{
  "defaults": {
    "string": {
      "enabled": false,
      "value": ""
    },
    "number": {
      "enabled": true,
      "value": "0"
    },
    "boolean": {
      "enabled": true,
      "value": "0"
    },
    "enum": {
      "enabled": true,
      "value": "1"
    }
  }
}
```

---

## 🚀 应用场景

### 场景 1：新建表时自动应用规范

```
用户创建新表 "users"
  ↓
自动应用表规范：
  - 添加主键字段 id (BIGINT, AUTO_INCREMENT)
  - 添加审计字段 created_at, updated_at
  - 添加软删除字段 deleted_at (如果启用)
  - 添加状态字段 status (如果启用)
  ↓
用户只需要添加业务字段（name, email 等）
```

### 场景 2：代码生成时使用规范

```java
// 生成的 Entity 类
@Table(name = "users")
public class UserEntity {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;  // 来自主键规范

    @Column(name = "name")
    private String name;  // 业务字段

    @Column(name = "created_at")
    private LocalDateTime createdAt;  // 来自审计字段规范

    @Column(name = "updated_at")
    private LocalDateTime updatedAt;  // 来自审计字段规范

    @Column(name = "status")
    private Integer status;  // 来自状态字段规范
}
```

---

## 📐 数据库表结构

### 现有字段（已存在）

```sql
CREATE TABLE IF NOT EXISTS table_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,

    -- 主键规范
    pk_enabled INTEGER DEFAULT 1,
    pk_field_name TEXT DEFAULT 'id',
    pk_field_type TEXT DEFAULT 'BIGINT',
    pk_auto_increment INTEGER DEFAULT 1,
    pk_comment TEXT,

    -- 审计字段配置
    audit_enabled INTEGER DEFAULT 1,
    audit_fields TEXT,  -- JSON

    -- 软删除字段配置
    soft_delete_enabled INTEGER DEFAULT 0,
    soft_delete_field TEXT DEFAULT 'deleted_at',
    soft_delete_field_type TEXT DEFAULT 'TIMESTAMP',
    soft_delete_nullable INTEGER DEFAULT 1,
    soft_delete_default TEXT,
    soft_delete_comment TEXT,

    -- 命名规范
    boolean_prefix TEXT DEFAULT 'is_',
    datetime_suffix TEXT DEFAULT '_at',

    -- 其他配置
    engine_type TEXT DEFAULT 'InnoDB',
    charset TEXT DEFAULT 'utf8mb4',
    collation TEXT DEFAULT 'utf8mb4_unicode_ci',

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (project_id) REFERENCES projects(id)
);
```

### 新增字段（需要迁移）

```sql
-- 状态字段配置（JSON）
ALTER TABLE table_preferences ADD COLUMN status_config TEXT;

-- 字段默认值配置（JSON）
ALTER TABLE table_preferences ADD COLUMN field_defaults TEXT;

-- 扩展命名规范（JSON，支持更多前缀后缀）
ALTER TABLE table_preferences ADD COLUMN naming_conventions TEXT;
```

---

## ✅ 实现优先级

### Phase 1: 核心功能（P0）
1. ✅ 主键规范配置
2. ✅ 审计字段配置（created_at, updated_at）
3. ✅ 软删除字段配置
4. ✅ 命名规范配置（boolean_prefix, datetime_suffix）
5. ✅ 保存/加载配置

### Phase 2: 扩展功能（P1）
6. ✅ 状态字段配置
7. ✅ 审计字段扩展（created_by, updated_by）
8. ✅ 存储配置（引擎、字符集）
9. ✅ 字段默认值配置

### Phase 3: 高级功能（P2）
10. ✅ 预览 SQL（根据规范生成建表语句）
11. ✅ 应用规范到现有表
12. ✅ 配置模板（保存/加载）

---

## 🎯 总结

**表规范配置包含 7 大类**：

1. **主键规范** - 定义主键字段标准
2. **审计字段** - created_at, updated_at, created_by, updated_by
3. **软删除字段** - deleted_at 配置
4. **状态字段** - status 字段及枚举值
5. **命名规范** - 字段前缀后缀
6. **存储配置** - 引擎、字符集
7. **字段默认值** - 不同类型字段的默认值

**价值**：
- ✅ 保持项目表结构一致性
- ✅ 减少重复配置
- ✅ 代码生成自动应用规范
- ✅ 新手友好，不会遗漏字段

这个设计合理吗？有哪些地方需要调整？
