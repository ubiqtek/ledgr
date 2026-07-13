use crate::config::{Config, HouseholdAccountRef};
use crate::db::{AccountStatus, Db};
use crate::derive;
use crate::model::{
    MonthlySpend, MonthlyTransfer, SpendEntryWithAccount, Transaction, TransferEntry,
};
use ratatui::widgets::TableState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Accounts,
    Transactions,
    MonthlyGap,
    MonthSpend,
    MonthlyTransfers,
    TransferMonth,
    Help,
}

pub struct App {
    pub db: Db,
    pub screen: Screen,
    /// Screens to return to on `back()`, most recent last — pushed by
    /// `navigate_to` whenever the screen actually changes.
    nav_stack: Vec<Screen>,
    pub accounts: Vec<AccountStatus>,
    pub selected_account: usize,
    /// Kept across frames (rather than rebuilt fresh each render) so its
    /// `offset` persists — that's what lets ratatui's Table scroll by the
    /// minimal amount needed to keep the selection in view instead of
    /// recentring the viewport on every keypress.
    pub accounts_table_state: TableState,
    pub transactions: Vec<Transaction>,
    pub selected_transaction: usize,
    pub transactions_table_state: TableState,
    pub monthly_spend: Vec<MonthlySpend>,
    pub selected_month: usize,
    pub monthly_gap_table_state: TableState,
    pub month_spend_entries: Vec<SpendEntryWithAccount>,
    pub selected_spend_entry: usize,
    pub month_spend_table_state: TableState,
    /// Reference household accounts (e.g. a partner's — see ADR 0008),
    /// loaded once from `config.toml` at startup, same lifecycle as the
    /// account-name overrides applied in `App::new`, so
    /// `open_monthly_transfers` doesn't need to re-read the config file on
    /// every call.
    pub household_accounts: Vec<HouseholdAccountRef>,
    /// The full flat list behind the Monthly Transfers screen, cached here
    /// so a future per-month drill-down can filter it in memory rather than
    /// re-running `derive::find_internal_transfers`.
    pub transfer_entries: Vec<TransferEntry>,
    pub monthly_transfers: Vec<MonthlyTransfer>,
    pub selected_transfer_month: usize,
    pub monthly_transfers_table_state: TableState,
    /// The selected month's transfer entries, filtered in memory from
    /// `transfer_entries` by `open_selected_transfer_month` — the per-month
    /// audit drill-down for the Monthly Transfers screen.
    pub transfer_month_entries: Vec<TransferEntry>,
    pub selected_transfer_entry: usize,
    pub transfer_month_table_state: TableState,
    /// `Some(buffer)` while editing the selected spend entry's note (`n` on
    /// `Screen::MonthSpend`) — its presence is what routes key events to
    /// text editing instead of navigation, see `main.rs`'s event loop.
    pub note_edit: Option<String>,
    /// `Some` while the "both legs of this transfer" popup is open (`i` on
    /// `Screen::TransferMonth`) — its presence routes key events to
    /// dismissing the popup instead of navigation, same pattern as
    /// `note_edit`.
    pub transfer_detail: Option<TransferDetail>,
    pub should_quit: bool,
    pub status: String,
}

/// Both sides of a transfer, shown in the popup opened by `i` on
/// `Screen::TransferMonth`. `counterpart` is `None` when the other leg isn't
/// a recorded transaction (e.g. the transfer's counterpart is a Reference
/// Household Account, which by definition has no imported transactions) or
/// couldn't be matched.
pub struct TransferDetail {
    pub own: Transaction,
    pub own_account_name: String,
    pub counterpart: Option<Transaction>,
    pub counterpart_label: String,
}

