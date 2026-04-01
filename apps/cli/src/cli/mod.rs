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
    }
}
