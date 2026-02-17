// UI rendering module for ratatui application

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs},
};
use serde_json::Value;

// Import api module (declared in main.rs)
use crate::api::MihomoClient;

#[derive(Debug, Clone, PartialEq)]
pub enum AppPage {
    Proxy,
    Log,
    Settings,
    Config,
}

impl AppPage {
    pub fn title(&self) -> &'static str {
        match self {
            AppPage::Proxy => "Proxy",
            AppPage::Log => "Log",
            AppPage::Settings => "Settings",
            AppPage::Config => "Config",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            AppPage::Proxy => 0,
            AppPage::Log => 1,
            AppPage::Settings => 2,
            AppPage::Config => 3,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => AppPage::Proxy,
            1 => AppPage::Log,
            2 => AppPage::Settings,
            _ => AppPage::Config,
        }
    }

    pub fn next(&self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn previous(&self) -> Self {
        Self::from_index(self.index() + 3)
    }
}

pub struct AppState {
    pub current_page: AppPage,
    pub configs: Option<Value>,
    pub loading: bool,
    pub error: Option<String>,
    pub scroll_offset: u16,
    pub scroll_state: ScrollbarState,
    pub stdout_output: String,
    pub logs: Option<String>,
    pub logs_loading: bool,
    runtime: tokio::runtime::Runtime,
}

impl AppState {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        Self {
            current_page: AppPage::Proxy,
            configs: None,
            loading: false,
            error: None,
            scroll_offset: 0,
            scroll_state: ScrollbarState::new(0),
            stdout_output: String::new(),
            logs: None,
            logs_loading: false,
            runtime,
        }
    }

    pub fn update_stdout(&mut self, output: String) {
        self.stdout_output = output;
    }

    pub fn clear_stdout(&mut self) {
        self.stdout_output.clear();
    }

    /// Parse SSE format and extract log messages
    fn parse_sse_logs(&self, sse_data: &str) -> String {
        let mut lines = Vec::new();
        for line in sse_data.lines() {
            // Skip SSE meta lines (event:, id:, retry:)
            if line.starts_with("event:") || line.starts_with("id:") || line.starts_with("retry:") {
                continue;
            }
            // Extract data content
            let content = if line.starts_with("data:") {
                line.strip_prefix("data:").unwrap_or(line).trim_start()
            } else {
                line
            };
            if !content.is_empty() {
                lines.push(content.to_string());
            }
        }
        lines.join("\n")
    }

    /// Limit logs to last N lines to prevent memory bloat
    fn limit_log_lines(&self, logs: &str, max_lines: usize) -> String {
        let all_lines: Vec<&str> = logs.lines().collect();
        if all_lines.len() <= max_lines {
            return logs.to_string();
        }
        let start_index = all_lines.len() - max_lines;
        all_lines[start_index..].join("\n")
    }

    pub fn load_logs(&mut self) {
        self.logs_loading = true;

        let result = self.runtime.block_on(async {
            let client = MihomoClient::new("http://127.0.0.1:9097", "123456");
            client.get_logs(None).await
        });

        match result {
            Ok(resp) => {
                // For SSE streaming response, read all chunks
                let body = self.runtime.block_on(async { resp.text().await });
                match body {
                    Ok(text) => {
                        // Parse SSE format
                        let parsed_logs = self.parse_sse_logs(&text);
                        // Limit to last MAX_LOG_LINES lines
                        let limited_logs = self.limit_log_lines(&parsed_logs, 1000);

                        // Append to existing logs or create new
                        if let Some(existing) = &self.logs {
                            let combined = format!("{}\n{}", existing, limited_logs);
                            // Limit combined logs too
                            self.logs = Some(self.limit_log_lines(&combined, 1000));
                        } else {
                            self.logs = Some(limited_logs);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Error reading logs: {}", e);
                        if let Some(ref mut existing) = self.logs {
                            existing.push_str("\n");
                            existing.push_str(&error_msg);
                        } else {
                            self.logs = Some(error_msg);
                        }
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to load logs: {}", e);
                if let Some(ref mut existing) = self.logs {
                    existing.push_str("\n");
                    existing.push_str(&error_msg);
                } else {
                    self.logs = Some(error_msg);
                }
            }
        }

        self.logs_loading = false;
    }

    pub fn scroll_logs_up(&mut self) {
        // Implement log-specific scrolling if needed
    }

    pub fn scroll_logs_down(&mut self) {
        // Implement log-specific scrolling if needed
    }

    pub fn next_page(&mut self) {
        self.current_page = self.current_page.next();
        // Load configs when entering profile page
        if self.current_page == AppPage::Config && self.configs.is_none() && !self.loading {
            self.load_configs();
        }
    }

    pub fn previous_page(&mut self) {
        self.current_page = self.current_page.previous();
    }

    pub fn load_configs(&mut self) {
        self.loading = true;
        self.error = None;

        // Use tokio runtime to execute async API call
        let result = self.runtime.block_on(async {
            let client = MihomoClient::new("http://127.0.0.1:9097", "123456");
            client.get_configs().await
        });

        match result {
            Ok(configs) => {
                self.configs = Some(configs);
                // Count lines for scrollbar
                if let Some(ref configs) = self.configs {
                    let text = serde_json::to_string_pretty(configs).unwrap_or_default();
                    let lines = text.lines().count();
                    self.scroll_state = ScrollbarState::new(lines.saturating_sub(1));
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to load configs: {}", e));
            }
        }

        self.loading = false;
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
        self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
        self.scroll_state = self.scroll_state.position(self.scroll_offset as usize);
    }
}

fn render_proxy_page(f: &mut Frame, area: Rect) {
    let content = Paragraph::new("Proxy Page")
        .alignment(Alignment::Center)
        .block(Block::default().title("Proxy").borders(Borders::ALL));
    f.render_widget(content, area);
}

fn render_log_page(f: &mut Frame, area: Rect, app: &mut AppState) {
    let content = if app.logs_loading {
        Paragraph::new("Loading logs...".to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow))
    } else if let Some(ref logs) = app.logs {
        Paragraph::new(logs.clone())
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White))
    } else {
        Paragraph::new("Press 'L' to load logs")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray))
    };

    let block = Block::default().title("Log").borders(Borders::ALL);
    f.render_widget(block, area);
    f.render_widget(content, area);
}