impl App {
    pub fn new(db: Db) -> anyhow::Result<Self> {
        let mut accounts = db.account_statuses()?;
        let config = Config::load_or_init(&Config::default_path()?)?;
        config.apply_account_name_overrides(accounts.iter_mut().map(|s| &mut s.account));
        Ok(Self {
            db,
            screen: Screen::Accounts,
            nav_stack: Vec::new(),
            accounts,
            selected_account: 0,
            accounts_table_state: TableState::default(),
            transactions: Vec::new(),
            selected_transaction: 0,
            transactions_table_state: TableState::default(),
            monthly_spend: Vec::new(),
            selected_month: 0,
            monthly_gap_table_state: TableState::default(),
            month_spend_entries: Vec::new(),
            selected_spend_entry: 0,
            month_spend_table_state: TableState::default(),
            household_accounts: config.household_accounts,
            transfer_entries: Vec::new(),
            monthly_transfers: Vec::new(),
            selected_transfer_month: 0,
            monthly_transfers_table_state: TableState::default(),
            transfer_month_entries: Vec::new(),
            selected_transfer_entry: 0,
            transfer_month_table_state: TableState::default(),
            note_edit: None,
            transfer_detail: None,
            should_quit: false,
            status: "j/k move, enter open, space leader, ctrl-d/u page, ? help, esc back, q quit"
                .into(),
        })
    }

    pub fn open_selected_account(&mut self) -> anyhow::Result<()> {
        let Some(status) = self.accounts.get(self.selected_account) else {
            return Ok(());
        };
        self.transactions = self.db.list_transactions_for_account(status.account.id)?;
        self.selected_transaction = 0;
        self.navigate_to(Screen::Transactions);
        Ok(())
    }

    pub fn open_monthly_gap(&mut self) -> anyhow::Result<()> {
        self.monthly_spend = self.db.monthly_spend_totals()?;
        self.selected_month = 0;
        self.navigate_to(Screen::MonthlyGap);
        Ok(())
    }

    pub fn open_selected_month(&mut self) -> anyhow::Result<()> {
        let Some(month) = self.monthly_spend.get(self.selected_month) else {
            return Ok(());
        };
        self.month_spend_entries = self.db.spend_entries_for_month(&month.month)?;
        self.selected_spend_entry = 0;
        self.navigate_to(Screen::MonthSpend);
        Ok(())
    }

    /// Opens the Monthly Transfers audit screen: re-derives the full set of
    /// internal transfers (`derive::find_internal_transfers` — read-only, no
    /// DB writes), caches the flat list on `self.transfer_entries` for a
    /// future per-month drill-down to filter in memory, and groups it into
    /// `self.monthly_transfers` for display.
    pub fn open_monthly_transfers(&mut self) -> anyhow::Result<()> {
        self.transfer_entries =
            derive::find_internal_transfers(&self.db, &self.household_accounts)?;
        self.monthly_transfers = group_monthly_transfers(&self.transfer_entries);
        self.selected_transfer_month = 0;
        self.navigate_to(Screen::MonthlyTransfers);
        Ok(())
    }

    /// Opens the per-month audit drill-down for the Monthly Transfers
    /// screen: filters the already-cached `self.transfer_entries` down to
    /// the selected month, in memory — no DB access or re-derivation, that's
    /// the whole point of caching the flat list in `open_monthly_transfers`.
    pub fn open_selected_transfer_month(&mut self) -> anyhow::Result<()> {
        let Some(month) = self.monthly_transfers.get(self.selected_transfer_month) else {
            return Ok(());
        };
        self.transfer_month_entries = self
            .transfer_entries
            .iter()
            .filter(|entry| entry.posted_at.get(..7) == Some(month.month.as_str()))
            .cloned()
            .collect();
        self.selected_transfer_entry = 0;
        self.navigate_to(Screen::TransferMonth);
        Ok(())
    }

