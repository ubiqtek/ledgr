//! Analysis over the local database.
//!
//! Starts small — this is where category breakdowns, net worth over time,
//! and (later) inference-assisted categorization will live.
//!
//! Categorisation lives on the derived spend ledger, not raw transactions —
//! see doc/implementation-notes/spend-ledger-design.md — so breakdowns are
//! computed over `spend_entries`, joined back to `transactions` (via
//! `spend_entry_sources`) only to filter by account.

use crate::db::Db;
use crate::model::Id;

/// Total signed amount (minor units) per category for one account's spend
/// entries, largest magnitude first. `None` category means uncategorised.
pub fn category_totals(db: &Db, account_id: Id) -> rusqlite::Result<Vec<(Option<Id>, i64)>> {
    let mut stmt = db.conn().prepare(
        "SELECT se.category_id, SUM(se.amount_minor) AS total
         FROM spend_entries se
         JOIN spend_entry_sources s ON s.spend_entry_id = se.id AND s.role = 'source'
         JOIN transactions t ON t.id = s.transaction_id
         WHERE t.account_id = ?1
         GROUP BY se.category_id
         ORDER BY ABS(total) DESC",
    )?;
    let rows = stmt.query_map([account_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::derive_spend_entries;
    use crate::model::{AccountType, NewAccount, NewTransaction};

    #[test]
    fn totals_group_by_category() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: None,
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
            })
            .expect("insert account");

        for amount in [-500, -700, 2000] {
            db.insert_transaction(&NewTransaction {
                account_id,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: amount,
                currency: "GBP".into(),
                description: "TESCO STORES ON 01 JUL CPM".into(),
                raw_description: None,
                trn_type: Some("OTHER".into()),
                external_id: None,
            })
            .expect("insert transaction");
        }
        derive_spend_entries(&db, &[]).expect("derive");

        // The +2000 credit has no matching pattern/TRNTYPE, so it's left
        // out of scope (a candidate income transaction) rather than spend;
        // only the two debits become uncategorised spend entries.
        let totals = category_totals(&db, account_id).expect("category totals");
        assert_eq!(totals, vec![(None, -1200)]);
    }
}
