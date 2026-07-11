use crate::app::{App, Screen};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.area());

    match app.screen {
        Screen::Accounts => draw_accounts(frame, app, chunks[0]),
        Screen::Transactions => draw_transactions(frame, app, chunks[0]),
    }
    draw_status(frame, app, chunks[1]);
}

fn draw_accounts(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if app.accounts.is_empty() {
        vec![ListItem::new(
            "No accounts yet. Import a statement to get started.",
        )]
    } else {
        app.accounts
            .iter()
            .enumerate()
            .map(|(i, account)| {
                let label = format!(
                    "{}  ({}{})",
                    account.name,
                    account.account_type.as_str(),
                    account
                        .institution
                        .as_ref()
                        .map(|inst| format!(", {inst}"))
                        .unwrap_or_default()
                );
                style_item(label, i == app.selected_account)
            })
            .collect()
    };

    let list = List::new(items).block(Block::default().title("Accounts").borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn draw_transactions(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if app.transactions.is_empty() {
        vec![ListItem::new("No transactions for this account yet.")]
    } else {
        app.transactions
            .iter()
            .enumerate()
            .map(|(i, tx)| {
                let amount = tx.amount_minor as f64 / 100.0;
                let label = format!(
                    "{}  {:>10.2} {}  {}",
                    tx.posted_at, amount, tx.currency, tx.description
                );
                style_item(label, i == app.selected_transaction)
            })
            .collect()
    };

    let account_name = app
        .accounts
        .get(app.selected_account)
        .map(|a| a.name.as_str())
        .unwrap_or("Transactions");
    let list = List::new(items).block(
        Block::default()
            .title(format!("Transactions \u{2014} {account_name}"))
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn style_item(label: String, selected: bool) -> ListItem<'static> {
    if selected {
        ListItem::new(Line::from(Span::styled(
            label,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )))
    } else {
        ListItem::new(label)
    }
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let paragraph = Paragraph::new(app.status.as_str()).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}
