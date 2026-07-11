use super::Db;
use crate::model::Id;
use rusqlite::{params, OptionalExtension};

impl Db {
    /// Records a balance anchor reported by the bank itself (e.g. OFX
    /// `LEDGERBAL`) for `account_id` as of `as_of` (a date, `YYYY-MM-DD`).
    /// A no-op if a snapshot already exists for that exact account/date
    /// (e.g. re-importing the same statement).
    pub fn insert_balance_snapshot(
        &self,
        account_id: Id,
        statement_id: Option<Id>,
        balance_minor: i64,
        as_of: &str,
    ) -> rusqlite::Result<()> {
        self.conn().execute(
            "INSERT OR IGNORE INTO balance_snapshots (account_id, statement_id, balance_minor, as_of)
             VALUES (?1, ?2, ?3, ?4)",
            params![account_id, statement_id, balance_minor, as_of],
        )?;
        Ok(())
    }

    /// The most recent balance anchor for an account, i.e. the balance as
    /// reported by the most recently-dated statement import. Returns
    /// `(balance_minor, as_of)`, or `None` if no statement for this account
    /// has carried a balance snapshot yet.
    pub fn latest_balance_snapshot(&self, account_id: Id) -> rusqlite::Result<Option<(i64, String)>> {
        self.conn()
            .query_row(
                "SELECT balance_minor, as_of FROM balance_snapshots
                 WHERE account_id = ?1
                 ORDER BY as_of DESC LIMIT 1",
                params![account_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
    }

    /// Reconstructs the account balance as of `date` (`YYYY-MM-DD`) from the
    /// nearest balance anchor plus the transactions between that anchor and
    /// `date`. Prefers the earliest anchor on or after `date` (walking
    /// backward, subtracting later transactions); falls back to the latest
    /// anchor before `date` (walking forward, adding later transactions) if
    /// no anchor exists on or after it. Returns `None` if there is no
    /// anchor at all for this account.
    pub fn balance_as_of(&self, account_id: Id, date: &str) -> rusqlite::Result<Option<i64>> {
        if let Some((anchor_balance, anchor_as_of)) = self
            .conn()
            .query_row(
                "SELECT balance_minor, as_of FROM balance_snapshots
                 WHERE account_id = ?1 AND as_of >= ?2
                 ORDER BY as_of ASC LIMIT 1",
                params![account_id, date],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?
        {
            let later_transactions: i64 = self.conn().query_row(
                "SELECT COALESCE(SUM(amount_minor), 0) FROM transactions
                 WHERE account_id = ?1 AND posted_at > ?2 AND posted_at <= ?3",
                params![account_id, date, anchor_as_of],
                |row| row.get(0),
            )?;
            return Ok(Some(anchor_balance - later_transactions));
        }

        if let Some((anchor_balance, anchor_as_of)) = self
            .conn()
            .query_row(
                "SELECT balance_minor, as_of FROM balance_snapshots
                 WHERE account_id = ?1 AND as_of < ?2
                 ORDER BY as_of DESC LIMIT 1",
                params![account_id, date],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?
        {
            let later_transactions: i64 = self.conn().query_row(
                "SELECT COALESCE(SUM(amount_minor), 0) FROM transactions
                 WHERE account_id = ?1 AND posted_at > ?2 AND posted_at <= ?3",
                params![account_id, anchor_as_of, date],
                |row| row.get(0),
            )?;
            return Ok(Some(anchor_balance + later_transactions));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AccountType, NewAccount, NewTransaction};

    fn test_account(db: &Db) -> Id {
        db.insert_account(&NewAccount {
            name: "Current Account".into(),
            institution: None,
            account_type: AccountType::Current,
            currency: "GBP".into(),
        })
        .expect("insert account")
    }

    fn transaction_on(db: &Db, account_id: Id, date: &str, amount_minor: i64) {
        db.insert_transaction(&NewTransaction {
            account_id,
            statement_id: None,
            posted_at: date.into(),
            amount_minor,
            currency: "GBP".into(),
            description: "test".into(),
            raw_description: None,
            category_id: None,
            external_id: None,
        })
        .expect("insert transaction");
    }

    #[test]
    fn latest_balance_snapshot_picks_the_most_recent_anchor() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = test_account(&db);

        db.insert_balance_snapshot(account_id, None, 10_000, "2026-06-01")
            .expect("insert snapshot");
        db.insert_balance_snapshot(account_id, None, 12_345, "2026-07-10")
            .expect("insert snapshot");

        let (balance, as_of) = db
            .latest_balance_snapshot(account_id)
            .expect("latest snapshot")
            .expect("has a snapshot");
        assert_eq!(balance, 12_345);
        assert_eq!(as_of, "2026-07-10");
    }

    #[test]
    fn re_importing_the_same_as_of_does_not_duplicate() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = test_account(&db);

        db.insert_balance_snapshot(account_id, None, 1_000, "2026-07-10")
            .expect("insert snapshot");
        db.insert_balance_snapshot(account_id, None, 1_000, "2026-07-10")
            .expect("insert snapshot again");

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT count(*) FROM balance_snapshots WHERE account_id = ?1",
                params![account_id],
                |row| row.get(0),
            )
            .expect("count snapshots");
        assert_eq!(count, 1);
    }

    #[test]
    fn balance_as_of_walks_backward_from_a_later_anchor() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = test_account(&db);

        // Anchor: balance was 1000 as of 2026-07-10.
        db.insert_balance_snapshot(account_id, None, 1_000, "2026-07-10")
            .expect("insert snapshot");
        // A -300 transaction posted after the target date, before the anchor.
        transaction_on(&db, account_id, "2026-07-05", -300);

        // Balance on 2026-07-01 (before that transaction) should be
        // 1000 - (-300) = 1300, since the -300 hadn't happened yet.
        let balance = db
            .balance_as_of(account_id, "2026-07-01")
            .expect("balance_as_of")
            .expect("has an anchor");
        assert_eq!(balance, 1_300);
    }

    #[test]
    fn balance_as_of_walks_forward_from_an_earlier_anchor() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = test_account(&db);

        // Anchor: balance was 1000 as of 2026-06-01.
        db.insert_balance_snapshot(account_id, None, 1_000, "2026-06-01")
            .expect("insert snapshot");
        // A +200 transaction posted after the anchor, before the target date.
        transaction_on(&db, account_id, "2026-06-15", 200);

        let balance = db
            .balance_as_of(account_id, "2026-07-01")
            .expect("balance_as_of")
            .expect("has an anchor");
        assert_eq!(balance, 1_200);
    }

    #[test]
    fn balance_as_of_returns_none_without_any_anchor() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = test_account(&db);

        assert_eq!(
            db.balance_as_of(account_id, "2026-07-01").expect("balance_as_of"),
            None
        );
    }
}
