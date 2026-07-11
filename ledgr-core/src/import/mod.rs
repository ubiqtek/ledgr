//! Statement import.
//!
//! Every supported bank/pension export format gets its own `StatementParser`
//! implementation. Adding a new institution's format means writing one new
//! parser, not touching the database or TUI layers.

mod generic_csv;

pub use generic_csv::GenericCsvParser;

use crate::model::{Id, NewTransaction};
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

    fn parse(&self, path: &Path, account_id: Id) -> Result<Vec<NewTransaction>, ImportError>;
}
