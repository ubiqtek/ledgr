//! Import.
//!
//! Every supported bank/pension export format gets its own `ImportFileParser`
//! implementation. Adding a new institution's format means writing one new
//! parser, not touching the database or TUI layers.

mod barclaycard_pdf;
mod barclays_ofx;
mod generic_csv;
mod pipeline;

pub use barclaycard_pdf::BarclaycardPdfParser;
pub use barclays_ofx::BarclaysOfxParser;
pub use generic_csv::GenericCsvParser;
pub use pipeline::import_inbox;

use crate::model::{CardIdentity, Id, NewAccount, NewTransaction};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("could not read import file: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse import: {0}")]
    Parse(String),
}

/// Parses a single downloaded import file into transactions ready to
/// insert for a given account. Implementations should not touch the
/// database themselves — that keeps parsers trivially unit-testable.
pub trait ImportFileParser {
    /// Human-readable name, e.g. `"Generic CSV"`. Required of every parser
    /// per the trait contract (see CLAUDE.md's Architecture section), but
    /// not yet surfaced anywhere (e.g. a "which format matched this file"
    /// line in `ledgr import`'s own output).
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// If the file itself identifies which account it belongs to (e.g. OFX's
    /// `BANKACCTFROM`), returns the account to resolve/create for it. `None`
    /// means the format carries no account identity of its own (e.g. a
    /// generic CSV) and the caller must supply one.
    fn account_identity(&self, _path: &Path) -> Result<Option<NewAccount>, ImportError> {
        Ok(None)
    }

    /// For formats with no stable account-identity field at all — e.g. a
    /// credit card statement export, which only ever exposes a maskable
    /// last-4 card number that changes on reissue (see
    /// doc/kb/barclaycard/pdf-export-structure.md) — returns the observed
    /// card identity instead. Mutually exclusive with `account_identity` in
    /// practice: a format implements one or the other, never both. `None`
    /// means this format doesn't carry this kind of identity either.
    fn card_identity(&self, _path: &Path) -> Result<Option<CardIdentity>, ImportError> {
        Ok(None)
    }

    /// If the file carries a bank-reported balance anchor (e.g. OFX
    /// `LEDGERBAL`), returns `(balance_minor, as_of)` — the transaction list
    /// in an import often doesn't reach back to account opening, so this
    /// is needed to know the real balance rather than just summing
    /// transactions. `None` means the format carries no such anchor.
    fn balance_snapshot(&self, _path: &Path) -> Result<Option<(i64, String)>, ImportError> {
        Ok(None)
    }

    fn parse(&self, path: &Path, account_id: Id) -> Result<Vec<NewTransaction>, ImportError>;
}
