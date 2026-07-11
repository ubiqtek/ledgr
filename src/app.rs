use crate::config::Config;
use crate::db::Db;
use crate::model::{Account, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Accounts,
    Transactions,
}

pub struct App {
    pub db: Db,
    pub screen: Screen,
    pub accounts: Vec<Account>,
    pub selected_account: usize,
    pub transactions: Vec<Transaction>,
    pub selected_transaction: usize,
    pub should_quit: bool,
    pub status: String,
}

impl App {
    pub fn new(db: Db) -> anyhow::Result<Self> {
        let mut accounts = db.list_accounts()?;
        let config = Config::load_or_init(&Config::default_path()?)?;
        config.apply_account_name_overrides(&mut accounts);
        Ok(Self {
            db,
            screen: Screen::Accounts,
            accounts,
            selected_account: 0,
            transactions: Vec::new(),
            selected_transaction: 0,
            should_quit: false,
            status: "j/k or arrows to move, enter to open, esc back, q to quit".into(),
        })
    }

    pub fn open_selected_account(&mut self) -> anyhow::Result<()> {
        let Some(account) = self.accounts.get(self.selected_account) else {
            return Ok(());
        };
        self.transactions = self.db.list_transactions_for_account(account.id)?;
        self.selected_transaction = 0;
        self.screen = Screen::Transactions;
        Ok(())
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = match self.screen {
            Screen::Accounts => self.accounts.len(),
            Screen::Transactions => self.transactions.len(),
        };
        if len == 0 {
            return;
        }
        let selected = match self.screen {
            Screen::Accounts => &mut self.selected_account,
            Screen::Transactions => &mut self.selected_transaction,
        };
        let next = *selected as i32 + delta;
        *selected = next.clamp(0, len as i32 - 1) as usize;
    }

    pub fn back(&mut self) {
        if self.screen == Screen::Transactions {
            self.screen = Screen::Accounts;
        }
    }
}
