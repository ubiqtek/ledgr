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
/// or person. Internal transfers between household accounts (including
/// credit card payments) never produce one of these; see `TransferEntry`
/// instead.
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
    /// The original charge this entry refunds, if this entry is itself a
    /// refund. See `schema.sql`'s doc comment on the column.
    pub refunds_spend_entry_id: Option<Id>,
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
    pub refunds_spend_entry_id: Option<Id>,
    pub classified_by: ClassifiedBy,
    pub confidence: Option<f64>,
    pub rule_name: Option<String>,
}

/// A spend entry alongside the id of the account its source transaction was
/// posted to — backs the TUI's per-month drill-down, so the user can verify
/// spend against the account it actually came from (e.g. spotting a card
/// payment that should have matched a Barclaycard transaction but didn't).
/// A spend entry's source account isn't stored on `spend_entries` itself —
/// it's derived via `spend_entry_sources` back to `transactions`. Carries
/// the raw id rather than a display name so the TUI can resolve it through
/// the same (possibly user-overridden, see `Config::apply_account_name_overrides`)
/// account list it already holds, rather than a second, divergent lookup.
#[derive(Debug, Clone)]
pub struct SpendEntryWithAccount {
    pub entry: SpendEntry,
    pub account_id: Id,
}

/// One row of the Monthly Gap view: total spend for a calendar month.
/// Income/gap columns land once Delta: The Gap, Task 1 (income ledger)
/// exists — see `doc/planning/plan.md`.
#[derive(Debug, Clone)]
pub struct MonthlySpend {
    /// `YYYY-MM`.
    pub month: String,
    /// Net of that month's `spend_entries` (signed, negative = money out —
    /// same convention as `amount_minor` elsewhere).
    pub spend_minor: i64,
}

/// How a transfer entry's counterpart leg was found — see
/// doc/implementation-notes/transfer-ledger-design.md, "Pairing algorithm".
/// The shared `Match` postfix is deliberate (mirrors each variant's
/// `pair_method` string, e.g. `"description_match"`), not an oversight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum TransferPairMethod {
    /// The counterpart's own description cross-references this account
    /// (manual transfers — `Db::find_transfer_counterpart`).
    DescriptionMatch,
    /// Equal-and-opposite amount, a date window, and both legs
    /// independently classifying the other as their household counterpart
    /// (automated transfers, e.g. standing orders).
    AmountDateMatch,
    /// Equal-and-opposite amount and a date window, where the candidate
    /// leg's own decoded counterpart resolves to *itself* rather than the
    /// true sender — real data found some automated transfers' receiving
    /// legs self-reference this way (e.g. the SHARED BILLS ACCO standing
    /// order), which defeats `AmountDateMatch`'s mutual-agreement check.
    /// Weaker signal than `AmountDateMatch` (no cross-check from the
    /// candidate's own decode), so tried last.
    SelfReferenceMatch,
    /// A **credit card payment** (see `ubiquitous-language.md`): the bank-side
    /// debit paired with its credit card account's payment-received line by
    /// date + exact amount (`Db::find_card_payment_counterpart`) — no
    /// household-registry decode involved at all, since a card statement
    /// carries no stable per-transaction identity to cross-reference (see
    /// doc/kb/barclaycard/pdf-export-structure.md). Kept as its own variant
    /// rather than folded into the tiers above because the matching
    /// mechanism is unrelated (account-type lookup, not a `NAME`-field
    /// decode).
    CreditCardPaymentMatch,
}

impl TransferPairMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransferPairMethod::DescriptionMatch => "description_match",
            TransferPairMethod::AmountDateMatch => "amount_date_match",
            TransferPairMethod::SelfReferenceMatch => "self_reference_match",
            TransferPairMethod::CreditCardPaymentMatch => "credit_card_payment_match",
        }
    }
}

/// Which side of a transfer entry a transaction is being recorded as —
/// `derive::classify` resolves one raw transaction at a time, so the
/// caller must say which slot (`Db::TransferLeg` methods) it belongs to.
/// Structural, not a sign convention stored on the row: `transfer_entries`
/// keeps money-out and money-in in separate columns (`out_*`/`in_*`)
/// rather than a signed `amount_minor` relative to "this account", exactly
/// because one row can represent both accounts at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferLegRole {
    /// Money left this leg's account.
    Out,
    /// Money arrived in this leg's account.
    In,
}

