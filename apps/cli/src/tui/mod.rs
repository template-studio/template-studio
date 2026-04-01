mod app;
mod ui;

use crate::client::ApiClient;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use tracing::info;

pub use app::App;

pub async fn run_tui(
    client: ApiClient,
    project_name: Option<String>,
    template_id: Option<String>,
    output_dir: &str,
    force: bool,
) -> Result<()> {
    info!("启动 TUI 模式...");

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 隐藏光标
    terminal.hide_cursor()?;

    // 创建应用
    let mut app = App::new(client, project_name, template_id, output_dir, force);

    // 运行应用
    let result = app.run(&mut terminal).await;

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
