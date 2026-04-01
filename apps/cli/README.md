# Template Studio CLI

基于 Rust 的模板生成器命令行工具,参考 Go 版本实现,支持 CLI 和 TUI 两种交互模式。

## 功能特性

- ✅ **CLI 模式**: 命令行参数直接执行,适合脚本和自动化
- ✅ **TUI 模式**: 现代化的终端用户界面,交互式操作
- 🔍 **模板搜索**: 远程模板库搜索和查询
- 🎯 **模板选择**: 交互式选择模板
- ⚙️ **变量配置**: 支持模板变量配置
- 📦 **项目生成**: 一键生成完整项目
- 🐙 **Git 集成**: 自动初始化 Git 仓库

## 构建和安装

```bash
# 从 workspace 根目录构建
cargo build --release --bin template-cli

# 编译后的可执行文件位置
# Windows: target/release/template-cli.exe
# Linux/Mac: target/release/template-cli
```

## 使用方法

### CLI 模式 (命令行模式)

```bash
# 直接创建项目(指定所有参数)
template-cli create my-project --template go-web --output ./projects

# 使用模板 ID 创建
template-cli create my-app --template 12345

# 创建并初始化 Git 仓库
template-cli create my-project --template go-web --git-init

# 强制覆盖已存在的目录
template-cli create my-project --template go-web --force
```

### TUI 模式 (交互式界面)

```bash
# 启动 TUI 模式(未指定项目名或模板时自动进入)
template-cli create

# 显式启用 TUI 模式
template-cli create --tui

# 指定项目名,其他参数交互输入
template-cli create my-project --tui
```

### 模板管理命令

```bash
# 列出所有可用模板
template-cli template list

# 按分类过滤模板
template-cli template list --category web

# 搜索模板
template-cli template search web

# 查看模板详细信息
template-cli template info go-web

# 查看模板变量
template-cli template info go-web --variables

# 查看模板文件结构
template-cli template info go-web --files
```

### 配置管理

```bash
# 显示当前配置
template-cli config show

# 设置配置项
template-cli config set server.url "http://localhost:8080"
template-cli config set server.api_key "your-api-key"
```

## 配置文件

配置文件位置:
- **Windows**: `%USERPROFILE%\.ciclebyte\template_studio_rust\config\config.toml`
- **Linux/Mac**: `~/.ciclebyte/template_studio_rust/config/config.toml`

首次运行时CLI会自动创建配置目录和默认配置文件。用户可以直接编辑此文件修改配置。

配置文件格式:

```toml
# Template Studio CLI 配置文件

[server]
url = "http://127.0.0.1:8080"  # 服务器地址
api_key = ""                    # API密钥(可选)

[user]
author = "Your Name"            # 默认作者
email = "your.email@example.com" # 默认邮箱

[storage]
template_path = "C:\\Users\\{user}\\.ciclebyte\\template_studio_rust\\data\\templates"  # 模板存储路径
```

**目录结构**:
```
~/.ciclebyte/template_studio_rust/
├── config/
│   └── config.toml          # CLI配置文件
└── data/
    └── templates/           # 模板缓存目录
```

## TUI 模式操作指南

### 快捷键

- **Enter/Space**: 确认/继续
- **Esc**: 返回/退出
- **↑/↓ 或 j/k**: 上下导航
- **/**: 搜索
- **q**: 退出

### 操作流程

1. **欢迎页面**: 按 Enter 开始
2. **项目名称**: 输入项目名称,按 Enter 继续
3. **输出目录**: 输入输出目录(留空使用当前目录),按 Enter 继续
4. **搜索模板**: 输入关键词搜索模板,按 Enter 查看结果
5. **选择模板**: 使用 ↑/↓ 选择模板,按 Enter 确认
6. **配置变量**: (TODO) 配置模板变量
7. **完成**: 确认信息,按 Enter 开始创建

## 开发说明

### 项目结构

```
apps/cli/
├── Cargo.toml              # 项目配置
├── README.md               # 本文件
└── src/
    ├── main.rs             # 程序入口
    ├── cli/                # CLI 命令层
    │   ├── mod.rs          # 命令定义
    │   └── commands.rs     # 命令处理
    ├── client/             # HTTP 客户端
    │   └── mod.rs          # API 客户端实现
    ├── config/             # 配置管理
    │   └── mod.rs          # 配置加载和保存
    ├── generator/          # 项目生成器
    │   └── mod.rs          # 文件生成逻辑
    └── tui/                # TUI 界面
        ├── mod.rs          # TUI 入口
        ├── app.rs          # 应用状态管理
        └── ui.rs           # UI 渲染
```

### 技术栈

- **CLI 框架**: clap 4.4
- **TUI 框架**: ratatui 0.23
- **HTTP 客户端**: reqwest 0.11
- **异步运行时**: tokio
- **配置管理**: config + toml
- **终端操作**: crossterm

### 与 Go 版本的差异

1. **性能**: Rust 版本性能更好,内存占用更低
2. **分发**: 单一二进制文件,无需依赖
3. **类型安全**: 编译时类型检查
4. **TUI 框架**: 使用 ratatui 而非 Bubble Tea
5. **错误处理**: 使用 anyhow 进行统一错误处理

## TODO

- [ ] 完善变量配置界面
- [ ] 支持从配置文件加载变量
- [ ] 实现模板预览功能
- [ ] 添加进度条显示
- [ ] 支持离线模式
- [ ] 添加单元测试

## License

MIT
