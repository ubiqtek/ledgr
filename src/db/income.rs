//! Persistence for the derived income ledger — `income_entries` and
//! `income_entry_sources`. See doc/planning/plan.md, Delta: The Gap, Task 1,
//! and ADR 0009. Mirrors `src/db/spend.rs`'s shape; classification logic
//! itself lives in `crate::derive`, kept separate from persistence so the
//! rules stay unit-testable without a database.

use super::Db;
use crate::model::{ClassifiedBy, Id, IncomeEntry, IncomeEntryWithAccount, MonthlyIncome, NewIncomeEntry};
use rusqlite::params;

impl Db {
    /// Inserts a new income entry and records `source_transaction_id` as its
    /// source — the only case the derivation pass needs (one raw transaction
    /// -> one income entry).
    pub fn insert_income_entry_with_source(
        &self,
        new: &NewIncomeEntry,
        source_transaction_id: Id,
    ) -> rusqlite::Result<Id> {
        let income_entry_id = self.insert_income_entry(new)?;
        self.insert_income_entry_source(income_entry_id, source_transaction_id)?;
        Ok(income_entry_id)
    }

    pub fn insert_income_entry(&self, new: &NewIncomeEntry) -> rusqlite::Result<Id> {
        self.conn().execute(
            "INSERT INTO income_entries
                (occurred_on, amount_minor, currency, counterparty, description, note,
                 classified_by, confidence, rule_name, classified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                new.occurred_on,
                new.amount_minor,
                new.currency,
                new.counterparty,
                new.description,
                new.note,
                new.classified_by.as_str(),
                new.confidence,
                new.rule_name,
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    pub fn insert_income_entry_source(
        &self,
        income_entry_id: Id,
        transaction_id: Id,
    ) -> rusqlite::Result<()> {
        self.conn().execute(
            "INSERT OR IGNORE INTO income_entry_sources (income_entry_id, transaction_id)
             VALUES (?1, ?2)",
            params![income_entry_id, transaction_id],
        )?;
        Ok(())
    }

    /// Net income per calendar month, most recent first — backs the TUI's
    /// Monthly Income screen. Same shape as `monthly_spend_totals`.
    pub fn monthly_income_totals(&self) -> rusqlite::Result<Vec<MonthlyIncome>> {
        let mut stmt = self.conn().prepare(
            "SELECT substr(occurred_on, 1, 7) AS month, SUM(amount_minor)
             FROM income_entries
             GROUP BY month
             ORDER BY month DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MonthlyIncome {
                month: row.get(0)?,
                income_minor: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    /// All income entries for one calendar month (`month` as `YYYY-MM`),
    /// oldest first, alongside the id of the account each entry's source
    /// transaction was posted to — backs the TUI's per-month drill-down.
    /// Same join shape as `spend_entries_for_month`.
    pub fn income_entries_for_month(
        &self,
        month: &str,
    ) -> rusqlite::Result<Vec<IncomeEntryWithAccount>> {
        let mut stmt = self.conn().prepare(
            "SELECT ie.id, ie.occurred_on, ie.amount_minor, ie.currency, ie.counterparty,
                    ie.description, ie.note, ie.classified_by, ie.confidence, ie.rule_name,
                    ie.classified_at, t.account_id, t.id
             FROM income_entries ie
             JOIN income_entry_sources ies ON ies.income_entry_id = ie.id
             JOIN transactions t ON t.id = ies.transaction_id
             WHERE substr(ie.occurred_on, 1, 7) = ?1
             ORDER BY ie.occurred_on, ie.id",
        )?;
        let rows = stmt.query_map([month], |row| {
            let classified_by_str: String = row.get(7)?;
            Ok(IncomeEntryWithAccount {
                entry: IncomeEntry {
                    id: row.get(0)?,
                    occurred_on: row.get(1)?,
                    amount_minor: row.get(2)?,
                    currency: row.get(3)?,
                    counterparty: row.get(4)?,
                    description: row.get(5)?,
                    note: row.get(6)?,
                    classified_by: ClassifiedBy::parse(&classified_by_str)
                        .unwrap_or(ClassifiedBy::Rule),
                    confidence: row.get(8)?,
                    rule_name: row.get(9)?,
                    classified_at: row.get(10)?,
                },
                account_id: row.get(11)?,
                transaction_id: row.get(12)?,
            })
        })?;
        rows.collect()
    }
}
