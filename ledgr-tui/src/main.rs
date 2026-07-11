mod app;
mod ui;

use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ledgr_core::Db;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

fn data_dir_db_path() -> anyhow::Result<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "ledgr", "ledgr")
        .ok_or_else(|| anyhow::anyhow!("could not determine a data directory for this platform"))?;
    let dir = dirs.data_dir();
    std::fs::create_dir_all(dir)?;
    Ok(dir.join("ledgr.db"))
}

fn main() -> anyhow::Result<()> {
    let db_path = data_dir_db_path()?;
    let db = Db::open(&db_path)?;
    let mut app = App::new(db)?;

    enable_raw_mode()?;
    let mut out = stdout();
    out.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => match app.screen {
                        app::Screen::Transactions => app.back(),
                        app::Screen::Accounts => app.should_quit = true,
                    },
                    KeyCode::Char('c')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        app.should_quit = true;
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
                    KeyCode::Enter => {
                        if app.screen == app::Screen::Accounts {
                            app.open_selected_account()?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
