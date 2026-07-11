use serde::{Deserialize, Serialize};

pub type Id = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Checking,
    Savings,
    CreditCard,
    Pension,
    Investment,
    Other,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Checking => "checking",
            AccountType::Savings => "savings",
            AccountType::CreditCard => "credit_card",
            AccountType::Pension => "pension",
            AccountType::Investment => "investment",
            AccountType::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "checking" => AccountType::Checking,
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
}

/// Fields needed to create a new account; `id` is assigned by the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccount {
    pub name: String,
    pub institution: Option<String>,
    pub account_type: AccountType,
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
    pub statement_id: Option<Id>,
    /// ISO 8601 date, e.g. `2026-07-11`.
    pub posted_at: String,
    /// Signed amount in minor currency units (e.g. pence), to avoid float drift.
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    pub raw_description: Option<String>,
    pub category_id: Option<Id>,
    pub external_id: Option<String>,
}

/// Fields needed to create a new transaction; `id` is assigned by the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub account_id: Id,
    pub statement_id: Option<Id>,
    pub posted_at: String,
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    pub raw_description: Option<String>,
    pub category_id: Option<Id>,
    pub external_id: Option<String>,
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
