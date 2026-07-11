//! Statement import.
//!
//! Every supported bank/pension export format gets its own `StatementParser`
//! implementation. Adding a new institution's format means writing one new
//! parser, not touching the database or TUI layers.

mod barclays_ofx;
mod generic_csv;
mod pipeline;

pub use barclays_ofx::BarclaysOfxParser;
pub use generic_csv::GenericCsvParser;
pub use pipeline::import_inbox;

use crate::model::{Id, NewAccount, NewTransaction};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("could not read statement file: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse statement: {0}")]
    Parse(String),
}

/// Parses a single downloaded statement file into transactions ready to
/// insert for a given account. Implementations should not touch the
/// database themselves — that keeps parsers trivially unit-testable.
pub trait StatementParser {
    /// Human-readable name, e.g. `"Generic CSV"`.
    fn name(&self) -> &'static str;

    /// If the file itself identifies which account it belongs to (e.g. OFX's
    /// `BANKACCTFROM`), returns the account to resolve/create for it. `None`
    /// means the format carries no account identity of its own (e.g. a
    /// generic CSV) and the caller must supply one.
    fn account_identity(&self, _path: &Path) -> Result<Option<NewAccount>, ImportError> {
        Ok(None)
    }

    /// If the file carries a bank-reported balance anchor (e.g. OFX
    /// `LEDGERBAL`), returns `(balance_minor, as_of)` — the transaction list
    /// in a statement often doesn't reach back to account opening, so this
    /// is needed to know the real balance rather than just summing
    /// transactions. `None` means the format carries no such anchor.
    fn balance_snapshot(&self, _path: &Path) -> Result<Option<(i64, String)>, ImportError> {
        Ok(None)
    }

    fn parse(&self, path: &Path, account_id: Id) -> Result<Vec<NewTransaction>, ImportError>;
}
