use crate::app::{App, Screen};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;

const SELECTED_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::White)
    .add_modifier(Modifier::BOLD);

pub fn draw(frame: &mut Frame, app: &mut App) {
    // The transitions between screens use different widget layouts (e.g.
    // Table columns vs. plain List rows), so a shorter cell can leave a
    // previous frame's characters showing through unless the whole frame is
    // cleared first.
    frame.render_widget(Clear, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.area());

    match app.screen {
        Screen::Accounts => draw_accounts(frame, app, chunks[0]),
        Screen::Transactions => draw_transactions(frame, app, chunks[0]),
        Screen::MonthlySpend => draw_monthly_spend(frame, app, chunks[0]),
        Screen::SpendMonth => draw_spend_month(frame, app, chunks[0]),
        Screen::MonthlyTransfers => draw_monthly_transfers(frame, app, chunks[0]),
        Screen::TransferMonth => draw_transfer_month(frame, app, chunks[0]),
        Screen::Help => draw_help(frame, chunks[0]),
    }
    draw_status(frame, app, chunks[1]);

    if let Some(buffer) = &app.note_edit {
        draw_note_editor(frame, buffer, frame.area());
    }

    if let Some(detail) = &app.transfer_detail {
        draw_transfer_detail(frame, detail, frame.area());
    }
}

/// A small centred popup for editing a spend entry's note, overlaid on top
/// of whatever screen is behind it (always `Screen::SpendMonth` today).
fn draw_note_editor(frame: &mut Frame, buffer: &str, area: Rect) {
    let popup = centered_rect(60, 3, area);
    frame.render_widget(Clear, popup);
    let paragraph = Paragraph::new(format!("{buffer}\u{2588}")).block(
        Block::default()
            .title("Note (Enter to save, Esc to cancel)")
            .borders(Borders::ALL),
    );
    frame.render_widget(paragraph, popup);
}

/// Shows both legs of a transfer side by side (`i` on
/// `Screen::TransferMonth`) — the selected entry plus, when found, its
/// counterpart transaction. The counterpart is `None` when the other side
/// isn't a recorded transaction at all (e.g. a Reference Household Account,
/// which by definition has no imports) rather than a lookup failure, so
/// that's stated plainly instead of looking like an error.
fn draw_transfer_detail(frame: &mut Frame, detail: &crate::app::TransferDetail, area: Rect) {
    let popup = centered_rect(70, 8, area);
    frame.render_widget(Clear, popup);

    let own_amount = crate::format_amount_minor(detail.own.amount_minor, &detail.own.currency);
    let mut lines = vec![
        format!(
            "{}  {}  {}",
            detail.own.posted_at, own_amount, detail.own_account_name
        ),
        detail.own.description.clone(),
        String::new(),
    ];
    match &detail.counterpart {
        Some(counterpart) => {
            let counterpart_amount =
                crate::format_amount_minor(counterpart.amount_minor, &counterpart.currency);
            lines.push(format!(
                "{}  {}  {}",
                counterpart.posted_at, counterpart_amount, detail.counterpart_label
            ));
            lines.push(counterpart.description.clone());
        }
        None => {
            lines.push(format!(
                "No matching transaction recorded for \"{}\" — likely a Reference \
                 Household Account (never imported) rather than a missing match.",
                detail.counterpart_label
            ));
        }
    }

    let paragraph = Paragraph::new(lines.join("\n")).block(
        Block::default()
            .title("Both legs of this transfer (any key to close)")
            .borders(Borders::ALL),
    );
    frame.render_widget(paragraph, popup);
}

