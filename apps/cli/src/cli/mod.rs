pub mod commands;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "template-cli")]
#[command(about = "Template Studio CLI - 模板生成器命令行工具", long_about = None)]
#[command(version = "0.1.0")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,

    /// 配置文件路径
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// 服务器 URL (覆盖配置文件)
    #[arg(long, global = true)]
    pub server_url: Option<String>,

    /// API 密钥 (覆盖配置文件)
    #[arg(long, global = true)]
    pub api_key: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 从模板创建新项目
    Create(CreateCommand),

    /// 模板管理命令
    Template {
        #[command(subcommand)]
        template_subcommand: TemplateCommands,
    },

    /// 配置管理
    Config {
        #[command(subcommand)]
        config_subcommand: ConfigCommands,
    },

    /// AI 辅助命令
    Ai {
        #[command(subcommand)]
        ai_subcommand: AiCommands,
    },
}

#[derive(Parser, Debug, Clone)]
pub struct CreateCommand {
    /// 项目名称
    #[arg(value_name = "PROJECT_NAME")]
    pub project_name: Option<String>,

    /// 模板名称或ID
    #[arg(short = 'T', long)]
    pub template: Option<String>,

    /// 输出目录
    #[arg(short = 'o', long, default_value = ".")]
    pub output: String,

    /// 启用TUI模式
    #[arg(short = 't', long)]
    pub tui: bool,

    /// 变量配置文件路径
    #[arg(long)]
    pub config_file: Option<String>,

    /// 强制覆盖已存在的目录
    #[arg(short = 'f', long)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// 列出可用模板
    List {
        /// 按分类过滤
        #[arg(long)]
        category: Option<String>,
    },

    /// 显示模板详细信息
    Info {
        /// 模板名称或ID
        template_name: String,

        /// 显示模板变量
        #[arg(short, long)]
        variables: bool,

        /// 显示文件结构
        #[arg(short, long)]
        files: bool,
    },

    /// 搜索模板
    Search {
        /// 搜索关键词
        keyword: String,

        /// 按分类搜索
        #[arg(long)]
        category: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// 显示当前配置
    Show,

    /// 设置配置项
    Set {
        /// 配置项名称
        key: String,

        /// 配置项值
        value: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiCommands {
    /// 分析模板变量
    AnalyzeVariables {
        /// 模板路径
        path: String,
        /// 输出格式
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// 自动填充变量
    FillVariables {
        /// 模板路径
        path: String,
        /// 项目 ID
        #[arg(long)]
        project: i64,
        /// AI 提供商
        #[arg(long)]
        provider: Option<String>,
        /// AI 模型
        #[arg(long)]
        model: Option<String>,
        /// 只预览不写入
        #[arg(long)]
        dry_run: bool,
        /// 直接写入
        #[arg(long)]
        write: bool,
        /// 输出格式
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// 项目转换为模板
    ConvertToTemplate {
        /// 项目路径
        path: String,
        /// 输出目录
        #[arg(short, long)]
        output: String,
        /// 模板名称
        #[arg(long)]
        name: Option<String>,
        /// 模板分类
        #[arg(long)]
        category: Option<String>,
        /// 转换策略 (conservative/aggressive)
        #[arg(long, default_value = "conservative")]
        strategy: String,
    },

    /// 渲染预览
    RenderPreview {
        /// 模板路径
        path: String,
        /// 变量文件路径
        #[arg(long)]
        vars_file: Option<String>,
        /// 变量 JSON 字符串
        #[arg(long)]
        vars: Option<String>,
        /// 输出完整内容
        #[arg(long)]
        full: bool,
    },

    /// 验证模板
    Validate {
        /// 模板路径
        path: String,
        /// 变量文件路径
        #[arg(long)]
        vars_file: Option<String>,
        /// 检查渲染输出
        #[arg(long)]
        check_output: bool,
    },

    /// 编辑模板文件
    EditFile {
        /// 文件路径
        path: String,
        /// 在指定行后插入
        #[arg(long)]
        insert: Option<usize>,
        /// 替换行范围 (start-end)
        #[arg(long)]
        replace: Option<String>,
        /// 删除行范围 (start-end)
        #[arg(long)]
        delete: Option<String>,
        /// 追加到末尾
        #[arg(long)]
        append: Option<String>,
        /// 内容
        #[arg(long)]
        content: Option<String>,
    },

    /// 推荐模板
    Recommend {
        /// 项目 ID
        #[arg(long)]
        project: Option<i64>,
        /// 编程语言
        #[arg(long)]
        language: Option<String>,
        /// 模板分类
        #[arg(long)]
        category: Option<String>,
        /// 输出推荐理由
        #[arg(long)]
        explain: bool,
    },

    /// AI 配置管理
    Config {
        #[command(subcommand)]
        config_subcommand: AiConfigCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiConfigCommands {
    /// 显示 AI 配置
    Show,

    /// 设置 AI 配置
    Set {
        /// 提供商名称
        #[arg(long)]
        provider: Option<String>,
        /// 模型名称
        #[arg(long)]
        model: Option<String>,
        /// API Key
        #[arg(long)]
        api_key: Option<String>,
        /// API 基础 URL
        #[arg(long)]
        base_url: Option<String>,
    },

    /// 测试 AI 连接
    Test,
}

pub async fn execute(args: CliArgs) -> Result<()> {
    match args.command {
        Commands::Create(create_cmd) => {
            commands::handle_create(create_cmd, args.config, args.server_url, args.api_key).await
        }
        Commands::Template { template_subcommand } => {
            commands::handle_template(template_subcommand, args.config, args.server_url, args.api_key).await
        }
        Commands::Config { config_subcommand } => {
            commands::handle_config(config_subcommand, args.config).await
        }
        Commands::Ai { ai_subcommand } => {
            commands::handle_ai(ai_subcommand, args.config, args.server_url, args.api_key).await
        }
    }
}