    /// Opens the "both legs of this transfer" popup for the selected entry
    /// on `Screen::TransferMonth`. The counterpart leg isn't cached anywhere
    /// (`find_internal_transfers` only ever sees one side), so this looks it
    /// up on demand via the same `find_transfer_counterpart` query
    /// derivation uses — read-only, no DB writes.
    pub fn show_transfer_detail(&mut self) -> anyhow::Result<()> {
        if self.screen != Screen::TransferMonth {
            return Ok(());
        }
        let Some(entry) = self
            .transfer_month_entries
            .get(self.selected_transfer_entry)
        else {
            return Ok(());
        };
        let Some(own) = self.db.get_transaction(entry.transaction_id)? else {
            return Ok(());
        };
        let own_account = self
            .accounts
            .iter()
            .find(|s| s.account.id == entry.account_id);
        let own_account_name = own_account
            .map(|s| s.account.name.clone())
            .unwrap_or_else(|| "?".to_string());

        let counterpart = match (
            own_account.and_then(|s| s.account.sort_code.as_deref()),
            own_account.and_then(|s| s.account.account_number.as_deref()),
            entry.counterpart_sort.as_deref(),
            entry.counterpart_account.as_deref(),
        ) {
            (
                Some(own_sort),
                Some(own_number),
                Some(counterpart_sort),
                Some(counterpart_number),
            ) => self
                .db
                .find_transfer_counterpart(
                    entry.transaction_id,
                    own_sort,
                    own_number,
                    counterpart_sort,
                    counterpart_number,
                    entry.amount_minor,
                    &entry.posted_at,
                )?
                .and_then(|id| self.db.get_transaction(id).ok().flatten()),
            _ => None,
        };
        let counterpart_label =
            resolve_counterparty(entry, &self.accounts, &self.household_accounts);

        self.transfer_detail = Some(TransferDetail {
            own,
            own_account_name,
            counterpart,
            counterpart_label,
        });
        Ok(())
    }

    pub fn close_transfer_detail(&mut self) {
        self.transfer_detail = None;
    }

    /// Opens the note editor for the selected spend entry on
    /// `Screen::MonthSpend`, pre-filled with its existing note (if any).
    pub fn start_editing_note(&mut self) {
        if self.screen != Screen::MonthSpend {
            return;
        }
        let Some(row) = self.month_spend_entries.get(self.selected_spend_entry) else {
            return;
        };
        self.note_edit = Some(row.entry.note.clone().unwrap_or_default());
    }

    /// Discards the in-progress note edit without saving.
    pub fn cancel_editing_note(&mut self) {
        self.note_edit = None;
    }