/// A `width_pct`-wide, `height`-tall rectangle centred within `area`.
fn centered_rect(width_pct: u16, height: u16, area: Rect) -> Rect {
    let width = area.width * width_pct / 100;
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

fn draw_accounts(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default().title("Accounts").borders(Borders::ALL);

    if app.accounts.is_empty() {
        frame.render_widget(
            Paragraph::new("No accounts yet. Import a file to get started.").block(block),
            area,
        );
        return;
    }

    let rows = app
        .accounts
        .iter()
        .map(|status| {
            let account = &status.account;
            let balance = match status.balance_minor {
                Some(minor) => crate::format_amount_minor(minor, &account.currency),
                None => "unknown".to_string(),
            };
            // last_imported_at is an ISO 8601 UTC timestamp, e.g.
            // "2026-07-11T16:53:10.605Z"; show date + hours:minutes, dropping
            // seconds/fractional seconds and the "Z" as more precision than is
            // useful in a column this narrow.
            let last_imported = status
                .last_imported_at
                .as_deref()
                .map(|ts| match ts.split_once('T') {
                    Some((date, time)) => format!("{date} {}", &time[..time.len().min(5)]),
                    None => ts.to_string(),
                })
                .unwrap_or_else(|| "never".to_string());
            let last4 = account
                .account_number
                .as_deref()
                .map(|n| n[n.len().saturating_sub(4)..].to_string())
                .or_else(|| status.card_last4.clone())
                .unwrap_or_else(|| "----".to_string());
            Row::new(vec![
                Cell::from(account.name.clone()),
                Cell::from(last4),
                Cell::from(account.account_type.as_str()),
                Cell::from(account.institution.clone().unwrap_or_default()),
                Cell::from(Line::from(balance).alignment(Alignment::Right)),
                Cell::from(last_imported),
            ])
        })
        .collect::<Vec<_>>();

    let table = Table::new(
        rows,
        [
            Constraint::Length(30),
            Constraint::Length(6),
            Constraint::Length(11),
            Constraint::Length(14),
            Constraint::Length(15),
            Constraint::Length(16),
        ],
    )
    .column_spacing(1)
    .block(block)
    .highlight_style(SELECTED_STYLE);

    app.accounts_table_state.select(Some(app.selected_account));
    frame.render_stateful_widget(table, area, &mut app.accounts_table_state);
}

fn draw_transactions(frame: &mut Frame, app: &mut App, area: Rect) {
    let account_name = app
        .accounts
        .get(app.selected_account)
        .map(|s| s.account.name.as_str())
        .unwrap_or("Transactions");
    let block = Block::default()
        .title(format!("Transactions \u{2014} {account_name}"))
        .borders(Borders::ALL);

    if app.transactions.is_empty() {
        frame.render_widget(
            Paragraph::new("No transactions for this account yet.").block(block),
            area,
        );
        return;
    }

    let rows = app
        .transactions
        .iter()
        .map(|tx| {
            let amount = tx.amount_minor as f64 / 100.0;
            Row::new(vec![
                Cell::from(tx.posted_at.clone()),
                Cell::from(format!("{amount:>10.2}")),
                Cell::from(tx.currency.clone()),
                Cell::from(tx.description.clone()),
            ])
        })
        .collect::<Vec<_>>();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Fill(1),
        ],
    )
    .column_spacing(1)
    .block(block)
    .highlight_style(SELECTED_STYLE);

    app.transactions_table_state
        .select(Some(app.selected_transaction));
    frame.render_stateful_widget(table, area, &mut app.transactions_table_state);
}

fn draw_monthly_spend(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default().title("Monthly Spend").borders(Borders::ALL);

    if app.monthly_spend.is_empty() {
        frame.render_widget(
            Paragraph::new("No spend entries yet. Run `ledgr import` first.").block(block),
            area,
        );
        return;
    }

    let rows = app
        .monthly_spend
        .iter()
        .map(|month| {
            let spend = crate::format_amount_minor(month.spend_minor.abs(), "GBP");
            Row::new(vec![
                Cell::from(month.month.clone()),
                Cell::from(Line::from(spend).alignment(Alignment::Right)),
            ])
        })
        .collect::<Vec<_>>();

    let table = Table::new(rows, [Constraint::Length(7), Constraint::Length(15)])
        .column_spacing(1)
        .block(block)
        .highlight_style(SELECTED_STYLE);

    app.monthly_spend_table_state.select(Some(app.selected_month));
    frame.render_stateful_widget(table, area, &mut app.monthly_spend_table_state);
}

fn draw_spend_month(frame: &mut Frame, app: &mut App, area: Rect) {
    let month = app
        .monthly_spend
        .get(app.selected_month)
        .map(|m| m.month.as_str())
        .unwrap_or("Spend");
    let block = Block::default()
        .title(format!("Spend \u{2014} {month}"))
        .borders(Borders::ALL);

    if app.spend_month_entries.is_empty() {
        frame.render_widget(
            Paragraph::new("No spend entries for this month.").block(block),
            area,
        );
        return;
    }

    let rows = app
        .spend_month_entries
        .iter()
        .map(|row| {
            let entry = &row.entry;
            let amount = crate::format_amount_minor(entry.amount_minor, &entry.currency);
            let rule = entry
                .rule_name
                .clone()
                .unwrap_or_else(|| entry.classified_by.as_str().to_string());
            let account_name = app
                .accounts
                .iter()
                .find(|s| s.account.id == row.account_id)
                .map(|s| s.account.name.as_str())
                .unwrap_or("?");
            let description = match &entry.note {
                Some(note) => format!("{}  \u{1f4dd} {note}", entry.description),
                None => entry.description.clone(),
            };
            Row::new(vec![
                Cell::from(entry.occurred_on.clone()),
                Cell::from(Line::from(amount).alignment(Alignment::Right)),
                Cell::from(entry.counterparty.clone().unwrap_or_default()),
                Cell::from(description),
                Cell::from(rule),
                Cell::from(account_name.to_string()),
            ])
        })
        .collect::<Vec<_>>();

    let header = Row::new(vec![
        "Date",
        "Amount",
        "Counterparty",
        "Description",
        "Rule",
        "Account",
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(20),
            Constraint::Fill(1),
            Constraint::Length(20),
            Constraint::Length(22),
        ],
    )
    .header(header)
    .column_spacing(1)
    .block(block)
    .highlight_style(SELECTED_STYLE);

    app.spend_month_table_state
        .select(Some(app.selected_spend_entry));
    frame.render_stateful_widget(table, area, &mut app.spend_month_table_state);
}

