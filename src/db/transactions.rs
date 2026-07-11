use super::Db;
use crate::model::{Id, NewTransaction, Transaction};
use rusqlite::params;

impl Db {
    /// Returns `None` (inserting nothing) if `new.external_id` is `Some` and
    /// already exists for this account — the case a statement gets
    /// re-imported under a different file hash (e.g. re-saved) but overlaps
    /// transactions already imported from an earlier download.
    pub fn insert_transaction(&self, new: &NewTransaction) -> rusqlite::Result<Option<Id>> {
        let rows = self.conn().execute(
            "INSERT OR IGNORE INTO transactions
                (account_id, statement_id, posted_at, amount_minor, currency,
                 description, raw_description, category_id, external_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                new.account_id,
                new.statement_id,
                new.posted_at,
                new.amount_minor,
                new.currency,
                new.description,
                new.raw_description,
                new.category_id,
                new.external_id,
            ],
        )?;
        Ok((rows > 0).then(|| self.conn().last_insert_rowid()))
    }

    pub fn list_transactions_for_account(
        &self,
        account_id: Id,
    ) -> rusqlite::Result<Vec<Transaction>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, account_id, statement_id, posted_at, amount_minor, currency,
                    description, raw_description, category_id, external_id
             FROM transactions
             WHERE account_id = ?1
             ORDER BY posted_at DESC, id DESC",
        )?;
        let rows = stmt.query_map(params![account_id], Self::row_to_transaction)?;
        rows.collect()
    }

    fn row_to_transaction(row: &rusqlite::Row) -> rusqlite::Result<Transaction> {
        Ok(Transaction {
            id: row.get(0)?,
            account_id: row.get(1)?,
            statement_id: row.get(2)?,
            posted_at: row.get(3)?,
            amount_minor: row.get(4)?,
            currency: row.get(5)?,
            description: row.get(6)?,
            raw_description: row.get(7)?,
            category_id: row.get(8)?,
            external_id: row.get(9)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AccountType, NewAccount};

    #[test]
    fn insert_and_list_transactions() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: None,
                account_type: AccountType::Current,
                currency: "GBP".into(),
            })
            .expect("insert account");

        db.insert_transaction(&NewTransaction {
            account_id,
            statement_id: None,
            posted_at: "2026-07-01".into(),
            amount_minor: -2599,
            currency: "GBP".into(),
            description: "Groceries".into(),
            raw_description: Some("TESCO STORES 1234".into()),
            category_id: None,
            external_id: None,
        })
        .expect("insert transaction");

        let txs = db
            .list_transactions_for_account(account_id)
            .expect("list transactions");
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].amount_minor, -2599);
    }

    #[test]
    fn inserting_a_transaction_with_a_duplicate_external_id_is_a_no_op() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: None,
                account_type: AccountType::Current,
                currency: "GBP".into(),
            })
            .expect("insert account");

        let mut txn = NewTransaction {
            account_id,
            statement_id: None,
            posted_at: "2026-07-01".into(),
            amount_minor: -2599,
            currency: "GBP".into(),
            description: "Groceries".into(),
            raw_description: Some("TESCO STORES 1234".into()),
            category_id: None,
            external_id: Some("FIT123".into()),
        };
        let first = db.insert_transaction(&txn).expect("insert transaction");
        assert!(first.is_some());

        // Same FITID, as if the same statement were re-imported under a
        // different file hash (e.g. re-saved) with an overlapping date range.
        txn.description = "Groceries (re-saved copy)".into();
        let second = db.insert_transaction(&txn).expect("insert transaction");
        assert!(second.is_none(), "duplicate external_id must be a no-op");

        let txs = db
            .list_transactions_for_account(account_id)
            .expect("list transactions");
        assert_eq!(txs.len(), 1, "should not have duplicated the transaction");
    }
}
