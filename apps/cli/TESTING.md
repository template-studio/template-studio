# CLI 应用测试指南

## ✅ 已验证的功能

### 1. 命令行帮助

```bash
# 主帮助
cargo run -- --help

# Create 命令帮助
cargo run -- create --help

# Template 命令帮助
cargo run -- template --help

# Template list 命令帮助
cargo run -- template list --help
```

**预期结果**: 显示完整的命令行帮助信息

### 2. CLI 模式测试

```bash
# 测试模板列表(需要后端 API 运行)
cargo run -- template list

# 测试模板搜索
cargo run -- template search web

# 测试配置显示
cargo run -- config show
```

**预期结果**:
- `template list`: 列出所有可用模板
- `template search`: 显示匹配的模板
- `config show`: 显示当前配置

### 3. TUI 模式测试

```bash
# 进入 TUI 模式(不指定参数)
cargo run -- create

# 或显式启用 TUI
cargo run -- create --tui
```

**操作步骤**:
1. 按 Enter 进入下一步
2. 输入项目名称
3. 输入输出目录(或留空)
4. 输入搜索关键词
5. 使用 ↑/↓ 选择模板
6. 按 Enter 确认
7. 按 Esc 或 Ctrl+C 退出

**预期结果**: 显示交互式 TUI 界面

### 4. 参数验证测试

```bash
# 测试参数冲突已修复
cargo run -- create --help | grep config-file

# 应该看到两个不同的配置选项:
# -c, --config <CONFIG>            配置文件路径(全局)
#     --config-file <CONFIG_FILE>  变量配置文件路径(create 命令)
```

## 🐛 已知问题

### 1. 后端 API 依赖

某些命令需要后端 API 运行:
- `template list`
- `template search`
- `template info`
- `create` (完整流程)

**解决方案**:
```bash
# 启动后端服务
cd apps/web
cargo run

# 然后在另一个终端测试 CLI
cd apps/cli
cargo run -- template list
```

### 2. 配置文件首次运行

首次运行时会自动创建配置目录和配置文件。

**配置目录结构**:
```
~/.ciclebyte/template_studio_rust/
├── config/
│   └── config.toml          # CLI配置文件
└── data/
    └── templates/           # 模板缓存目录
```

**配置文件位置**:
- Windows: `%USERPROFILE%\.ciclebyte\template_studio_rust\config\config.toml`
- Linux/Mac: `~/.ciclebyte/template_studio_rust/config/config.toml`

**默认配置内容**:
```toml
[server]
url = "http://127.0.0.1:8080"
api_key = ""

[user]
author = null
email = null

[storage]
template_path = "C:\\Users\\{user}\\.ciclebyte\\template_studio_rust\\data\\templates"
```

用户可以直接编辑配置文件修改URL、模板存储路径等配置。

### 3. TUI 模式在某些终端可能显示异常

如果 TUI 界面显示异常,尝试:
- 使用 Windows Terminal
- 使用 PowerShell
- 确保终端支持 ANSI 颜色

## 📊 测试清单

### 基础功能
- [x] 主帮助命令正常
- [x] Create 帮助命令正常
- [x] Template 帮助命令正常
- [x] 参数冲突已修复
- [x] 编译无错误(仅有警告)

### CLI 模式
- [ ] template list (需要后端)
- [ ] template search (需要后端)
- [ ] template info (需要后端)
- [ ] config show
- [ ] create with all parameters (需要后端)

### TUI 模式
- [x] TUI 界面启动
- [x] 欢迎页面显示
- [x] 项目名称输入
- [x] 输出目录输入
- [x] 模板搜索输入
- [x] 模板选择列表
- [x] 键盘导航(↑/↓/Enter/Esc)
- [x] 错误提示显示

### 错误处理
- [x] 参数冲突检测
- [x] 无效参数提示
- [x] 缺少参数提示
- [ ] API 错误处理(需要后端)
- [ ] 文件系统错误处理

## 🚀 快速测试流程

### 1. 最小测试(不需要后端)

```bash
# 测试帮助命令
cargo run -- --help
cargo run -- create --help
cargo run -- template --help

# 测试配置命令
cargo run -- config show

# 测试 TUI 启动(启动后立即退出)
cargo run -- create
# 按 Ctrl+C 或 Esc 退出
```

### 2. 完整测试(需要后端)

```bash
# 1. 启动后端(终端1)
cd apps/web
cargo run

# 2. 测试 CLI(终端2)
cd apps/cli

# 列出模板
cargo run -- template list

# 搜索模板
cargo run -- template search web

# 创建项目(CLI 模式)
cargo run -- create my-app --template <template-id> --output ./test-output

# 创建项目(TUI 模式)
cargo run -- create
```

## 💡 调试技巧

### 启用详细日志

```bash
# 设置环境变量启用调试日志
export RUST_LOG=debug
cargo run -- create
```

### 检查配置文件

```bash
# Windows
type %USERPROFILE%\.config\template-studio\config.toml

# Linux/Mac
cat ~/.config/template-studio/config.toml
```

### 清理重建

```bash
# 清理构建缓存
cargo clean

# 重新构建
cargo build --release
```

## 📝 测试报告模板

```
测试日期: ___________
测试环境: ___________
测试人员: ___________

基础功能:
- [ ] 帮助命令正常
- [ ] 参数解析正确
- [ ] 配置加载正常

CLI 模式:
- [ ] 模板列表
- [ ] 模板搜索
- [ ] 项目创建

TUI 模式:
- [ ] 界面显示正常
- [ ] 键盘操作响应
- [ ] 状态切换正确

问题记录:
1. ___________
2. ___________

建议:
1. ___________
2. ___________
```

## 🎯 下一步

1. ✅ 修复编译警告
2. ⏳ 完善错误处理
3. ⏳ 添加单元测试
4. ⏳ 实现变量配置界面
5. ⏳ 添加集成测试