fn draw_monthly_transfers(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title("Monthly Transfers")
        .borders(Borders::ALL);

    if app.monthly_transfers.is_empty() {
        frame.render_widget(
            Paragraph::new("No internal transfers found. Run `ledgr import` first.").block(block),
            area,
        );
        return;
    }

    let rows = app
        .monthly_transfers
        .iter()
        .map(|month| {
            let out = crate::format_amount_minor(month.transferred_out_minor.abs(), "GBP");
            let inn = crate::format_amount_minor(month.transferred_in_minor, "GBP");
            Row::new(vec![
                Cell::from(month.month.clone()),
                Cell::from(Line::from(out).alignment(Alignment::Right)),
                Cell::from(Line::from(inn).alignment(Alignment::Right)),
            ])
        })
        .collect::<Vec<_>>();

    let header = Row::new(vec!["Month", "Transferred Out", "Transferred In"])
        .style(Style::default().add_modifier(Modifier::BOLD));

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(16),
            Constraint::Length(16),
        ],
    )
    .header(header)
    .column_spacing(1)
    .block(block)
    .highlight_style(SELECTED_STYLE);

    app.monthly_transfers_table_state
        .select(Some(app.selected_transfer_month));
    frame.render_stateful_widget(table, area, &mut app.monthly_transfers_table_state);
}

fn draw_transfer_month(frame: &mut Frame, app: &mut App, area: Rect) {
    let month = app
        .monthly_transfers
        .get(app.selected_transfer_month)
        .map(|m| m.month.as_str())
        .unwrap_or("Transfers");
    let block = Block::default()
        .title(format!("Transfers \u{2014} {month}"))
        .borders(Borders::ALL);

    if app.transfer_month_entries.is_empty() {
        frame.render_widget(
            Paragraph::new("No transfers for this month.").block(block),
            area,
        );
        return;
    }

    let rows = app
        .transfer_month_entries
        .iter()
        .map(|entry| {
            let amount = crate::format_amount_minor(entry.amount_minor, &entry.currency);
            let from = crate::app::resolve_transfer_leg_name(
                entry.out_account_id,
                entry.out_sort.as_deref(),
                entry.out_account.as_deref(),
                &app.accounts,
                &app.household_accounts,
            );
            let to = crate::app::resolve_transfer_leg_name(
                entry.in_account_id,
                entry.in_sort.as_deref(),
                entry.in_account.as_deref(),
                &app.accounts,
                &app.household_accounts,
            );
            let description = entry
                .out_description
                .as_deref()
                .or(entry.in_description.as_deref())
                .unwrap_or("");
            Row::new(vec![
                Cell::from(entry.occurred_on.clone()),
                Cell::from(Line::from(amount).alignment(Alignment::Right)),
                Cell::from(description.to_string()),
                Cell::from(from),
                Cell::from(to),
            ])
        })
        .collect::<Vec<_>>();

    let header = Row::new(vec![
        Cell::from("Date"),
        Cell::from(Line::from("Amount").alignment(Alignment::Right)),
        Cell::from("Description"),
        Cell::from("From"),
        Cell::from("To"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Fill(1),
            Constraint::Length(20),
            Constraint::Length(22),
        ],
    )
    .header(header)
    .column_spacing(1)
    .block(block)
    .highlight_style(SELECTED_STYLE);

    app.transfer_month_table_state
        .select(Some(app.selected_transfer_entry));
    frame.render_stateful_widget(table, area, &mut app.transfer_month_table_state);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    const BINDINGS: &[(&str, &str)] = &[
        ("j / k, \u{2193} / \u{2191}", "Move selection"),
        ("Ctrl-d / Ctrl-u", "Page down / up"),
        ("gg / G", "Jump to top / bottom"),
        ("Enter", "Open selected account, or drill into a month"),
        ("<space>a", "Jump to Accounts screen"),
        ("<space>s", "Jump to Monthly Spend screen"),
        ("<space>t", "Jump to Monthly Transfers screen"),
        ("n", "Edit note on selected spend entry (spend drill-down)"),
        (
            "i",
            "Show both legs of selected transfer (transfers drill-down)",
        ),
        ("y", "Copy selected row to the clipboard"),
        ("Esc / q", "Back (or quit from the accounts screen)"),
        ("?", "Toggle this help screen"),
        ("Ctrl-c", "Quit"),
    ];
    let lines: Vec<ratatui::text::Line> = BINDINGS
        .iter()
        .map(|(key, action)| ratatui::text::Line::from(format!("{key:<20} {action}")))
        .collect();

    let paragraph =
        Paragraph::new(lines).block(Block::default().title("Help").borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let paragraph = Paragraph::new(app.status.as_str()).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}
