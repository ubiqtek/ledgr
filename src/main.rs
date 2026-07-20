mod analysis;
mod app;
mod config;
mod db;
mod derive;
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

/// Scans the configured inbox for import files, imports any not seen
/// before, moves each into `processed/` once handled, then runs the spend
/// ledger derivation pass over any newly-imported (or previously
/// unclassified) transactions. Run via `ledgr import`.
fn run_import(db: Db) -> anyhow::Result<()> {
    let config = Config::load_or_init(&Config::default_path()?)?;
    let inbox = Inbox::new(config.inbox_dir.clone());
    let summary = import::import_inbox(&db, &inbox)?;

    println!(
        "imported {} file(s), {} transaction(s); skipped {} file(s) already imported, {} transaction(s) already imported",
        summary.files_imported,
        summary.transactions_imported,
        summary.files_skipped,
        summary.transactions_deduplicated
    );
    println!("inbox: {}", config.inbox_dir.display());

    let derivation = derive::run_derivation(
        &db,
        &config.household_accounts,
        &config.income_sources,
        &config.registered_people,
        &config.reimbursement_sources,
    )?;
    println!(
        "spend ledger: {} entries created, {} internal transfer(s) detected ({} paired, {} backfilled), {} credit card payment(s) matched ({} still unmatched), {} out of scope",
        derivation.spend_entries_created,
        derivation.transfers_detected,
        derivation.transfers_paired,
        derivation.transfers_backfilled,
        derivation.card_payments_matched,
        derivation.card_payments_unmatched,
        derivation.out_of_scope
    );
    println!(
        "income ledger: {} entries created",
        derivation.income_entries_created
    );
    Ok(())
}

/// Prints a summary of every account: balance, transaction count, date
/// range covered, and when it was last imported into. Run via
/// `ledgr status`.
fn run_status(db: Db) -> anyhow::Result<()> {
    let mut statuses = db.account_statuses()?;
    let config = Config::load_or_init(&Config::default_path()?)?;

    if statuses.is_empty() && config.household_accounts.is_empty() {
        println!("no accounts yet — run `ledgr import` first");
        return Ok(());
    }

    config.apply_account_name_overrides(statuses.iter_mut().map(|s| &mut s.account));

    if !statuses.is_empty() {
        println!("Tracked Accounts:");
        println!();

        let mut rows: Vec<Vec<String>> = statuses
            .iter()
            .map(|status| {
                let account = &status.account;
                let balance = match status.balance_minor {
                    Some(balance) => format_amount_minor(balance, &account.currency),
                    None => "unknown".to_string(),
                };
                let date_range = match (&status.earliest_transaction, &status.latest_transaction) {
                    (Some(earliest), Some(latest)) => format!("{earliest} to {latest}"),
                    _ => "(no transactions)".to_string(),
                };
                let account_column = match &account.account_number {
                    Some(account_number) => last4_str(account_number),
                    None => status
                        .card_last4
                        .as_deref()
                        .map(|last4| format!("({last4})"))
                        .unwrap_or_else(|| "-".to_string()),
                };
                vec![
                    account.name.clone(),
                    account_column,
                    balance,
                    status.transaction_count.to_string(),
                    date_range,
                    status
                        .last_imported_at
                        .clone()
                        .unwrap_or_else(|| "never".to_string()),
                ]
            })
            .collect();
        align_decimal_column(&mut rows, 2);

        print_table(
            &[
                "Name",
                "Account",
                "Balance",
                "Txns",
                "Date Range",
                "Last Imported",
            ],
            &rows,
            &[2, 3],
        );
        println!();
    }

    if !config.household_accounts.is_empty() {
        println!("Household Reference Accounts (no balance/transaction data — for transfer detection only):");
        println!();

        let rows: Vec<Vec<String>> = config
            .household_accounts
            .iter()
            .map(|account| {
                vec![
                    account
                        .label
                        .clone()
                        .unwrap_or_else(|| "(no label)".to_string()),
                    last4_str(&account.account_number),
                ]
            })
            .collect();

        print_table(&["Label", "Account"], &rows, &[]);
        println!();
    }

    if !config.income_sources.is_empty()
        || !config.registered_people.is_empty()
        || !config.reimbursement_sources.is_empty()
    {
        println!("Named Entities (drive income/reimbursement classification rules):");
        println!();

        let mut rows: Vec<Vec<String>> = config
            .income_sources
            .iter()
            .map(|source| {
                vec![
                    source.label.clone().unwrap_or_else(|| source.name.clone()),
                    source.kind.display().to_string(),
                    source
                        .full_name
                        .clone()
                        .unwrap_or_else(|| source.name.clone()),
                    source.name.clone(),
                ]
            })
            .collect();
        rows.extend(config.registered_people.iter().map(|person| {
            vec![
                person.label.clone().unwrap_or_else(|| person.name.clone()),
                "Friend/Family".to_string(),
                person
                    .full_name
                    .clone()
                    .unwrap_or_else(|| person.name.clone()),
                person.name.clone(),
            ]
        }));
        rows.extend(config.reimbursement_sources.iter().map(|source| {
            vec![
                source.label.clone().unwrap_or_else(|| source.name.clone()),
                source.kind.clone(),
                source
                    .full_name
                    .clone()
                    .unwrap_or_else(|| source.name.clone()),
                source.name.clone(),
            ]
        }));

        print_table(&["Label", "Type", "Name", "Matches"], &rows, &[]);
        println!();
    }

    let spend_ledger = db.spend_ledger_summary()?;
    println!("Spend Ledger:");
    println!("  {} entries", spend_ledger.entries);
    println!();

    let transfer_ledger = db.transfer_ledger_summary()?;
    let (unpaired_reference, unpaired_unresolved) =
        db.unpaired_transfer_counterparties()?.into_iter().fold(
            (0, 0),
            |(reference, unresolved), (sort, account)| match (sort, account) {
                (Some(sort), Some(account))
                    if config.household_account_matches(&sort, &account) =>
                {
                    (reference + 1, unresolved)
                }
                _ => (reference, unresolved + 1),
            },
        );
    println!("Transfer Ledger:");
    println!(
        "  {} entries ({} paired, {} unpaired: {} to reference accounts, {} unresolved)",
        transfer_ledger.entries,
        transfer_ledger.paired,
        transfer_ledger.unpaired,
        unpaired_reference,
        unpaired_unresolved
    );
    if transfer_ledger.card_payments_matched > 0 || transfer_ledger.card_payments_unmatched > 0 {
        println!(
            "  credit card payments: {} matched, {} unmatched",
            transfer_ledger.card_payments_matched, transfer_ledger.card_payments_unmatched
        );
    }
    println!();

    Ok(())
}

