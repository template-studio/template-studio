mod cli;
mod client;
mod config;
mod generator;
mod storage;
mod tui;
mod variables;
mod renderer;
mod ai;

use anyhow::Result;
use clap::Parser;
use cli::CliArgs;
use tracing::{info, error};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = CliArgs::parse();

    // 检查是否进入TUI模式
    let is_tui_mode = matches!(args.command, cli::Commands::Create(ref cmd) if cmd.tui || cmd.project_name.is_none() || cmd.template.is_none());

    // 初始化日志
    if is_tui_mode {
        // TUI模式：日志写入文件，不输出到终端
        init_file_logging()?;
    } else {
        // CLI模式：日志输出到终端
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into())
            )
            .init();
    }

    info!("🚀 Template Studio CLI 启动中...");

    // 执行命令
    if let Err(err) = cli::execute(args).await {
        error!("❌ 错误: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

/// 初始化文件日志（用于TUI模式）
fn init_file_logging() -> Result<()> {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
    use tracing_appender::rolling;

    // 获取日志目录
    let log_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cicbyte")
        .join("template_studio")
        .join("logs");

    // 创建日志目录
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| anyhow::anyhow!("创建日志目录失败: {}", e))?;

    // 设置日志文件（按天滚动）
    let file_appender = rolling::daily(&log_dir, "template-cli.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 初始化日志订阅器
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
        )
        .init();

    // 在标准错误输出中显示日志文件位置（仅一次）
    eprintln!("📄 日志文件: {}", log_dir.join("template-cli.log").display());

    Ok(())
}
