// clashtui/yact/src/main.rs

mod api;
mod ui;

use ui::*;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

fn main() -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;

    execute!(stdout, Clear(ClearType::All))?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = AppState::new();
    let mut running = true;

    while running {
        terminal.draw(|frame| {
            render_ui(frame, &mut app);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Left => app.previous_page(),
                    KeyCode::Right => app.next_page(),
                    KeyCode::Up => app.scroll_up(),
                    KeyCode::Down => app.scroll_down(),
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if app.current_page == ui::AppPage::Config {
                            app.configs = None;
                            app.load_configs();
                        }
                    }
                    KeyCode::Enter => {
                        if app.current_page == ui::AppPage::Config && app.configs.is_none() {
                            app.load_configs();
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => running = false,
                    KeyCode::Esc => running = false,
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        if app.current_page == ui::AppPage::Log {
                            app.load_logs();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), Clear(ClearType::All))?;
    terminal.show_cursor()?;

    Ok(())
}