    /// Saves the in-progress note edit (an empty buffer clears the note) and
    /// closes the editor. Deliberately updates the in-memory row too, not
    /// just the database, so the change is visible immediately without
    /// re-querying.
    pub fn commit_note(&mut self) -> anyhow::Result<()> {
        let Some(buffer) = self.note_edit.take() else {
            return Ok(());
        };
        let Some(row) = self.month_spend_entries.get_mut(self.selected_spend_entry) else {
            return Ok(());
        };
        let trimmed = buffer.trim();
        let note = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
        self.db
            .set_spend_entry_note(row.entry.id, note.as_deref())?;
        row.entry.note = note;
        Ok(())
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = match self.screen {
            Screen::Accounts => self.accounts.len(),
            Screen::Transactions => self.transactions.len(),
            Screen::MonthlyGap => self.monthly_spend.len(),
            Screen::MonthSpend => self.month_spend_entries.len(),
            Screen::MonthlyTransfers => self.monthly_transfers.len(),
            Screen::TransferMonth => self.transfer_month_entries.len(),
            Screen::Help => return,
        };
        if len == 0 {
            return;
        }
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
            Screen::MonthlyGap => &mut self.selected_month,
            Screen::MonthSpend => &mut self.selected_spend_entry,
            Screen::MonthlyTransfers => &mut self.selected_transfer_month,
            Screen::TransferMonth => &mut self.selected_transfer_entry,
            Screen::Help => return,
        };
        let next = *selected as i32 + delta;
        *selected = next.clamp(0, len as i32 - 1) as usize;
    }

    /// Jumps to the first row of the current list, like nvim's `gg`.
    pub fn select_first(&mut self) {
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
            Screen::MonthlyGap => &mut self.selected_month,
            Screen::MonthSpend => &mut self.selected_spend_entry,
            Screen::MonthlyTransfers => &mut self.selected_transfer_month,
            Screen::TransferMonth => &mut self.selected_transfer_entry,
            Screen::Help => return,
        };
        *selected = 0;
    }

    /// Jumps to the last row of the current list, like nvim's `G`.
    pub fn select_last(&mut self) {
        let len = match self.screen {
            Screen::Accounts => self.accounts.len(),
            Screen::Transactions => self.transactions.len(),
            Screen::MonthlyGap => self.monthly_spend.len(),
            Screen::MonthSpend => self.month_spend_entries.len(),
            Screen::MonthlyTransfers => self.monthly_transfers.len(),
            Screen::TransferMonth => self.transfer_month_entries.len(),
            Screen::Help => return,
        };
        if len == 0 {
            return;
        }
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
            Screen::MonthlyGap => &mut self.selected_month,
            Screen::MonthSpend => &mut self.selected_spend_entry,
            Screen::MonthlyTransfers => &mut self.selected_transfer_month,
            Screen::TransferMonth => &mut self.selected_transfer_entry,
            Screen::Help => return,
        };
        *selected = len - 1;
    }

    /// Tab-separated text for the currently selected row, for `y` (copy to
    /// clipboard) — one string per screen's visible columns, mirroring
    /// `ui.rs`'s row rendering for that screen. `None` when the current
    /// screen has no row list (e.g. `Screen::Help`) or the list is empty.
    pub fn selected_row_text(&self) -> Option<String> {
        match self.screen {
            Screen::Accounts => {
                let status = self.accounts.get(self.selected_account)?;
                let account = &status.account;
                let balance = status
                    .balance_minor
                    .map(|minor| crate::format_amount_minor(minor, &account.currency))
                    .unwrap_or_else(|| "unknown".to_string());
                Some(format!(
                    "{}\t{}\t{}\t{}",
                    account.name,
                    account.account_type.as_str(),
                    account.institution.clone().unwrap_or_default(),
                    balance
                ))
            }
            Screen::Transactions => {
                let txn = self.transactions.get(self.selected_transaction)?;
                Some(format!(
                    "{}\t{}\t{}",
                    txn.posted_at,
                    crate::format_amount_minor(txn.amount_minor, &txn.currency),
                    txn.description
                ))
            }
            Screen::MonthlyGap => {
                let month = self.monthly_spend.get(self.selected_month)?;
                Some(format!(
                    "{}\t{}",
                    month.month,
                    crate::format_amount_minor(month.spend_minor.abs(), "GBP")
                ))
            }
            Screen::MonthSpend => {
                let row = self.month_spend_entries.get(self.selected_spend_entry)?;
                let entry = &row.entry;
                let account_name = self
                    .accounts
                    .iter()
                    .find(|s| s.account.id == row.account_id)
                    .map(|s| s.account.name.as_str())
                    .unwrap_or("?");
                let rule = entry
                    .rule_name
                    .clone()
                    .unwrap_or_else(|| entry.classified_by.as_str().to_string());
                Some(format!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    entry.occurred_on,
                    crate::format_amount_minor(entry.amount_minor, &entry.currency),
                    entry.counterparty.clone().unwrap_or_default(),
                    entry.description,
                    rule,
                    account_name
                ))
            }
            Screen::MonthlyTransfers => {
                let month = self.monthly_transfers.get(self.selected_transfer_month)?;
                Some(format!(
                    "{}\t{}\t{}",
                    month.month,
                    crate::format_amount_minor(month.transferred_out_minor.abs(), "GBP"),
                    crate::format_amount_minor(month.transferred_in_minor, "GBP")
                ))
            }
            Screen::TransferMonth => {
                let entry = self
                    .transfer_month_entries
                    .get(self.selected_transfer_entry)?;
                let account_name = self
                    .accounts
                    .iter()
                    .find(|s| s.account.id == entry.account_id)
                    .map(|s| s.account.name.as_str())
                    .unwrap_or("?");
                let counterparty =
                    resolve_counterparty(entry, &self.accounts, &self.household_accounts);
                let (from, to) = if entry.amount_minor < 0 {
                    (account_name.to_string(), counterparty)
                } else {
                    (counterparty, account_name.to_string())
                };
                Some(format!(
                    "{}\t{}\t{}\t{}\t{}",
                    entry.posted_at,
                    crate::format_amount_minor(entry.amount_minor.abs(), &entry.currency),
                    entry.description,
                    from,
                    to
                ))
            }
            Screen::Help => None,
        }
    }

    /// Switches to `screen`, first pushing the current screen onto the
    /// navigation-history stack so `back()` can return to it — a no-op if
    /// `screen` is already the current screen (no duplicate push).
    pub fn navigate_to(&mut self, screen: Screen) {
        if screen == self.screen {
            return;
        }
        self.nav_stack.push(self.screen);
        self.screen = screen;
    }

    /// Pops the navigation-history stack and returns to whatever screen was
    /// there, falling back to `Screen::Accounts` if the stack is empty.
    pub fn back(&mut self) {
        self.screen = self.nav_stack.pop().unwrap_or(Screen::Accounts);
    }

    /// Shows the help screen, or leaves it (returning to whichever screen
    /// was open before) if it's already showing.
    pub fn toggle_help(&mut self) {
        if self.screen == Screen::Help {
            self.back();
        } else {
            self.navigate_to(Screen::Help);
        }
    }
}

