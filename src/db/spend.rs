//! Persistence for the derived spend ledger — `spend_entries` and
//! `spend_entry_sources` — plus `transaction_links` access used by transfer
//! pairing and refund linking. See
//! doc/implementation-notes/spend-ledger-design.md. Classification logic
//! itself lives in `crate::derive`, kept separate from persistence so the
//! rules stay unit-testable without a database.

use super::Db;
use crate::model::{
    ClassifiedBy, Id, LinkRelation, MonthlySpend, NewSpendEntry, SpendEntry, SpendEntrySourceRole,
    SpendEntryWithAccount, Transaction,
};
use rusqlite::{params, OptionalExtension};

impl Db {
    /// Transactions not yet linked to a spend entry as their `source` — the
    /// candidate set for a derivation pass. Transactions classified as
    /// internal transfers or left out-of-scope (income, cash) never gain a
    /// spend entry, so they stay in this set forever; re-processing them is
    /// harmless (transfer pairing is idempotent via a UNIQUE constraint, and
    /// re-classifying an out-of-scope transaction is a no-op).
    pub fn pending_derivation_transactions(&self) -> rusqlite::Result<Vec<Transaction>> {
        let mut stmt = self.conn().prepare(
            "SELECT t.id, t.account_id, t.import_id, t.posted_at, t.amount_minor,
                    t.currency, t.description, t.raw_description, t.trn_type, t.external_id,
                    t.notes
             FROM transactions t
             LEFT JOIN spend_entry_sources s
                    ON s.transaction_id = t.id AND s.role = 'source'
             WHERE s.transaction_id IS NULL
             ORDER BY t.posted_at, t.id",
        )?;
        let rows = stmt.query_map([], Self::row_to_transaction)?;
        rows.collect()
    }

    /// Inserts a new spend entry and records `source_transaction_id` as its
    /// `source` provenance row — the common case for the derivation pass
    /// (one raw transaction -> one spend entry).
    pub fn insert_spend_entry_with_source(
        &self,
        new: &NewSpendEntry,
        source_transaction_id: Id,
    ) -> rusqlite::Result<Id> {
        let spend_entry_id = self.insert_spend_entry(new)?;
        self.insert_spend_entry_source(
            spend_entry_id,
            source_transaction_id,
            SpendEntrySourceRole::Source,
        )?;
        Ok(spend_entry_id)
    }

