# AI 建表功能使用指南

## 功能概述

AI 建表功能允许用户通过自然语言描述，让 AI 自动生成数据库表的 SQL 语句，并直接在项目中创建表结构。

## 前置条件

### 1. 配置 AI 服务

1. 打开应用，进入 **设置** → **AI 服务**
2. 配置至少一个 AI 提供商（DeepSeek / GLM / LongCat）
3. 填写 API 密钥
4. 切换启用状态

### 2. 设置默认服务

1. 在 AI 服务二级菜单中，点击顶部的 **默认服务**
2. 选择默认提供商
3. 选择默认模型

### 3. 创建项目

1. 进入 **项目** 页面
2. 点击 **新建项目**
3. 选择数据源、输入项目名称和数据库名称
4. 保存项目

## 使用流程

### 步骤 1：输入描述

在项目的表管理页面，点击 **AI 建表** 按钮：

1. 选择 **SQL 类型**（MySQL / PostgreSQL / SQLite）
2. 在 **表描述** 文本框中输入表结构描述

**示例输入：**
```
用户表：
- id：主键，自增
- username：用户名，唯一，不为空，最大长度 50
- email：邮箱，唯一，不为空
- password：密码哈希，不为空
- created_at：创建时间，默认当前时间
- updated_at：更新时间，默认当前时间
```

点击 **生成 SQL** 按钮

### 步骤 2：预览 SQL

AI 会生成 SQL 建表语句，例如：

```sql
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

**操作：**
- 如果满意，点击 **下一步**
- 如果需要修改，可以手动编辑 SQL
- 点击 **重新生成** 可以让 AI 重新生成

### 步骤 3：预览字段

系统会解析 SQL，显示字段预览表格：

| 字段名 | 类型 | 允许空值 | 主键 | 默认值 |
|--------|------|----------|------|--------|
| id | INT | × | ✓ | - |
| username | VARCHAR(50) | × | × | - |
| email | VARCHAR(255) | × | × | - |

**操作：**
- 检查字段解析是否正确
- 如果正确，点击 **完成** 执行创建
- 如果有问题，点击 **上一步** 修改 SQL

### 步骤 4：执行结果

- **成功**：显示绿色成功提示，表已创建
- **失败**：显示错误信息，可以点击 **AI 修复** 按钮

**AI 修复：**
点击 **AI 修复** 后，AI 会：
1. 分析错误原因
2. 自动修复 SQL 语句
3. 返回修复后的 SQL
4. 自动跳转到步骤 3 重新预览

## 支持的数据库类型

### MySQL
- 支持 MySQL 5.7+ 语法
- 支持 AUTO_INCREMENT、ON UPDATE CURRENT_TIMESTAMP 等特性

### PostgreSQL
- 支持 PostgreSQL 12+ 语法
- 支持 SERIAL、BIGSERIAL 等自增类型
- 支持 ARRAY、JSON 等高级类型

### SQLite
- 支持 SQLite 3.x 语法
- 支持 INTEGER PRIMARY KEY AUTOINCREMENT
- 支持 WITHOUT ROWID 等特性

## AI 提示词最佳实践

### ✅ 好的描述

**详细具体：**
```
商品表：
- id：主键，自增
- name：商品名称，不为空，最大长度 100
- price：价格，DECIMAL(10,2)，不为空，默认 0.00
- stock：库存，INT，不为空，默认 0
- category_id：分类ID，外键关联 categories 表
- description：商品描述，TEXT
- created_at：创建时间
- updated_at：更新时间

索引：
- idx_name (name)
- idx_category (category_id)
- idx_price (price)
```

**明确类型和约束：**
```
订单表：
- order_id：BIGINT，主键，自增
- user_id：BIGINT，不为空，外键
- order_no：VARCHAR(32)，唯一，不为空
- total_amount：DECIMAL(12,2)，不为空
- status：TINYINT，默认 0（0:待付款,1:已付款,2:已发货）
```

### ❌ 不好的描述

**太模糊：**
```
创建一个用户表
```

**缺少约束信息：**
```
表有 id, name, email 字段
```

**类型不明确：**
```
价格字段，数字类型
```

## 常见问题

### Q1: AI 生成的 SQL 执行失败怎么办？

A: 点击 **AI 修复** 按钮，AI 会根据错误信息自动修复 SQL。

### Q2: 可以一次创建多个表吗？

A: 可以！在表描述中依次描述多个表，AI 会生成多个 CREATE TABLE 语句。

**示例：**
```
分类表：
- id：主键，自增
- name：分类名称，不为空
- parent_id：父级ID，默认 0

商品表：
- id：主键，自增
- name：商品名称
- category_id：分类ID，外键关联 categories
```

### Q3: 如何修改已生成的 SQL？

A: 在步骤 2 的 SQL 预览界面，可以直接编辑 SQL 文本框中的内容。

### Q4: 支持添加索引吗？

A: 支持！在描述中明确指出需要创建索引，AI 会生成对应的索引语句。

**示例：**
```
用户表：
- id：主键
- username：用户名，不为空
- email：邮箱，不为空

索引：
- idx_username (username)
- idx_email (email)
```

### Q5: 字段解析错误怎么办？

A:
1. 返回步骤 2，手动修改 SQL
2. 或者修改表描述，重新生成
3. 确保使用标准 SQL 语法

## 技术实现

### 前端命令调用

```javascript
// 生成 SQL
await invoke('ai_generate_sql', {
  provider: 'deepseek',
  model: 'deepseek-chat',
  prompt: '请生成 MySQL 建表 SQL...'
})

// 解析 SQL
await invoke('parse_ai_sql', {
  projectId: 1,
  sql: 'CREATE TABLE users ...',
  dialect: 'mysql'
})

// 执行 SQL
await invoke('execute_ai_sql', {
  projectId: 1,
  sql: 'CREATE TABLE users ...',
  dialect: 'mysql'
})

// 修复 SQL
await invoke('ai_fix_sql', {
  provider: 'deepseek',
  model: 'deepseek-chat',
  sql: 'CREATE TABLE users ...',
  error: '错误信息',
  dialect: 'mysql'
})
```

### 后端实现

- **AI 调用**：使用 `reqwest` 调用 OpenAI 兼容 API
- **SQL 解析**：使用 `sqlparser-rs` 解析多种 SQL 方言
- **数据存储**：使用 `sqlx` 操作 SQLite 数据库

## 未来改进计划

- [ ] 支持外键关系的自动检测
- [ ] 支持表关系的可视化展示
- [ ] 支持从现有表生成描述
- [ ] 支持 SQL 优化建议
- [ ] 支持批量导入和编辑

---

**最后更新：** 2025-02-09