/// Resolves a transfer entry's counterpart to a display name for the
/// per-month audit drill-down: a tracked account first (e.g. "Adventure
/// Fund"), then a reference household account (e.g. "Joint Annual
/// Expense"), falling back to the raw sort code/account number digits if
/// neither matches — that fallback is itself a signal something's off,
/// similar in spirit to how a `"fallback"` rule name flags a low-confidence
/// spend classification elsewhere, so it's shown plainly rather than hidden.
pub(crate) fn resolve_counterparty(
    entry: &TransferEntry,
    accounts: &[AccountStatus],
    household_accounts: &[HouseholdAccountRef],
) -> String {
    let (Some(sort), Some(account_number)) = (&entry.counterpart_sort, &entry.counterpart_account)
    else {
        return "?".to_string();
    };

    // Barclays truncates the account-number digits when a long label pushes
    // the NAME field past its length limit (e.g. "SHARED BILLS ACCO 208794
    // 231650", the real account being `...23165086`) — `account_number` may
    // therefore be a prefix rather than the full number. Matched the same
    // way `derive::household_contains` matches it for classification, so
    // this resolves to the same account classification already settled on.
    if let Some(status) = accounts.iter().find(|s| {
        s.account.sort_code.as_deref() == Some(sort.as_str())
            && s
                .account
                .account_number
                .as_deref()
                .is_some_and(|full| full.starts_with(account_number.as_str()))
    }) {
        return status.account.name.clone();
    }

    if let Some(household) = household_accounts.iter().find(|h| {
        &h.sort_code == sort && h.account_number.starts_with(account_number.as_str())
    }) {
        return household
            .label
            .clone()
            .unwrap_or_else(|| format!("{sort} {account_number}"));
    }

    format!("{sort} {account_number}")
}

