use crate::config::{Config, HouseholdAccountRef, RegisteredPersonRef};
use crate::db::{AccountStatus, Db};
use crate::derive;
use crate::model::{
    Id, IncomeEntryWithAccount, MonthlyGap, MonthlyIncome, MonthlySpend, MonthlyTransfer,
    SpendEntryWithAccount, Transaction, TransferEntry,
};
use ratatui::widgets::TableState;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Accounts,
    Transactions,
    MonthlySpend,
    SpendMonth,
    MonthlyIncome,
    IncomeMonth,
    MonthlyTransfers,
    TransferMonth,
    Gap,
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
    pub monthly_spend_table_state: TableState,
    pub spend_month_entries: Vec<SpendEntryWithAccount>,
    pub selected_spend_entry: usize,
    pub spend_month_table_state: TableState,
    pub monthly_income: Vec<MonthlyIncome>,
    pub selected_income_month: usize,
    pub monthly_income_table_state: TableState,
    pub income_month_entries: Vec<IncomeEntryWithAccount>,
    pub selected_income_entry: usize,
    pub income_month_table_state: TableState,
    /// `Screen::Gap`'s month-by-month rows. No selection index or
    /// `TableState` — unlike the other monthly screens, the Gap screen has
    /// no drill-down to select into, just a summary report.
    pub monthly_gap: Vec<MonthlyGap>,
    /// Total cash (`Current`/`Savings` accounts) balance at the start of
    /// the calendar year and at the end of the last complete month (same
    /// cutoff as `monthly_gap`, not literally "now" — see `open_gap`) —
    /// lets the Gap screen's summary show where a YTD spend-exceeds-income
    /// shortfall actually came from (e.g. drawn down from savings) rather
    /// than leaving it unexplained.
    pub cash_at_year_start: i64,
    pub cash_now: i64,
    /// Reference household accounts (e.g. a partner's — see ADR 0008),
    /// loaded once from `config.toml` at startup, same lifecycle as the
    /// account-name overrides applied in `App::new`, so
    /// `open_monthly_transfers` doesn't need to re-read the config file on
    /// every call.
    pub household_accounts: Vec<HouseholdAccountRef>,
    pub monthly_transfers: Vec<MonthlyTransfer>,
    pub selected_transfer_month: usize,
    pub monthly_transfers_table_state: TableState,
    /// The selected month's transfer entries, queried directly from
    /// `transfer_entries` by `open_selected_transfer_month` — the per-month
    /// audit drill-down for the Monthly Transfers screen.
    pub transfer_month_entries: Vec<TransferEntry>,
    pub selected_transfer_entry: usize,
    pub transfer_month_table_state: TableState,
    /// `Some(buffer)` while editing the selected spend entry's note (`n` on
    /// `Screen::SpendMonth`) — its presence is what routes key events to
    /// text editing instead of navigation, see `main.rs`'s event loop.
    pub note_edit: Option<String>,
    /// `Some` while the "both legs of this transfer" popup is open (`i` on
    /// `Screen::TransferMonth`) — its presence routes key events to
    /// dismissing the popup instead of navigation, same pattern as
    /// `note_edit`.
    pub transfer_detail: Option<TransferDetail>,
    /// `Some` while the "source transaction" popup is open (`i` on
    /// `Screen::IncomeMonth`) — shows the raw transaction behind the
    /// selected income entry for verification. Same routing pattern as
    /// `transfer_detail`.
    pub income_detail: Option<Transaction>,
    /// `Some` while the "add reference" form is open (`a` on
    /// `Screen::IncomeMonth`) — registers the selected entry's sender as a
    /// Registered Person so future (and this) payment(s) from them classify
    /// as a spend-ledger reimbursement instead of income. Same routing
    /// pattern as `note_edit`.
    pub person_form: Option<PersonForm>,
    pub should_quit: bool,
    pub status: String,
    config: Config,
    config_path: PathBuf,
}

/// Which field of `PersonForm` is currently being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonFormField {
    Name,
    Label,
    FullName,
}

