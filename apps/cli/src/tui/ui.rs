use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppState};

pub fn render_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    // 创建主布局
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // 标题
            Constraint::Length(3),  // 进度
            Constraint::Min(0),     // 主要内容
            Constraint::Length(3),  // 底部帮助
        ])
        .split(f.size());

    // 渲染标题
    render_title::<B>(f, chunks[0]);

    // 渲染进度
    render_progress::<B>(f, chunks[1], app);

    // 渲染主要内容
    match app.state {
        AppState::Welcome => render_welcome::<B>(f, chunks[2]),
        AppState::ProjectName => render_project_name::<B>(f, chunks[2], app),
        AppState::OutputDir => render_output_dir::<B>(f, chunks[2], app),
        AppState::TemplateSearch => render_template_search::<B>(f, chunks[2], app),
        AppState::TemplateSelect => render_template_select::<B>(f, chunks[2], app),
        AppState::VersionSelect => render_version_select::<B>(f, chunks[2], app),
        AppState::Variables => render_variables::<B>(f, chunks[2], app),
        AppState::Complete => render_complete::<B>(f, chunks[2], app),
    }

    // 渲染错误信息
    if let Some(ref error) = app.error_message {
        render_error::<B>(f, Rect {
            x: chunks[2].x,
            y: chunks[2].bottom().min(f.size().bottom() - 4),
            width: chunks[2].width,
            height: 3,
        }, error);
    }

    // 渲染底部帮助
    render_footer::<B>(f, chunks[3], app);
}

fn render_title<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let title = Paragraph::new("🚀 Template Studio CLI - TUI模式")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    f.render_widget(title, area);
}

