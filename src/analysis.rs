//! Analysis over the local database.
//!
//! Starts small — this is where category breakdowns, net worth over time,
//! and (later) inference-assisted categorization will live.

use crate::db::Db;
use crate::model::Id;

/// Total signed amount (minor units) per category for one account, largest
/// magnitude first. `None` category means uncategorized.
pub fn category_totals(db: &Db, account_id: Id) -> rusqlite::Result<Vec<(Option<Id>, i64)>> {
    let mut stmt = db.conn().prepare(
        "SELECT category_id, SUM(amount_minor) AS total
         FROM transactions
         WHERE account_id = ?1
         GROUP BY category_id
         ORDER BY ABS(total) DESC",
    )?;
    let rows = stmt.query_map([account_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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
            })
            .expect("insert account");

        for amount in [-500, -700, 2000] {
            db.insert_transaction(&NewTransaction {
                account_id,
                statement_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: amount,
                currency: "GBP".into(),
                description: "test".into(),
                raw_description: None,
                category_id: None,
                external_id: None,
            })
            .expect("insert transaction");
        }

        let totals = category_totals(&db, account_id).expect("category totals");
        assert_eq!(totals, vec![(None, 800)]);
    }
}