impl PersonFormField {
    fn next(self) -> Self {
        match self {
            PersonFormField::Name => PersonFormField::Label,
            PersonFormField::Label => PersonFormField::FullName,
            PersonFormField::FullName => PersonFormField::FullName,
        }
    }

    fn previous(self) -> Self {
        match self {
            PersonFormField::Name => PersonFormField::Name,
            PersonFormField::Label => PersonFormField::Name,
            PersonFormField::FullName => PersonFormField::Label,
        }
    }
}

/// The "add reference" form's in-progress state, opened by `a` on
/// `Screen::IncomeMonth`. `income_entry_id` identifies the entry to remove
/// from the income ledger (so it can be re-derived as a reimbursement) once
/// the form is submitted.
pub struct PersonForm {
    pub name: String,
    pub label: String,
    pub full_name: String,
    pub field: PersonFormField,
    income_entry_id: Id,
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
        let config_path = Config::default_path()?;
        let config = Config::load_or_init(&config_path)?;
        config.apply_account_name_overrides(accounts.iter_mut().map(|s| &mut s.account));
        let (monthly_gap, cash_at_year_start, cash_now) = load_gap_data(&db)?;
        Ok(Self {
            db,
            // The Gap screen is the first thing the user wants to see on
            // launch — a household finance overview, not an account list.
            screen: Screen::Gap,
            nav_stack: Vec::new(),
            accounts,
            selected_account: 0,
            accounts_table_state: TableState::default(),
            transactions: Vec::new(),
            selected_transaction: 0,
            transactions_table_state: TableState::default(),
            monthly_spend: Vec::new(),
            selected_month: 0,
            monthly_spend_table_state: TableState::default(),
            spend_month_entries: Vec::new(),
            selected_spend_entry: 0,
            spend_month_table_state: TableState::default(),
            monthly_income: Vec::new(),
            selected_income_month: 0,
            monthly_income_table_state: TableState::default(),
            income_month_entries: Vec::new(),
            selected_income_entry: 0,
            income_month_table_state: TableState::default(),
            monthly_gap,
            cash_at_year_start,
            cash_now,
            household_accounts: config.household_accounts.clone(),
            monthly_transfers: Vec::new(),
            selected_transfer_month: 0,
            monthly_transfers_table_state: TableState::default(),
            transfer_month_entries: Vec::new(),
            selected_transfer_entry: 0,
            transfer_month_table_state: TableState::default(),
            note_edit: None,
            transfer_detail: None,
            income_detail: None,
            person_form: None,
            should_quit: false,
            status: "j/k move, enter open, space leader, ctrl-d/u page, ? help, esc back, q quit"
                .into(),
            config,
            config_path,
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

    pub fn open_monthly_spend(&mut self) -> anyhow::Result<()> {
        self.monthly_spend = self.db.monthly_spend_totals()?;
        // Rows are earliest-first; start the selection on the most recent
        // month (the last row) rather than January.
        self.selected_month = self.monthly_spend.len().saturating_sub(1);
        self.navigate_to(Screen::MonthlySpend);
        Ok(())
    }

    pub fn open_selected_month(&mut self) -> anyhow::Result<()> {
        let Some(month) = self.monthly_spend.get(self.selected_month) else {
            return Ok(());
        };
        self.spend_month_entries = self.db.spend_entries_for_month(&month.month)?;
        self.selected_spend_entry = 0;
        self.navigate_to(Screen::SpendMonth);
        Ok(())
    }

    pub fn open_monthly_income(&mut self) -> anyhow::Result<()> {
        self.monthly_income = self.db.monthly_income_totals()?;
        // Rows are earliest-first; start the selection on the most recent
        // month (the last row) rather than January.
        self.selected_income_month = self.monthly_income.len().saturating_sub(1);
        self.navigate_to(Screen::MonthlyIncome);
        Ok(())
    }

    pub fn open_selected_income_month(&mut self) -> anyhow::Result<()> {
        let Some(month) = self.monthly_income.get(self.selected_income_month) else {
            return Ok(());
        };
        self.income_month_entries = self.db.income_entries_for_month(&month.month)?;
        self.selected_income_entry = 0;
        self.navigate_to(Screen::IncomeMonth);
        Ok(())
    }

    /// Opens `Screen::Gap` — the YTD summary + month-by-month report
    /// combining the spend and income ledgers. No selection state to reset,
    /// unlike the other monthly screens: this screen has no drill-down.
    /// The whole report (summary and month-by-month alike) only covers
    /// complete calendar months — the current, still-in-progress month is
    /// dropped entirely, since its partial spend with no matching income
    /// yet (e.g. salary not yet paid) would misrepresent both.
    pub fn open_gap(&mut self) -> anyhow::Result<()> {
        let (monthly_gap, cash_at_year_start, cash_now) = load_gap_data(&self.db)?;
        self.monthly_gap = monthly_gap;
        self.cash_at_year_start = cash_at_year_start;
        self.cash_now = cash_now;
        self.navigate_to(Screen::Gap);
        Ok(())
    }

    /// Opens the "source transaction" popup for the selected entry on
    /// `Screen::IncomeMonth` — lets the user verify an income entry against
    /// the raw imported transaction it was derived from (e.g. to check the
    /// real description/amount behind a salary or cashback entry).
    pub fn show_income_detail(&mut self) -> anyhow::Result<()> {
        if self.screen != Screen::IncomeMonth {
            return Ok(());
        }
        let Some(entry) = self.income_month_entries.get(self.selected_income_entry) else {
            return Ok(());
        };
        self.income_detail = self.db.get_transaction(entry.transaction_id)?;
        Ok(())
    }

    pub fn close_income_detail(&mut self) {
        self.income_detail = None;
    }

    /// Opens the "add reference" form for the selected entry on
    /// `Screen::IncomeMonth` (`a`) — registers its sender as a Registered
    /// Person so this (and every future) payment from them is recognised as
    /// a spend-ledger reimbursement instead of falling through to the
    /// generic Income fallback rules. Pre-fills the Name field with a guess
    /// derived from the entry's own description, editable before submitting.
    pub fn start_adding_person(&mut self) {
        if self.screen != Screen::IncomeMonth {
            return;
        }
        let Some(entry) = self.income_month_entries.get(self.selected_income_entry) else {
            return;
        };
        self.person_form = Some(PersonForm {
            name: guess_person_name(&entry.entry.description),
            label: String::new(),
            full_name: String::new(),
            field: PersonFormField::Name,
            income_entry_id: entry.entry.id,
        });
    }

    /// Discards the in-progress "add reference" form without saving.
    pub fn cancel_person_form(&mut self) {
        self.person_form = None;
    }

    pub fn person_form_next_field(&mut self) {
        if let Some(form) = &mut self.person_form {
            form.field = form.field.next();
        }
    }

    pub fn person_form_previous_field(&mut self) {
        if let Some(form) = &mut self.person_form {
            form.field = form.field.previous();
        }
    }

    pub fn person_form_push_char(&mut self, c: char) {
        let Some(form) = &mut self.person_form else {
            return;
        };
        match form.field {
            PersonFormField::Name => form.name.push(c),
            PersonFormField::Label => form.label.push(c),
            PersonFormField::FullName => form.full_name.push(c),
        }
    }

    pub fn person_form_pop_char(&mut self) {
        let Some(form) = &mut self.person_form else {
            return;
        };
        match form.field {
            PersonFormField::Name => form.name.pop(),
            PersonFormField::Label => form.label.pop(),
            PersonFormField::FullName => form.full_name.pop(),
        };
    }

    /// `Enter` on the "add reference" form: advances to the next field, or
    /// submits (`commit_person_form`) when already on the last field
    /// (`FullName`) — standard multi-field form behaviour.
    pub fn person_form_enter(&mut self) -> anyhow::Result<()> {
        let Some(form) = &self.person_form else {
            return Ok(());
        };
        if form.field == PersonFormField::FullName {
            self.commit_person_form()
        } else {
            self.person_form_next_field();
            Ok(())
        }
    }

    /// Submits the "add reference" form: registers the new Registered Person
    /// in `config.toml`, removes the entry the form was opened from out of
    /// the income ledger (freeing its source transaction for
    /// re-derivation), then re-runs the derivation pass so it — now matching
    /// rule 1e (Registered Person) — lands in the spend ledger as a
    /// reimbursement instead. Refreshes the Monthly Income totals and the
    /// current month's drill-down in place so the change is visible without
    /// leaving the screen. A blank Name field cancels rather than saving.
    pub fn commit_person_form(&mut self) -> anyhow::Result<()> {
        let Some(form) = self.person_form.take() else {
            return Ok(());
        };
        let name = form.name.trim();
        if name.is_empty() {
            return Ok(());
        }
        let label = form.label.trim();
        let full_name = form.full_name.trim();
        self.config.add_registered_person(RegisteredPersonRef {
            name: name.to_string(),
            label: (!label.is_empty()).then(|| label.to_string()),
            full_name: (!full_name.is_empty()).then(|| full_name.to_string()),
        });
        self.config.save(&self.config_path)?;

        self.db.delete_income_entry(form.income_entry_id)?;
        derive::run_derivation(
            &self.db,
            &self.config.household_accounts,
            &self.config.income_sources,
            &self.config.registered_people,
            &self.config.reimbursement_sources,
        )?;

        let month = self
            .monthly_income
            .get(self.selected_income_month)
            .map(|m| m.month.clone());
        self.monthly_income = self.db.monthly_income_totals()?;
        if let Some(month) = month {
            if let Some(idx) = self.monthly_income.iter().position(|m| m.month == month) {
                self.selected_income_month = idx;
            }
            self.income_month_entries = self.db.income_entries_for_month(&month)?;
        } else {
            self.income_month_entries = Vec::new();
        }
        let len = self.income_month_entries.len();
        if self.selected_income_entry >= len {
            self.selected_income_entry = len.saturating_sub(1);
        }
        Ok(())
    }

    /// Opens the Monthly Transfers audit screen: queries the persisted
    /// transfer ledger's monthly aggregates directly (`Db::monthly_transfer_totals`)
    /// — no live re-derivation, per ADR 0009.
    pub fn open_monthly_transfers(&mut self) -> anyhow::Result<()> {
        self.monthly_transfers = self.db.monthly_transfer_totals()?;
        // Rows are earliest-first; start the selection on the most recent
        // month (the last row) rather than January.
        self.selected_transfer_month = self.monthly_transfers.len().saturating_sub(1);
        self.navigate_to(Screen::MonthlyTransfers);
        Ok(())
    }

    /// Opens the per-month audit drill-down for the Monthly Transfers
    /// screen: queries `transfer_entries` directly for the selected month
    /// (`Db::transfer_entries_for_month`) — no in-memory filtering of a
    /// cached full list, and no live re-derivation.
    pub fn open_selected_transfer_month(&mut self) -> anyhow::Result<()> {
        let Some(month) = self.monthly_transfers.get(self.selected_transfer_month) else {
            return Ok(());
        };
        self.transfer_month_entries = self.db.transfer_entries_for_month(&month.month)?;
        self.selected_transfer_entry = 0;
        self.navigate_to(Screen::TransferMonth);
        Ok(())
    }

    /// Opens the "both legs of this transfer" popup for the selected entry
    /// on `Screen::TransferMonth`. Both legs already live directly on the
    /// selected entry's `transfer_entries` row (`out_*`/`in_*`), so this is
    /// just reading it — a foreign-key follow via `Db::get_transaction`,
    /// not a live re-derivation. The outgoing side is shown as "own"
    /// (matching the drill-down's canonical row), falling back to the
    /// incoming side only for the rare row where the outgoing leg's
    /// transaction hasn't been found at all yet.
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

        let (
            own_transaction_id,
            own_account_id,
            own_sort,
            own_account,
            counterpart_transaction_id,
            counterpart_account_id,
            counterpart_sort,
            counterpart_account,
        ) = if entry.out_transaction_id.is_some() {
            (
                entry.out_transaction_id,
                entry.out_account_id,
                entry.out_sort.as_deref(),
                entry.out_account.as_deref(),
                entry.in_transaction_id,
                entry.in_account_id,
                entry.in_sort.as_deref(),
                entry.in_account.as_deref(),
            )
        } else {
            (
                entry.in_transaction_id,
                entry.in_account_id,
                entry.in_sort.as_deref(),
                entry.in_account.as_deref(),
                entry.out_transaction_id,
                entry.out_account_id,
                entry.out_sort.as_deref(),
                entry.out_account.as_deref(),
            )
        };

        let Some(own_transaction_id) = own_transaction_id else {
            return Ok(());
        };
        let Some(own) = self.db.get_transaction(own_transaction_id)? else {
            return Ok(());
        };
        let own_account_name = resolve_transfer_leg_name(
            own_account_id,
            own_sort,
            own_account,
            &self.accounts,
            &self.household_accounts,
        );

        let counterpart =
            counterpart_transaction_id.and_then(|id| self.db.get_transaction(id).ok().flatten());
        let counterpart_label = resolve_transfer_leg_name(
            counterpart_account_id,
            counterpart_sort,
            counterpart_account,
            &self.accounts,
            &self.household_accounts,
        );

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
    /// `Screen::SpendMonth`, pre-filled with its existing note (if any).
    pub fn start_editing_note(&mut self) {
        if self.screen != Screen::SpendMonth {
            return;
        }
        let Some(row) = self.spend_month_entries.get(self.selected_spend_entry) else {
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
        let Some(row) = self.spend_month_entries.get_mut(self.selected_spend_entry) else {
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
            Screen::MonthlySpend => self.monthly_spend.len(),
            Screen::SpendMonth => self.spend_month_entries.len(),
            Screen::MonthlyIncome => self.monthly_income.len(),
            Screen::IncomeMonth => self.income_month_entries.len(),
            Screen::MonthlyTransfers => self.monthly_transfers.len(),
            Screen::TransferMonth => self.transfer_month_entries.len(),
            Screen::Gap => return,
            Screen::Help => return,
        };
        if len == 0 {
            return;
        }
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
            Screen::MonthlySpend => &mut self.selected_month,
            Screen::SpendMonth => &mut self.selected_spend_entry,
            Screen::MonthlyIncome => &mut self.selected_income_month,
            Screen::IncomeMonth => &mut self.selected_income_entry,
            Screen::MonthlyTransfers => &mut self.selected_transfer_month,
            Screen::TransferMonth => &mut self.selected_transfer_entry,
            Screen::Gap => return,
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
            Screen::MonthlySpend => &mut self.selected_month,
            Screen::SpendMonth => &mut self.selected_spend_entry,
            Screen::MonthlyIncome => &mut self.selected_income_month,
            Screen::IncomeMonth => &mut self.selected_income_entry,
            Screen::MonthlyTransfers => &mut self.selected_transfer_month,
            Screen::TransferMonth => &mut self.selected_transfer_entry,
            Screen::Gap => return,
            Screen::Help => return,
        };
        *selected = 0;
    }

    /// Jumps to the last row of the current list, like nvim's `G`.
    pub fn select_last(&mut self) {
        let len = match self.screen {
            Screen::Accounts => self.accounts.len(),
            Screen::Transactions => self.transactions.len(),
            Screen::MonthlySpend => self.monthly_spend.len(),
            Screen::SpendMonth => self.spend_month_entries.len(),
            Screen::MonthlyIncome => self.monthly_income.len(),
            Screen::IncomeMonth => self.income_month_entries.len(),
            Screen::MonthlyTransfers => self.monthly_transfers.len(),
            Screen::TransferMonth => self.transfer_month_entries.len(),
            Screen::Gap => return,
            Screen::Help => return,
        };
        if len == 0 {
            return;
        }
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
            Screen::MonthlySpend => &mut self.selected_month,
            Screen::SpendMonth => &mut self.selected_spend_entry,
            Screen::MonthlyIncome => &mut self.selected_income_month,
            Screen::IncomeMonth => &mut self.selected_income_entry,
            Screen::MonthlyTransfers => &mut self.selected_transfer_month,
            Screen::TransferMonth => &mut self.selected_transfer_entry,
            Screen::Gap => return,
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
            Screen::MonthlySpend => {
                let month = self.monthly_spend.get(self.selected_month)?;
                Some(format!(
                    "{}\t{}",
                    month.month,
                    crate::format_amount_minor(month.spend_minor.abs(), "GBP")
                ))
            }
            Screen::SpendMonth => {
                let row = self.spend_month_entries.get(self.selected_spend_entry)?;
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
            Screen::MonthlyIncome => {
                let month = self.monthly_income.get(self.selected_income_month)?;
                Some(format!(
                    "{}\t{}",
                    month.month,
                    crate::format_amount_minor(month.income_minor, "GBP")
                ))
            }
            Screen::IncomeMonth => {
                let row = self.income_month_entries.get(self.selected_income_entry)?;
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
                let total = month.own_minor + month.reference_minor;
                Some(format!(
                    "{}\t{}\t{}\t{}",
                    month.month,
                    crate::format_amount_minor(month.own_minor, "GBP"),
                    crate::format_amount_minor(month.reference_minor, "GBP"),
                    crate::format_amount_minor(total, "GBP")
                ))
            }
            Screen::TransferMonth => {
                let entry = self
                    .transfer_month_entries
                    .get(self.selected_transfer_entry)?;
                let from = resolve_transfer_leg_name(
                    entry.out_account_id,
                    entry.out_sort.as_deref(),
                    entry.out_account.as_deref(),
                    &self.accounts,
                    &self.household_accounts,
                );
                let to = resolve_transfer_leg_name(
                    entry.in_account_id,
                    entry.in_sort.as_deref(),
                    entry.in_account.as_deref(),
                    &self.accounts,
                    &self.household_accounts,
                );
                let description = entry
                    .out_description
                    .as_deref()
                    .or(entry.in_description.as_deref())
                    .unwrap_or("");
                Some(format!(
                    "{}\t{}\t{}\t{}\t{}",
                    entry.occurred_on,
                    crate::format_amount_minor(entry.amount_minor, &entry.currency),
                    description,
                    from,
                    to
                ))
            }
            Screen::Gap => None,
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

    /// Whether `back()` has anywhere to return to — `false` on the screen
    /// the user launched into (or landed on via a leader-key jump with an
    /// empty history), which is when `q`/`Esc` should quit instead.
    pub fn can_go_back(&self) -> bool {
        !self.nav_stack.is_empty()
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

/// Loads `Screen::Gap`'s data: month-by-month rows (excluding the current,
/// still-in-progress month) plus cash balances at the start of the year and
/// at the end of the last complete month. Shared by `App::new` (the Gap
/// screen is what the user sees on launch) and `App::open_gap` (jumping
/// back to it via `<space>g`), so both stay in sync.
fn load_gap_data(db: &Db) -> anyhow::Result<(Vec<MonthlyGap>, i64, i64)> {
    use chrono::Datelike;

    let today = chrono::Local::now().date_naive();
    let current_month = today.format("%Y-%m").to_string();
    let monthly_gap = db
        .monthly_gap_totals()?
        .into_iter()
        .filter(|m| m.month < current_month)
        .collect();
    // The last day of the last complete month — both the cash comparison
    // and the ledger totals above should end at the same cutoff, rather
    // than the ledger stopping in June while cash keeps counting into July.
    let period_end = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .expect("valid date")
        .pred_opt()
        .expect("month always has a preceding day");
    let year_start = format!("{}-01-01", today.year());
    let cash_at_year_start = db.cash_balance_as_of(&year_start)?;
    let cash_now = db.cash_balance_as_of(&period_end.format("%Y-%m-%d").to_string())?;
    Ok((monthly_gap, cash_at_year_start, cash_now))
}

/// Guesses a Registered Person `name` value from a transaction description,
/// for pre-filling the "add reference" form's Name field: the first two
/// whitespace-separated words (e.g. `"S Barritt"` out of `"S Barritt FARTER
/// BGC"`) — matches `derive::matches_person_name`'s own `"<initial>
/// <Surname>"`/`"<Surname> <initial>"` variants directly, since a
/// two-word truncated form is itself a valid match target. Just a starting
/// point — the user can edit it before submitting.
fn guess_person_name(description: &str) -> String {
    description
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Resolves one side (out or in) of a transfer entry to a display name: a
/// tracked account first (e.g. "Adventure Fund", when `account_id` is
/// resolved — always true once that side's transaction is known), then a
/// reference household account (e.g. "Joint Annual Expense", matched by the
/// raw decoded digits), falling back to the raw sort code/account number
/// digits if neither matches — that fallback is itself a signal something's
/// off, similar in spirit to how a `"fallback"` rule name flags a
/// low-confidence spend classification elsewhere, so it's shown plainly
/// rather than hidden.
pub(crate) fn resolve_transfer_leg_name(
    account_id: Option<Id>,
    sort: Option<&str>,
    account_number: Option<&str>,
    accounts: &[AccountStatus],
    household_accounts: &[HouseholdAccountRef],
) -> String {
    if let Some(id) = account_id {
        if let Some(status) = accounts.iter().find(|s| s.account.id == id) {
            return status.account.name.clone();
        }
    }

    let (Some(sort), Some(account_number)) = (sort, account_number) else {
        return "?".to_string();
    };

    if let Some(household) = household_accounts
        .iter()
        .find(|h| h.sort_code == sort && h.account_number.starts_with(account_number))
    {
        return household
            .label
            .clone()
            .unwrap_or_else(|| format!("{sort} {account_number}"));
    }

    format!("{sort} {account_number}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account_status(id: Id, name: &str) -> AccountStatus {
        AccountStatus {
            account: crate::model::Account {
                id,
                name: name.into(),
                institution: None,
                account_type: crate::model::AccountType::Savings,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
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
    fn resolve_transfer_leg_name_matches_tracked_account_by_id() {
        let accounts = vec![account_status(4, "Adventure Fund")];
        let household = vec![household_ref("209912", "12345678", "Should not be used")];

        assert_eq!(
            resolve_transfer_leg_name(Some(4), None, None, &accounts, &household),
            "Adventure Fund"
        );
    }

    #[test]
    fn resolve_transfer_leg_name_falls_back_to_household_reference_account() {
        let accounts = vec![account_status(1, "Current Account")];
        let household = vec![household_ref("609934", "99998888", "Joint Annual Expense")];

        assert_eq!(
            resolve_transfer_leg_name(
                None,
                Some("609934"),
                Some("99998888"),
                &accounts,
                &household
            ),
            "Joint Annual Expense"
        );
    }

    #[test]
    fn resolve_transfer_leg_name_matches_a_truncated_household_account_number() {
        // Barclays truncates the account-number digits when a long label
        // pushes the NAME field past its length limit — the stored digits
        // may be a prefix of the real account number.
        let household = vec![household_ref("208794", "23165086", "Bills Account")];

        assert_eq!(
            resolve_transfer_leg_name(None, Some("208794"), Some("231650"), &[], &household),
            "Bills Account"
        );
    }

    #[test]
    fn resolve_transfer_leg_name_falls_back_to_raw_digits_when_unresolved() {
        let accounts = vec![account_status(1, "Current Account")];
        let household = vec![household_ref("111111", "22222222", "Unrelated")];

        assert_eq!(
            resolve_transfer_leg_name(
                None,
                Some("609934"),
                Some("99998888"),
                &accounts,
                &household
            ),
            "609934 99998888"
        );
    }
}