/// Groups transfer entries into per-month totals for the Monthly Transfers
/// screen, newest month first (same ordering convention as
/// `Db::monthly_spend_totals`). Out and in are kept separate — not netted —
/// per `MonthlyTransfer`'s doc comment.
fn group_monthly_transfers(entries: &[TransferEntry]) -> Vec<MonthlyTransfer> {
    let mut totals: std::collections::BTreeMap<String, (i64, i64)> =
        std::collections::BTreeMap::new();
    for entry in entries {
        let month = entry.posted_at.get(..7).unwrap_or(&entry.posted_at);
        let (out, inn) = totals.entry(month.to_string()).or_default();
        if entry.amount_minor < 0 {
            *out += entry.amount_minor;
        } else {
            *inn += entry.amount_minor;
        }
    }
    totals
        .into_iter()
        .rev()
        .map(
            |(month, (transferred_out_minor, transferred_in_minor))| MonthlyTransfer {
                month,
                transferred_out_minor,
                transferred_in_minor,
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(posted_at: &str, amount_minor: i64) -> TransferEntry {
        TransferEntry {
            transaction_id: 1,
            account_id: 1,
            posted_at: posted_at.into(),
            amount_minor,
            currency: "GBP".into(),
            description: "TEST".into(),
            counterpart_sort: None,
            counterpart_account: None,
        }
    }

    #[test]
    fn group_monthly_transfers_keeps_in_and_out_separate_newest_month_first() {
        let entries = vec![
            entry("2026-06-05", -12000),
            entry("2026-06-10", 5000),
            entry("2026-07-01", 3000),
        ];

        let months = group_monthly_transfers(&entries);

        assert_eq!(months.len(), 2);
        assert_eq!(months[0].month, "2026-07");
        assert_eq!(months[0].transferred_out_minor, 0);
        assert_eq!(months[0].transferred_in_minor, 3000);
        assert_eq!(months[1].month, "2026-06");
        assert_eq!(months[1].transferred_out_minor, -12000);
        assert_eq!(months[1].transferred_in_minor, 5000);
    }

    fn transfer_entry_to(sort: &str, account_number: &str) -> TransferEntry {
        let mut e = entry("2026-07-01", -1000);
        e.counterpart_sort = Some(sort.into());
        e.counterpart_account = Some(account_number.into());
        e
    }

    fn account_status(name: &str, sort: &str, account_number: &str) -> AccountStatus {
        AccountStatus {
            account: crate::model::Account {
                id: 1,
                name: name.into(),
                institution: None,
                account_type: crate::model::AccountType::Savings,
                currency: "GBP".into(),
                sort_code: Some(sort.into()),
                account_number: Some(account_number.into()),
            },
            transaction_count: 0,
            balance_minor: None,
            balance_as_of: None,
            earliest_transaction: None,
            latest_transaction: None,
            last_imported_at: None,
            card_last4: None,
        }
    }

    fn household_ref(sort: &str, account_number: &str, label: &str) -> HouseholdAccountRef {
        HouseholdAccountRef {
            sort_code: sort.into(),
            account_number: account_number.into(),
            label: Some(label.into()),
            name: None,
        }
    }

    #[test]
    fn resolve_counterparty_matches_tracked_account_first() {
        let entry = transfer_entry_to("209912", "12345678");
        let accounts = vec![account_status("Adventure Fund", "209912", "12345678")];
        let household = vec![household_ref("209912", "12345678", "Should not be used")];

        assert_eq!(
            resolve_counterparty(&entry, &accounts, &household),
            "Adventure Fund"
        );
    }

    #[test]
    fn resolve_counterparty_falls_back_to_household_reference_account() {
        let entry = transfer_entry_to("609934", "99998888");
        let accounts = vec![account_status("Current Account", "111111", "22222222")];
        let household = vec![household_ref("609934", "99998888", "Joint Annual Expense")];

        assert_eq!(
            resolve_counterparty(&entry, &accounts, &household),
            "Joint Annual Expense"
        );
    }

    #[test]
    fn resolve_counterparty_falls_back_to_raw_digits_when_unresolved() {
        let entry = transfer_entry_to("609934", "99998888");
        let accounts = vec![account_status("Current Account", "111111", "22222222")];
        let household = vec![household_ref("111111", "22222222", "Unrelated")];

        assert_eq!(
            resolve_counterparty(&entry, &accounts, &household),
            "609934 99998888"
        );
    }
}