/// One raw transaction being recorded as a leg of a transfer entry —
/// either creating a new one-sided row, or filling in the previously-empty
/// side of an existing one. See
/// doc/implementation-notes/transfer-ledger-design.md.
#[derive(Debug, Clone)]
pub struct NewTransferLeg {
    pub transaction_id: Id,
    pub account_id: Id,
    pub role: TransferLegRole,
    pub occurred_on: String,
    /// Unsigned magnitude — see `TransferLegRole`'s doc comment.
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    /// This leg's own decode of who its counterpart *should* be — the raw
    /// digits are always known (classify() only reaches
    /// `Classification::InternalTransfer` once the household registry
    /// recognises them); the resolved `accounts.id` is `Some` only when
    /// the counterpart is itself a tracked account.
    pub counterpart_sort_code: String,
    pub counterpart_account_number: String,
    pub counterpart_account_id: Option<Id>,
    pub classified_by: ClassifiedBy,
    pub confidence: Option<f64>,
    pub rule_name: Option<String>,
}

/// One entry in the derived transfer ledger — one real-world transfer,
/// linking the transactions on both sides directly (`Db::insert_transfer_leg`/
/// `Db::complete_transfer_leg`/`Db::create_paired_transfer`). Persisted by
/// `run_derivation` into `transfer_entries`; the Monthly Transfers
/// screen only ever queries this table (ADR 0009) — ledgr never re-derives
/// it live. Either side may be unresolved (no transaction found yet, or
/// never findable — a Reference Household Account has no `transactions.id`
/// to ever point at); `*_sort`/`*_account` carry the raw decoded digits
/// either way, resolved to a display label by `App::resolve_transfer_leg`
/// (a tracked account's real name, a Reference Household Account's
/// configured label from config, or the raw digits as a last-resort
/// fallback).
#[derive(Debug, Clone)]
pub struct TransferEntry {
    pub id: Id,
    /// ISO 8601 date, e.g. `2026-07-11` — same convention as
    /// `Transaction::posted_at`. Reflects the outgoing leg's date once
    /// known (the canonical, displayed side); the incoming leg's date
    /// only if the outgoing leg hasn't been found yet.
    pub occurred_on: String,
    /// Unsigned magnitude — see `TransferLegRole`'s doc comment.
    pub amount_minor: i64,
    pub currency: String,

    pub out_transaction_id: Option<Id>,
    pub out_account_id: Option<Id>,
    pub out_sort: Option<String>,
    pub out_account: Option<String>,
    pub out_description: Option<String>,

    pub in_transaction_id: Option<Id>,
    pub in_account_id: Option<Id>,
    pub in_sort: Option<String>,
    pub in_account: Option<String>,
    pub in_description: Option<String>,

    pub pair_method: Option<TransferPairMethod>,
    pub pair_confidence: Option<f64>,
}

/// A `transfer_entries` row still missing one side — the candidate set for
/// `run_derivation`'s re-pairing sweep (see
/// doc/implementation-notes/transfer-ledger-design.md, "Pairing algorithm").
/// Deliberately not the display-facing `TransferEntry`: this only carries
/// what the sweep needs (no raw sort/account digits, no resolved names).
#[derive(Debug, Clone)]
pub struct OpenTransferEntry {
    pub id: Id,
    pub occurred_on: String,
    pub amount_minor: i64,
    pub currency: String,
    pub out_transaction_id: Option<Id>,
    pub out_account_id: Option<Id>,
    pub out_description: Option<String>,
    pub in_transaction_id: Option<Id>,
    pub in_account_id: Option<Id>,
    pub in_description: Option<String>,
}

/// One row of the Monthly Transfers view: money moved to/from household
/// accounts in a calendar month. Both directions are kept separate — not
/// netted — so the user can see "£X went out, £Y came in" per month, the
/// whole point of the screen (see `doc/planning/plan.md`).
#[derive(Debug, Clone)]
pub struct MonthlyTransfer {
    /// `YYYY-MM`.
    pub month: String,
    /// Sum of outbound transfer amounts that month (negative — same signed
    /// convention as `amount_minor` elsewhere).
    pub transferred_out_minor: i64,
    /// Sum of inbound transfer amounts that month (positive).
    pub transferred_in_minor: i64,
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
