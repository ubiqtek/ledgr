mod analysis;
mod app;
mod config;
mod db;
mod import;
mod inbox;
mod model;
mod ui;

use app::App;
use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use db::Db;
use inbox::Inbox;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

/// XDG-style, `~/.local/share/ledgr/ledgr.db` on every platform — matches
/// `Config::default_path`'s `~/.config/ledgr` (see
/// `doc/adr/0005-xdg-data-location.md`), deliberately not the platform-native
/// data directory.
fn data_dir_db_path() -> anyhow::Result<std::path::PathBuf> {
    let home = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("could not determine the home directory for this platform"))?
        .home_dir()
        .to_path_buf();
    let dir = home.join(".local/share/ledgr");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("ledgr.db"))
}

/// Scans the configured inbox for statement files, imports any not seen
/// before, and moves each into `processed/` once handled. Run via
/// `ledgr import`.
fn run_import(db: Db) -> anyhow::Result<()> {
    let config = Config::load_or_init(&Config::default_path()?)?;
    let inbox = Inbox::new(config.inbox_dir.clone());
    let summary = import::import_inbox(&db, &inbox)?;

    println!(
        "imported {} file(s), {} transaction(s); skipped {} file(s) already imported",
        summary.files_imported, summary.transactions_imported, summary.files_skipped
    );
    println!("inbox: {}", config.inbox_dir.display());
    Ok(())
}

/// Prints a summary of every account: balance, transaction count, date
/// range covered, and when it was last imported into. Run via
/// `ledgr status`.
fn run_status(db: Db) -> anyhow::Result<()> {
    let mut statuses = db.account_statuses()?;
    if statuses.is_empty() {
        println!("no accounts yet — run `ledgr import` first");
        return Ok(());
    }

    let config = Config::load_or_init(&Config::default_path()?)?;
    config.apply_account_name_overrides(statuses.iter_mut().map(|s| &mut s.account));

    for status in &statuses {
        let account = &status.account;
        let institution = account.institution.as_deref().unwrap_or("unknown institution");
        println!(
            "{} ({institution}) — {}, {}",
            account.name,
            account.account_type.as_str(),
            account.currency
        );
        match (status.balance_minor, &status.balance_as_of) {
            (Some(balance), Some(as_of)) => println!(
                "  balance:       {} (as of {as_of})",
                format_amount_minor(balance, &account.currency)
            ),
            _ => println!("  balance:       unknown (no balance data imported yet)"),
        }
        println!("  transactions:  {}", status.transaction_count);
        match (&status.earliest_transaction, &status.latest_transaction) {
            (Some(earliest), Some(latest)) => {
                println!("  date range:    {earliest} to {latest}")
            }
            _ => println!("  date range:    (no transactions)"),
        }
        println!(
            "  last imported: {}",
            status.last_imported_at.as_deref().unwrap_or("never")
        );
        println!();
    }
    Ok(())
}

/// Sets the display name shown for the account whose bank-generated name
/// carries the given last-4 digits (e.g. `"Barclays Current Account
/// (...5678)"`), overriding the bank's own naming everywhere ledgr shows
/// account names. Run via `ledgr name-account <last4> "<name>"`.
fn run_name_account(db: Db, last4: &str, name: &str) -> anyhow::Result<()> {
    let config_path = Config::default_path()?;
    let mut config = Config::load_or_init(&config_path)?;
    config.set_account_name(last4, name);
    config.save(&config_path)?;

    let matches = db
        .list_accounts()?
        .into_iter()
        .any(|account| account.name.contains(&format!("(...{last4})")));
    if matches {
        println!("{last4} -> \"{name}\"");
    } else {
        println!(
            "{last4} -> \"{name}\" (saved, but no imported account currently ends in ...{last4})"
        );
    }
    Ok(())
}

/// Formats a signed minor-unit amount (e.g. pence) as a major-unit string
/// with the currency code, e.g. `-4550` GBP -> `-45.50 GBP`.
fn format_amount_minor(amount_minor: i64, currency: &str) -> String {
    let sign = if amount_minor < 0 { "-" } else { "" };
    let major = amount_minor.unsigned_abs();
    format!("{sign}{}.{:02} {currency}", major / 100, major % 100)
}

fn main() -> anyhow::Result<()> {
    let db_path = data_dir_db_path()?;
    let db = Db::open(&db_path)?;

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("import") => return run_import(db),
        Some("status") => return run_status(db),
        Some("name-account") => {
            let (Some(last4), Some(name)) = (args.get(2), args.get(3)) else {
                anyhow::bail!("usage: ledgr name-account <last-4-digits> \"<name>\"");
            };
            return run_name_account(db, last4, name);
        }
        _ => {}
    }

    let mut app = App::new(db)?;

    enable_raw_mode()?;
    let mut out = stdout();
    out.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

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
    // Set when a lone `g` was just pressed, waiting to see if a second `g`
    // follows (nvim's `gg` "go to top"); cleared on any other key.
    let mut pending_g = false;

    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        // Visible list height (borders + status line taken off), used as the
        // Ctrl-d/Ctrl-u page-scroll distance.
        let page = terminal.size()?.height.saturating_sub(3).max(1) as i32;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                let ctrl = key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL);
                let was_pending_g = pending_g;
                pending_g = false;
                match key.code {
                    KeyCode::Char('g') if was_pending_g => app.select_first(),
                    KeyCode::Char('g') => pending_g = true,
                    KeyCode::Char('G') => app.select_last(),
                    KeyCode::Char('c') if ctrl => app.should_quit = true,
                    KeyCode::Char('d') if ctrl => app.move_selection(page),
                    KeyCode::Char('u') if ctrl => app.move_selection(-page),
                    KeyCode::Char('q') | KeyCode::Esc => match app.screen {
                        app::Screen::Accounts => app.should_quit = true,
                        app::Screen::Transactions | app::Screen::Help => app.back(),
                    },
                    KeyCode::Char('?') => app.toggle_help(),
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
