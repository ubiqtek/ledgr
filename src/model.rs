use serde::{Deserialize, Serialize};

pub type Id = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Current,
    Savings,
    CreditCard,
    Pension,
    Investment,
    Other,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Current => "current",
            AccountType::Savings => "savings",
            AccountType::CreditCard => "credit_card",
            AccountType::Pension => "pension",
            AccountType::Investment => "investment",
            AccountType::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "current" => AccountType::Current,
            "savings" => AccountType::Savings,
            "credit_card" => AccountType::CreditCard,
            "pension" => AccountType::Pension,
            "investment" => AccountType::Investment,
            "other" => AccountType::Other,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Id,
    pub name: String,
    pub institution: Option<String>,
    pub account_type: AccountType,
    pub currency: String,
    /// Sort code + account number, when the import format identifies its
    /// own account (e.g. OFX `BANKACCTFROM`). Used by spend ledger
    /// derivation to recognise transfer counterparties.
    pub sort_code: Option<String>,
    pub account_number: Option<String>,
}

/// Fields needed to create a new account; `id` is assigned by the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub name: String,
    pub institution: Option<String>,
    pub account_type: AccountType,
    pub currency: String,
    pub sort_code: Option<String>,
    pub account_number: Option<String>,
}

/// A credit card account's identity as observed in one import — the only
/// stable-ish thing a statement export carries, which isn't very stable at
/// all (see doc/kb/barclaycard/pdf-export-structure.md): the last 4 digits
/// of the card number change on reissue, and nothing in the export ties an
/// old number to a new one. Used instead of `NewAccount`/`account_identity`
/// (which assumes a name/number that's stable across re-imports) for
/// formats where that assumption doesn't hold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardIdentity {
    pub institution: String,
    /// e.g. "Barclaycard Rewards" — the card product, not unique to one
    /// customer (shared BIN range), only useful for a display name.
    pub product_label: String,
    pub last4: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Id,
    pub name: String,
    pub parent_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Id,
    pub account_id: Id,
    pub import_id: Option<Id>,
    /// ISO 8601 date, e.g. `2026-07-11`.
    pub posted_at: String,
    /// Signed amount in minor currency units (e.g. pence), to avoid float drift.
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    pub raw_description: Option<String>,
    /// OFX `TRNTYPE` or equivalent, e.g. `"OTHER"`, `"DIRECTDEBIT"`. Never
    /// reliable alone for identifying a transfer — see the OFX KB article —
    /// but used by spend ledger derivation alongside `description`.
    pub trn_type: Option<String>,
    pub external_id: Option<String>,
    /// Catch-all for import-format detail that doesn't fit any field above.
    pub notes: Option<String>,
}

/// Fields needed to create a new transaction; `id` is assigned by the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub account_id: Id,
    pub import_id: Option<Id>,
    pub posted_at: String,
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    pub raw_description: Option<String>,
    pub trn_type: Option<String>,
    pub external_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkRelation {
    Transfer,
    Refund,
    DuplicateOf,
    Related,
}

impl LinkRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkRelation::Transfer => "transfer",
            LinkRelation::Refund => "refund",
            LinkRelation::DuplicateOf => "duplicate_of",
            LinkRelation::Related => "related",
        }
    }
}

/// An edge between two transactions, e.g. the two legs of a transfer between
/// accounts, or a refund pointing back at its original charge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLink {
    pub id: Id,
    pub from_transaction_id: Id,
    pub to_transaction_id: Id,
    pub relation: LinkRelation,
    /// Set when the link was inferred rather than user-confirmed.
    pub confidence: Option<f64>,
}

/// Provenance of a spend entry's classification (counterparty + category).
/// See doc/implementation-notes/spend-ledger-design.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassifiedBy {
    Rule,
    Matcher,
    Manual,
}

impl ClassifiedBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClassifiedBy::Rule => "rule",
            ClassifiedBy::Matcher => "matcher",
            ClassifiedBy::Manual => "manual",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "rule" => ClassifiedBy::Rule,
            "matcher" => ClassifiedBy::Matcher,
            "manual" => ClassifiedBy::Manual,
            _ => return None,
        })
    }
}

/// One entry in the derived spend ledger — real-world spending to a merchant
/// or person. Internal transfers between household accounts never produce
/// one of these; see `TransactionLink` with `LinkRelation::Transfer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendEntry {
    pub id: Id,
    pub occurred_on: String,
    pub amount_minor: i64,
    pub currency: String,
    pub counterparty: Option<String>,
    pub description: String,
    pub note: Option<String>,
    pub category_id: Option<Id>,
    pub classified_by: ClassifiedBy,
    pub confidence: Option<f64>,
    pub rule_name: Option<String>,
    pub classified_at: String,
}

/// Fields needed to create a new spend entry; `id` is assigned by the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSpendEntry {
    pub occurred_on: String,
    pub amount_minor: i64,
    pub currency: String,
    pub counterparty: Option<String>,
    pub description: String,
    pub note: Option<String>,
    pub category_id: Option<Id>,
    pub classified_by: ClassifiedBy,
    pub confidence: Option<f64>,
    pub rule_name: Option<String>,
}

/// Which raw transaction(s) a spend entry derives from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpendEntrySourceRole {
    /// The raw row the entry represents.
    Source,
    /// A matched transfer carrying the note (spend enrichment) — not
    /// produced by the derivation pass yet, see the spend ledger design doc.
    Annotation,
}

impl SpendEntrySourceRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpendEntrySourceRole::Source => "source",
            SpendEntrySourceRole::Annotation => "annotation",
        }
    }
}