fn render_settings_page(f: &mut Frame, area: Rect) {
    let content = Paragraph::new("Settings Page")
        .alignment(Alignment::Center)
        .block(Block::default().title("Settings").borders(Borders::ALL));
    f.render_widget(content, area);
}

fn render_config_page(f: &mut Frame, area: Rect, app: &mut AppState) {
    let block = Block::default().title("Config").borders(Borders::ALL);

    let content = if app.loading {
        Paragraph::new("Loading configs...".to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow))
    } else if let Some(ref error) = app.error {
        Paragraph::new(error.clone())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red))
    } else if let Some(ref configs) = app.configs {
        let text = serde_json::to_string_pretty(configs).unwrap_or_default();
        Paragraph::new(text)
            .alignment(Alignment::Left)
            .scroll((app.scroll_offset, 0))
    } else {
        Paragraph::new("Press Enter or R to load configs")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray))
    };

    f.render_widget(block, area);
}

impl AppState {
    fn total_lines(&self) -> u16 {
        if let Some(ref configs) = self.configs {
            let text = serde_json::to_string_pretty(configs).unwrap_or_default();
            text.lines().count() as u16
        } else {
            0
        }
    }
}

fn render_stdout_block(f: &mut Frame, area: Rect, app: &mut AppState) {
    let content = if app.stdout_output.is_empty() {
        Paragraph::new("No output")
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::Gray))
    } else {
        Paragraph::new(app.stdout_output.clone())
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::Green))
    };

    let block = Block::default()
        .title("Standard Output")
        .borders(Borders::ALL);

    f.render_widget(block, area);
    f.render_widget(content, area);
}

pub fn render_bottom_nav_bar(f: &mut Frame, area: Rect, current_page: &AppPage) {
    let titles = vec!["Proxy", "Log", "Settings", "Profile"];
    let selected_index = current_page.index();

    let tabs = Tabs::new(titles)
        .select(selected_index)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");

    f.render_widget(tabs, area);
}

pub fn render_main_content(f: &mut Frame, area: Rect, app: &mut AppState) {
    match app.current_page {
        AppPage::Proxy => render_proxy_page(f, area),
        AppPage::Log => render_log_page(f, area, app),
        AppPage::Settings => render_settings_page(f, area),
        AppPage::Config => render_config_page(f, area, app),
    }
}

pub fn render_ui(f: &mut Frame, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Min(1),
            Constraint::Length(5),
            Constraint::Length(2),
        ])
        .split(f.area());

    let main_area = chunks[0];
    let stdout_area = chunks[1];
    let nav_area = chunks[2];

    render_main_content(f, main_area, app);
    render_stdout_block(f, stdout_area, app);
    render_bottom_nav_bar(f, nav_area, &app.current_page);
}
