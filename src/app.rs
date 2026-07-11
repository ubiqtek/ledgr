use crate::config::Config;
use crate::db::{AccountStatus, Db};
use crate::model::Transaction;
use ratatui::widgets::TableState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Accounts,
    Transactions,
    Help,
}

pub struct App {
    pub db: Db,
    pub screen: Screen,
    /// Screen to return to when leaving `Screen::Help`.
    previous_screen: Screen,
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
    pub should_quit: bool,
    pub status: String,
}

impl App {
    pub fn new(db: Db) -> anyhow::Result<Self> {
        let mut accounts = db.account_statuses()?;
        let config = Config::load_or_init(&Config::default_path()?)?;
        config.apply_account_name_overrides(accounts.iter_mut().map(|s| &mut s.account));
        Ok(Self {
            db,
            screen: Screen::Accounts,
            previous_screen: Screen::Accounts,
            accounts,
            selected_account: 0,
            accounts_table_state: TableState::default(),
            transactions: Vec::new(),
            selected_transaction: 0,
            transactions_table_state: TableState::default(),
            should_quit: false,
            status: "j/k move, enter open, ctrl-d/u page, ? help, esc back, q quit".into(),
        })
    }

    pub fn open_selected_account(&mut self) -> anyhow::Result<()> {
        let Some(status) = self.accounts.get(self.selected_account) else {
            return Ok(());
        };
        self.transactions = self.db.list_transactions_for_account(status.account.id)?;
        self.selected_transaction = 0;
        self.screen = Screen::Transactions;
        Ok(())
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = match self.screen {
            Screen::Accounts => self.accounts.len(),
            Screen::Transactions => self.transactions.len(),
            Screen::Help => return,
        };
        if len == 0 {
            return;
        }
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
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
            Screen::Help => return,
        };
        *selected = 0;
    }

    /// Jumps to the last row of the current list, like nvim's `G`.
    pub fn select_last(&mut self) {
        let len = match self.screen {
            Screen::Accounts => self.accounts.len(),
            Screen::Transactions => self.transactions.len(),
            Screen::Help => return,
        };
        if len == 0 {
            return;
        }
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
            Screen::Help => return,
        };
        *selected = len - 1;
    }

    pub fn back(&mut self) {
        match self.screen {
            Screen::Transactions => self.screen = Screen::Accounts,
            Screen::Help => self.screen = self.previous_screen,
            Screen::Accounts => {}
        }
    }

    /// Shows the help screen, or leaves it (returning to whichever screen
    /// was open before) if it's already showing.
    pub fn toggle_help(&mut self) {
        if self.screen == Screen::Help {
            self.back();
        } else {
            self.previous_screen = self.screen;
            self.screen = Screen::Help;
        }
    }
}