fn last4_str(value: &str) -> String {
    if value.len() > 4 {
        format!("({})", &value[value.len() - 4..])
    } else {
        value.to_string()
    }
}

/// Left-pads every cell in a column with spaces so their `.` characters line
/// up (e.g. `"7.47 GBP"` and `"3106.58 GBP"` both end up with their decimal
/// point in the same screen column). Cells with no `.` (e.g. "unknown") are
/// left as-is.
fn align_decimal_column(rows: &mut [Vec<String>], col: usize) {
    let max_int_len = rows
        .iter()
        .filter_map(|row| row[col].find('.'))
        .max()
        .unwrap_or(0);
    for row in rows.iter_mut() {
        if let Some(dot) = row[col].find('.') {
            let pad = max_int_len - dot;
            if pad > 0 {
                row[col] = format!("{}{}", " ".repeat(pad), row[col]);
            }
        }
    }
}

/// Prints a left-aligned table: a header row, then one row per data row,
/// with each column padded to the widest cell (header or data) in that
/// column. The last column is left unpadded to avoid trailing whitespace.
/// Columns whose index appears in `right_aligned` are right-aligned instead
/// of left-aligned (e.g. a numeric count column).
fn print_table(headers: &[&str], rows: &[Vec<String>], right_aligned: &[usize]) {
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(i, header)| {
            rows.iter()
                .map(|row| row[i].len())
                .chain(std::iter::once(header.len()))
                .max()
                .unwrap_or(header.len())
        })
        .collect();

    let format_row = |cells: &[String]| -> String {
        cells
            .iter()
            .zip(&widths)
            .enumerate()
            .map(|(i, (cell, width))| {
                if i == cells.len() - 1 {
                    cell.clone()
                } else if right_aligned.contains(&i) {
                    format!("{cell:>width$}")
                } else {
                    format!("{cell:<width$}")
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    };

    let header_cells: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
    println!("  {}", format_row(&header_cells));
    for row in rows {
        println!("  {}", format_row(row));
    }
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

/// Sets (or, given an empty string, clears) a spend entry's note — e.g.
/// recording that an unrecognised merchant was looked into and is
/// legitimate. Run via `ledgr note <spend-entry-id> "<text>"`. Also usable
/// non-interactively (no TUI needed), which is the point: the same action
/// as the TUI's `n` key on the spend drill-down screen, callable by script
/// or by an assistant working alongside the user.
fn run_note(db: Db, id_arg: &str, text: &str) -> anyhow::Result<()> {
    let id: crate::model::Id = id_arg
        .parse()
        .map_err(|_| anyhow::anyhow!("not a valid spend entry id: {id_arg}"))?;
    let note = if text.trim().is_empty() {
        None
    } else {
        Some(text.trim())
    };
    db.set_spend_entry_note(id, note)?;
    match note {
        Some(text) => println!("spend entry {id}: note set to \"{text}\""),
        None => println!("spend entry {id}: note cleared"),
    }
    Ok(())
}

/// Copies `text` to the system clipboard via `pbcopy` (`y` on a selected
/// row) — macOS-only, matching this app's only supported platform today.
/// Best-effort: a missing/failing `pbcopy` is swallowed by the caller rather
/// than surfaced, since it's not worth crashing the TUI over.
fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;
    child
        .stdin
        .as_mut()
        .expect("stdin piped above")
        .write_all(text.as_bytes())?;
    child.wait()?;
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
        Some("note") => {
            let (Some(id), Some(text)) = (args.get(2), args.get(3)) else {
                anyhow::bail!("usage: ledgr note <spend-entry-id> \"<text>\"");
            };
            return run_note(db, id, text);
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
    // Set when `<space>` (the leader key) was just pressed, waiting for the
    // next keypress to dispatch a top-level navigation jump; cleared on any
    // other key, same pattern as `pending_g`.
    let mut pending_leader = false;

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

                // While editing a note, every key is text input (or
                // commit/cancel) rather than navigation — handled entirely
                // separately from the match below.
                if app.note_edit.is_some() {
                    match key.code {
                        KeyCode::Enter => app.commit_note()?,
                        KeyCode::Esc => app.cancel_editing_note(),
                        KeyCode::Backspace => {
                            app.note_edit.as_mut().expect("checked above").pop();
                        }
                        KeyCode::Char(c) => {
                            app.note_edit.as_mut().expect("checked above").push(c);
                        }
                        _ => {}
                    }
                    continue;
                }

                // While the "add reference" form is open, every key is text
                // input (or field navigation/commit/cancel) rather than
                // list navigation — same pattern as `note_edit` above.
                if app.person_form.is_some() {
                    match key.code {
                        KeyCode::Enter => app.person_form_enter()?,
                        KeyCode::Esc => app.cancel_person_form(),
                        KeyCode::Tab | KeyCode::Down => app.person_form_next_field(),
                        KeyCode::BackTab | KeyCode::Up => app.person_form_previous_field(),
                        KeyCode::Backspace => app.person_form_pop_char(),
                        KeyCode::Char(c) => app.person_form_push_char(c),
                        _ => {}
                    }
                    continue;
                }

                // While the "record a spend from this transfer" form is
                // open, every key is text input (or field
                // navigation/commit/cancel) rather than list navigation —
                // same pattern as `person_form` above.
                if app.spend_form.is_some() {
                    match key.code {
                        KeyCode::Enter => app.spend_form_enter()?,
                        KeyCode::Esc => app.cancel_spend_form(),
                        KeyCode::Tab | KeyCode::Down => app.spend_form_next_field(),
                        KeyCode::BackTab | KeyCode::Up => app.spend_form_previous_field(),
                        KeyCode::Backspace => app.spend_form_pop_char(),
                        KeyCode::Char(c) => app.spend_form_push_char(c),
                        _ => {}
                    }
                    continue;
                }

                // While the transfer filter box is open, every key is text
                // input (or confirm/cancel/clear) rather than navigation —
                // same pattern as `note_edit` above, except `Enter` keeps
                // the filter applied (only stops editing it) rather than
                // clearing it.
                if app.transfer_filter_editing {
                    match key.code {
                        KeyCode::Enter => app.confirm_transfer_filter(),
                        KeyCode::Esc => app.cancel_transfer_filter(),
                        KeyCode::Char('g') if ctrl => app.clear_transfer_filter(),
                        KeyCode::Backspace => app.transfer_filter_pop_char(),
                        KeyCode::Char(c) => app.transfer_filter_push_char(c),
                        _ => {}
                    }
                    continue;
                }

                // While the transfer-detail popup is open, any key dismisses
                // it rather than being treated as navigation.
                if app.transfer_detail.is_some() {
                    app.close_transfer_detail();
                    continue;
                }

                // While the income-detail popup is open, any key dismisses
                // it rather than being treated as navigation.
                if app.income_detail.is_some() {
                    app.close_income_detail();
                    continue;
                }

                // While the Gap screen's Salary/Other breakdown popup is
                // open, any key dismisses it rather than being treated as
                // navigation.
                if app.gap_detail.is_some() {
                    app.close_gap_detail();
                    continue;
                }

                let was_pending_g = pending_g;
                pending_g = false;
                let was_pending_leader = pending_leader;
                pending_leader = false;

                if was_pending_leader {
                    match key.code {
                        KeyCode::Char('a') => app.navigate_to(app::Screen::Accounts),
                        KeyCode::Char('s') => app.open_monthly_spend()?,
                        KeyCode::Char('i') => app.open_monthly_income()?,
                        KeyCode::Char('t') => app.open_monthly_transfers()?,
                        KeyCode::Char('g') => app.open_gap()?,
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char(' ') => pending_leader = true,
                    KeyCode::Char('c') if ctrl => app.should_quit = true,
                    KeyCode::Char('d') if ctrl => app.move_selection(page),
                    KeyCode::Char('u') if ctrl => app.move_selection(-page),
                    KeyCode::Char('g') if ctrl => app.clear_transfer_filter(),
                    KeyCode::Char('g') if was_pending_g => app.select_first(),
                    KeyCode::Char('g') => pending_g = true,
                    KeyCode::Char('G') => app.select_last(),
                    KeyCode::Char('f') if app.screen == app::Screen::TransferMonth => {
                        app.start_transfer_filter();
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app.can_go_back() {
                            app.back()?;
                        } else {
                            app.should_quit = true;
                        }
                    }
                    KeyCode::Char('?') => app.toggle_help()?,
                    KeyCode::Char('n')
                        if matches!(
                            app.screen,
                            app::Screen::SpendMonth
                                | app::Screen::IncomeMonth
                                | app::Screen::TransferMonth
                        ) =>
                    {
                        app.start_editing_note();
                    }
                    KeyCode::Char('i') if app.screen == app::Screen::TransferMonth => {
                        app.show_transfer_detail()?;
                    }
                    KeyCode::Char('i') if app.screen == app::Screen::SpendMonth => {
                        app.show_spend_transfer_detail()?;
                    }
                    KeyCode::Char('s') if app.screen == app::Screen::TransferMonth => {
                        app.start_spend_from_transfer();
                    }
                    KeyCode::Char('i') if app.screen == app::Screen::IncomeMonth => {
                        app.show_income_detail()?;
                    }
                    KeyCode::Char('i') if app.screen == app::Screen::Gap => {
                        app.show_gap_detail();
                    }
                    KeyCode::Char('a') if app.screen == app::Screen::IncomeMonth => {
                        app.start_adding_person();
                    }
                    KeyCode::Char('y') => {
                        if let Some(text) = app.selected_row_text() {
                            let _ = copy_to_clipboard(&text);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
                    KeyCode::Enter => match app.screen {
                        app::Screen::Accounts => app.open_selected_account()?,
                        app::Screen::MonthlySpend => app.open_selected_month()?,
                        app::Screen::MonthlyIncome => app.open_selected_income_month()?,
                        app::Screen::MonthlyTransfers => app.open_selected_transfer_month()?,
                        app::Screen::Gap => app.open_selected_gap_month()?,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