fn render_progress<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let steps = [
        "欢迎",
        "项目名称",
        "输出目录",
        "搜索模板",
        "选择模板",
        "选择版本",
        "配置变量",
        "完成",
    ];

    let current_index = match app.state {
        AppState::Welcome => 0,
        AppState::ProjectName => 1,
        AppState::OutputDir => 2,
        AppState::TemplateSearch => 3,
        AppState::TemplateSelect => 4,
        AppState::VersionSelect => 5,
        AppState::Variables => 6,
        AppState::Complete => 7,
    };

    let progress_text: Vec<Span> = steps
        .iter()
        .enumerate()
        .flat_map(|(i, step)| {
            let mut spans = vec![];

            if i < current_index {
                // 已完成
                spans.push(Span::styled(
                    "✓ ",
                    Style::default().fg(Color::Green),
                ));
                spans.push(Span::styled(
                    *step,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::DIM),
                ));
            } else if i == current_index {
                // 当前
                spans.push(Span::styled(
                    "◉ ",
                    Style::default().fg(Color::Cyan),
                ));
                spans.push(Span::styled(
                    *step,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                // 未完成
                spans.push(Span::styled(
                    "○ ",
                    Style::default().fg(Color::DarkGray),
                ));
                spans.push(Span::styled(
                    *step,
                    Style::default().fg(Color::DarkGray),
                ));
            }

            if i < steps.len() - 1 {
                spans.push(Span::styled(" → ", Style::default().fg(Color::DarkGray)));
            }

            spans
        })
        .collect();

    let paragraph = Paragraph::new(Line::from(progress_text))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_welcome<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let text = vec![
        Line::from("欢迎使用 Template Studio CLI!"),
        Line::from(""),
        Line::from("这个工具将帮助您:"),
        Line::from("  1. 设置项目基本信息"),
        Line::from("  2. 选择合适的模板"),
        Line::from("  3. 配置项目变量"),
        Line::from("  4. 生成项目代码"),
        Line::from(""),
        Line::from("按 Enter 继续..."),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_project_name<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let text = vec![
        Line::from("第1步: 设置项目名称"),
        Line::from(""),
        Line::from(format!("> {}", app.project_input)),
        Line::from(""),
        Line::from("(输入项目名称后按 Enter 继续)"),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("项目名称"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_output_dir<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    use std::path::Path;

    let mut text = vec![
        Line::from("第2步: 设置输出目录"),
        Line::from(""),
        Line::from(format!("> {}", app.output_input)),
        Line::from(""),
    ];

    // 检查项目路径是否已存在
    if !app.project_input.is_empty() {
        let project_path = Path::new(&app.output_input).join(&app.project_input);
        if project_path.exists() {
            text.push(Line::from(Span::styled(
                "⚠️  警告: 项目路径已存在，创建将覆盖原有内容",
                Style::default().fg(Color::Red),
            )));
            text.push(Line::from(""));
        }
    }

    text.push(Line::from("(输入输出目录后按 Enter 继续，留空使用当前目录)"));

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("输出目录"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_template_search<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let text = vec![
        Line::from("第3步: 搜索模板"),
        Line::from(""),
        Line::from("输入模板关键词（如: web, api, vue）:"),
        Line::from(""),
        Line::from(format!("> {}", app.search_input)),
        Line::from(""),
        Line::from(format!("已加载 {} 个模板", app.templates.len())),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("搜索模板"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_template_select<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .filtered_templates
        .iter()
        .enumerate()
        .map(|(i, tmpl)| {
            let prefix = if Some(i) == app.selected_template {
                "→ "
            } else {
                "  "
            };

            let content = format!(
                "{}{} (ID: {})\n  {}",
                prefix,
                tmpl.name,
                tmpl.id,
                tmpl.description.as_deref().unwrap_or("无描述")
            );

            let style = if Some(i) == app.selected_template {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("选择模板"));

    f.render_widget(list, area);
}

fn render_version_select<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let mut lines = vec![
        Line::from("第5步: 选择模板版本"),
        Line::from(""),
    ];

    // 获取当前选择的模板
    if let Some(idx) = app.selected_template {
        if let Some(tmpl) = app.filtered_templates.get(idx) {
            lines.push(Line::from(format!("模板: {}", tmpl.name)));
            lines.push(Line::from(""));

            if app.versions.is_empty() {
                lines.push(Line::from("正在加载版本..."));
            } else {
                lines.push(Line::from(format!("可用版本 (共{}个):", app.versions.len())));
                lines.push(Line::from(""));

                // 显示版本列表
                for (i, version) in app.versions.iter().enumerate() {
                    let is_selected = app.selected_version == Some(i);
                    let is_latest = version.is_latest;
                    let is_deprecated = version.is_deprecated;

                    let prefix = if is_selected {
                        "→ "
                    } else {
                        "  "
                    };

                    let mut marker = Vec::new();
                    if is_latest {
                        marker.push("最新");
                    }
                    if is_deprecated {
                        marker.push("已弃用");
                    }

                    let marker_str = if marker.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", marker.join(", "))
                    };

                    let created_info = version.created_at
                        .format("%Y-%m-%d %H:%M")
                        .to_string();

                    if is_selected {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled(format!("{}{}", version.version, marker_str), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        ]));
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(format!("提交: {}", version.commit_message.as_deref().unwrap_or("无")), Style::default().fg(Color::DarkGray)),
                        ]));
                        if let Some(changelog) = &version.changelog {
                            lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled(changelog, Style::default().fg(Color::DarkGray)),
                            ]));
                        }
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(format!("创建时间: {}", created_info), Style::default().fg(Color::DarkGray)),
                        ]));
                    } else {
                        lines.push(Line::from(format!("{}{}{}", prefix, version.version, marker_str)));
                        lines.push(Line::from(format!("  {}", version.commit_message.as_deref().unwrap_or("无"))));
                    }
                    lines.push(Line::from(""));
                }
            }
        }
    }

    lines.push(Line::from("(使用 ↑/↓ 选择版本，按 Enter 下载并继续)"));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("选择版本"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_variables<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let mut lines = vec![
        Line::from("第6步: 配置项目变量"),
        Line::from(""),
    ];

    if let Some(ref defs) = app.variable_definitions {
        if app.variable_order.is_empty() {
            lines.push(Line::from("该模板没有配置变量"));
            lines.push(Line::from(""));
            lines.push(Line::from("(按 Enter 继续)"));
        } else {
            // 使用预存的稳定顺序
            let mut current_group = String::new();

            for (idx, name) in app.variable_order.iter().enumerate() {
                if let Some(def) = defs.get(name) {
                    // 显示分组标题
                    if current_group != def.ui.group {
                        current_group = def.ui.group.clone();
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            format!("[{}]", current_group),
                            Style::default().fg(Color::Cyan),
                        )));
                    }

                    // 构建变量行
                    let is_selected = idx == app.variable_input_index;
                    let required_mark = if def.required { " *" } else { "" };
                    let prefix = if is_selected { "> " } else { "  " };

                    let value = app.variable_values.get(name).map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => "".to_string(),
                    }).unwrap_or_else(|| {
                        // 使用默认值
                        match def.variable_type.as_str() {
                            "boolean" | "conditional" => {
                                if def.default.as_bool().unwrap_or(false) { "true".to_string() } else { "false".to_string() }
                            }
                            _ => {
                                def.default.as_str().unwrap_or("").to_string()
                            }
                        }
                    });

                    let display_value = if is_selected && !app.variable_input_buffer.is_empty() {
                        app.variable_input_buffer.clone()
                    } else {
                        value
                    };

                    let style = if is_selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(format!("{}{}: ", def.title, required_mark), style),
                        Span::styled(display_value, Style::default().fg(Color::Green)),
                    ]));

                    if is_selected && !def.description.is_empty() {
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(def.description.clone(), Style::default().fg(Color::DarkGray)),
                        ]));
                    }
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from("(↑/↓: 切换 • Enter: 确认 • Tab: 跳过 • Esc: 返回)"));
        }
    } else {
        lines.push(Line::from("正在加载变量定义..."));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("配置变量"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_complete<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let text = if app.show_confirm_dialog {
        vec![
            Line::from("⚠️  确认覆盖"),
            Line::from(""),
            Line::from(app.confirm_message.as_str()),
        ]
    } else {
        vec![
            Line::from("✅ 项目配置完成!"),
            Line::from(""),
            Line::from(format!("项目名称: {}", app.project_input)),
            Line::from(format!("输出目录: {}", app.output_input)),
            Line::from(format!("模板: {}", app.template_id.as_deref().unwrap_or("未选择"))),
            Line::from(""),
            Line::from("按 Enter 开始创建项目..."),
        ]
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("完成"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_error<B: Backend>(f: &mut Frame<B>, area: Rect, message: &str) {
    let paragraph = Paragraph::new(Line::from(format!("❌ 错误: {}", message)))
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

fn render_footer<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let help_text = match app.state {
        AppState::Welcome => "Enter/Space: 继续 • q/Esc: 退出",
        AppState::ProjectName | AppState::OutputDir | AppState::TemplateSearch => {
            "Enter: 继续 • Esc: 返回"
        }
        AppState::TemplateSelect => {
            "Enter: 选择 • ↑/↓: 导航 • Esc: 返回 • /: 搜索"
        }
        AppState::VersionSelect => {
            "Enter: 下载并确认 • ↑/↓: 切换版本 • Esc: 返回"
        }
        AppState::Variables => "Enter: 继续 • Esc: 返回",
        AppState::Complete => {
            if app.show_confirm_dialog {
                "Enter: 确认覆盖 • Esc: 取消"
            } else {
                "Enter: 创建 • q/Esc: 退出"
            }
        }
    };

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}