    pub fn insert_spend_entry(&self, new: &NewSpendEntry) -> rusqlite::Result<Id> {
        self.conn().execute(
            "INSERT INTO spend_entries
                (occurred_on, amount_minor, currency, counterparty, description, note,
                 category_id, classified_by, confidence, rule_name, classified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                new.occurred_on,
                new.amount_minor,
                new.currency,
                new.counterparty,
                new.description,
                new.note,
                new.category_id,
                new.classified_by.as_str(),
                new.confidence,
                new.rule_name,
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Sets (or, given `None`, clears) a spend entry's free-text `note` —
    /// e.g. the user's own record of having looked into an unrecognised
    /// merchant and decided it's legitimate. Deliberately doesn't touch
    /// `classified_by`/`confidence`/`rule_name`: a note is just an
    /// annotation on top of whatever classified the entry, not a
    /// reclassification (that's still `classified_by = 'manual'`, unused by
    /// this method).
    pub fn set_spend_entry_note(&self, id: Id, note: Option<&str>) -> rusqlite::Result<()> {
        self.conn().execute(
            "UPDATE spend_entries SET note = ?1 WHERE id = ?2",
            params![note, id],
        )?;
        Ok(())
    }

    /// All spend entries, most recent first — the derivation pass's output,
    /// for `ledgr status`-style summaries and the future review queue TUI
    /// (Spend Ledger Task 3).
    pub fn list_spend_entries(&self) -> rusqlite::Result<Vec<SpendEntry>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, occurred_on, amount_minor, currency, counterparty, description,
                    note, category_id, classified_by, confidence, rule_name, classified_at
             FROM spend_entries
             ORDER BY occurred_on DESC, id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let classified_by_str: String = row.get(8)?;
            Ok(SpendEntry {
                id: row.get(0)?,
                occurred_on: row.get(1)?,
                amount_minor: row.get(2)?,
                currency: row.get(3)?,
                counterparty: row.get(4)?,
                description: row.get(5)?,
                note: row.get(6)?,
                category_id: row.get(7)?,
                classified_by: ClassifiedBy::parse(&classified_by_str)
                    .unwrap_or(ClassifiedBy::Rule),
                confidence: row.get(9)?,
                rule_name: row.get(10)?,
                classified_at: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    /// All spend entries for one calendar month (`month` as `YYYY-MM`),
    /// oldest first, alongside the id of the account each entry's source
    /// transaction was posted to — backs the TUI's per-month drill-down, so
    /// the user can eyeball whether anything that isn't real spend (a missed
    /// transfer, say) has slipped into the ledger, and verify spend against
    /// the account it actually came from. Joins through `spend_entry_sources`
    /// (`role = 'source'`) since `spend_entries` itself carries no
    /// `account_id` — see `SpendEntryWithAccount`'s doc comment.
    pub fn spend_entries_for_month(
        &self,
        month: &str,
    ) -> rusqlite::Result<Vec<SpendEntryWithAccount>> {
        let mut stmt = self.conn().prepare(
            "SELECT se.id, se.occurred_on, se.amount_minor, se.currency, se.counterparty,
                    se.description, se.note, se.category_id, se.classified_by, se.confidence,
                    se.rule_name, se.classified_at, t.account_id
             FROM spend_entries se
             JOIN spend_entry_sources ses ON ses.spend_entry_id = se.id AND ses.role = 'source'
             JOIN transactions t ON t.id = ses.transaction_id
             WHERE substr(se.occurred_on, 1, 7) = ?1
             ORDER BY se.occurred_on, se.id",
        )?;
        let rows = stmt.query_map([month], |row| {
            let classified_by_str: String = row.get(8)?;
            Ok(SpendEntryWithAccount {
                entry: SpendEntry {
                    id: row.get(0)?,
                    occurred_on: row.get(1)?,
                    amount_minor: row.get(2)?,
                    currency: row.get(3)?,
                    counterparty: row.get(4)?,
                    description: row.get(5)?,
                    note: row.get(6)?,
                    category_id: row.get(7)?,
                    classified_by: ClassifiedBy::parse(&classified_by_str)
                        .unwrap_or(ClassifiedBy::Rule),
                    confidence: row.get(9)?,
                    rule_name: row.get(10)?,
                    classified_at: row.get(11)?,
                },
                account_id: row.get(12)?,
            })
        })?;
        rows.collect()
    }

    /// Net spend per calendar month, most recent first — backs the TUI's
    /// Monthly Gap screen. `occurred_on` is `YYYY-MM-DD`, so a plain string
    /// prefix groups correctly without needing SQLite's `strftime`.
    pub fn monthly_spend_totals(&self) -> rusqlite::Result<Vec<MonthlySpend>> {
        let mut stmt = self.conn().prepare(
            "SELECT substr(occurred_on, 1, 7) AS month, SUM(amount_minor)
             FROM spend_entries
             GROUP BY month
             ORDER BY month DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MonthlySpend {
                month: row.get(0)?,
                spend_minor: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    pub fn insert_spend_entry_source(
        &self,
        spend_entry_id: Id,
        transaction_id: Id,
        role: SpendEntrySourceRole,
    ) -> rusqlite::Result<()> {
        self.conn().execute(
            "INSERT OR IGNORE INTO spend_entry_sources (spend_entry_id, transaction_id, role)
             VALUES (?1, ?2, ?3)",
            params![spend_entry_id, transaction_id, role.as_str()],
        )?;
        Ok(())
    }

    /// Records an edge between two transactions (e.g. both legs of a
    /// transfer, or a refund pointing back at its original charge).
    /// `INSERT OR IGNORE` because a re-run derivation pass may attempt the
    /// same link twice — the `UNIQUE(from, to, relation)` constraint makes
    /// that a no-op rather than an error.
    pub fn insert_transaction_link(
        &self,
        from_transaction_id: Id,
        to_transaction_id: Id,
        relation: LinkRelation,
        confidence: Option<f64>,
    ) -> rusqlite::Result<()> {
        self.conn().execute(
            "INSERT OR IGNORE INTO transaction_links
                (from_transaction_id, to_transaction_id, relation, confidence)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                from_transaction_id,
                to_transaction_id,
                relation.as_str(),
                confidence
            ],
        )?;
        Ok(())
    }

    /// Best-effort search for a transfer's counterpart: the other leg,
    /// belonging to the account this transaction's `NAME` prefix points at,
    /// for the equal-and-opposite amount, whose own `NAME` prefix points
    /// back at this transaction's account, within a 3-day window (transfers
    /// are usually same-day; the extra slack covers weekends). See
    /// doc/implementation-notes/spend-ledger-design.md, "Transfer pairing".
    #[allow(clippy::too_many_arguments)]
    pub fn find_transfer_counterpart(
        &self,
        transaction_id: Id,
        own_sort_code: &str,
        own_account_number: &str,
        counterpart_sort_code: &str,
        counterpart_account_number: &str,
        amount_minor: i64,
        posted_at: &str,
    ) -> rusqlite::Result<Option<Id>> {
        let own_prefix = format!("{own_sort_code} {own_account_number}");
        self.conn()
            .query_row(
                "SELECT t.id
                 FROM transactions t
                 JOIN accounts a ON a.id = t.account_id
                 WHERE a.sort_code = ?1 AND a.account_number = ?2
                   AND t.amount_minor = ?3
                   AND t.description LIKE (?4 || '%')
                   AND t.id != ?5
                   AND julianday(t.posted_at) BETWEEN julianday(?6) - 3 AND julianday(?6) + 3
                 ORDER BY ABS(julianday(t.posted_at) - julianday(?6))
                 LIMIT 1",
                params![
                    counterpart_sort_code,
                    counterpart_account_number,
                    -amount_minor,
                    own_prefix,
                    transaction_id,
                    posted_at,
                ],
                |row| row.get(0),
            )
            .optional()
    }

    /// Best-effort search for a credit card bill payment's counterpart: any
    /// `CreditCard` account's transaction for the equal-and-opposite amount
    /// within a 3-day window (see
    /// doc/kb/barclaycard/pdf-export-structure.md's recommended
    /// date+amount matching strategy — deliberately doesn't key off the
    /// card number, which isn't stable across a reissue). With only one
    /// registered credit card today this can't yet be ambiguous between
    /// several cards; that will need revisiting once a second card
    /// (Credit Card Transaction Import Task 3) exists.
    pub fn find_card_payment_counterpart(
        &self,
        transaction_id: Id,
        amount_minor: i64,
        posted_at: &str,
    ) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT t.id
                 FROM transactions t
                 JOIN accounts a ON a.id = t.account_id
                 WHERE a.account_type = 'credit_card'
                   AND t.amount_minor = ?1
                   AND t.id != ?2
                   AND julianday(t.posted_at) BETWEEN julianday(?3) - 3 AND julianday(?3) + 3
                 ORDER BY ABS(julianday(t.posted_at) - julianday(?3))
                 LIMIT 1",
                params![-amount_minor, transaction_id, posted_at],
                |row| row.get(0),
            )
            .optional()
    }

    /// Best-effort search for the original charge a card refund pays back:
    /// same account, negative amount of the same magnitude, sharing the
    /// merchant prefix (text before " ON "), posted on or before the refund.
    /// `None` if nothing matches — the refund entry is still recorded (see
    /// the design doc: "linked ... when findable").
    pub fn find_refund_original(
        &self,
        account_id: Id,
        merchant_prefix: &str,
        refund_amount_minor: i64,
        posted_on_or_before: &str,
    ) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT id FROM transactions
                 WHERE account_id = ?1
                   AND amount_minor = ?2
                   AND description LIKE (?3 || '%')
                   AND posted_at <= ?4
                 ORDER BY posted_at DESC
                 LIMIT 1",
                params![
                    account_id,
                    -refund_amount_minor,
                    merchant_prefix,
                    posted_on_or_before
                ],
                |row| row.get(0),
            )
            .optional()
    }
}
