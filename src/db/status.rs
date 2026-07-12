use super::Db;
use crate::model::{Account, AccountType};

/// Summary of one account for `ledgr status`: balance, transaction count,
/// the date range covered, and when it was last imported into.
///
/// `balance_minor` comes from the most recent balance anchor (e.g. OFX
/// `LEDGERBAL`), not a sum of imported transactions — an import's
/// transaction list often doesn't reach back to account opening, so summing
/// it understates/misstates the real balance. See `Db::balance_as_of`. For a
/// `CreditCard` account it's negated (a card statement reports the amount
/// owed as a positive figure, but that's a liability, not an asset) so
/// consumers get assets-positive/liabilities-negative for free, per ADR 0007.
#[derive(Debug, Clone)]
pub struct AccountStatus {
    pub account: Account,
    pub transaction_count: i64,
    pub balance_minor: Option<i64>,
    pub balance_as_of: Option<String>,
    pub earliest_transaction: Option<String>,
    pub latest_transaction: Option<String>,
    pub last_imported_at: Option<String>,
    /// Last 4 digits of the account's current card number, for account
    /// types with no bank sort code/account number (e.g. `CreditCard`,
    /// whose identity comes from `account_card_numbers` instead).
    pub card_last4: Option<String>,
}

impl Db {
    /// One row per account, ordered by name, each carrying its own
    /// transaction-count/balance/date-range/last-import summary.
    pub fn account_statuses(&self) -> rusqlite::Result<Vec<AccountStatus>> {
        let mut stmt = self.conn().prepare(
            "SELECT a.id, a.name, a.institution, a.account_type, a.currency,
                    a.sort_code, a.account_number,
                    COUNT(t.id) AS tx_count,
                    MIN(t.posted_at),
                    MAX(t.posted_at),
                    (SELECT MAX(s.imported_at) FROM imports s WHERE s.account_id = a.id)
             FROM accounts a
             LEFT JOIN transactions t ON t.account_id = a.id
             GROUP BY a.id
             ORDER BY a.name",
        )?;
        let rows = stmt.query_map([], |row| {
            let account_type_str: String = row.get(3)?;
            Ok((
                Account {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    institution: row.get(2)?,
                    account_type: AccountType::parse(&account_type_str)
                        .unwrap_or(AccountType::Other),
                    currency: row.get(4)?,
                    sort_code: row.get(5)?,
                    account_number: row.get(6)?,
                },
                row.get::<_, i64>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })?;

        rows.map(|row| {
            let (
                account,
                transaction_count,
                earliest_transaction,
                latest_transaction,
                last_imported_at,
            ) = row?;
            let (balance_minor, balance_as_of) = match self.latest_balance_snapshot(account.id)? {
                Some((balance, as_of)) => {
                    let balance = if account.account_type == AccountType::CreditCard {
                        -balance
                    } else {
                        balance
                    };
                    (Some(balance), Some(as_of))
                }
                None => (None, None),
            };
            let card_last4 = if account.account_number.is_none() {
                self.card_number_history(account.id)?.into_iter().next()
            } else {
                None
            };
            Ok(AccountStatus {
                account,
                transaction_count,
                balance_minor,
                balance_as_of,
                earliest_transaction,
                latest_transaction,
                last_imported_at,
                card_last4,
            })
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewAccount, NewTransaction};

    #[test]
    fn status_for_account_with_no_transactions() {
        let db = Db::open_in_memory().expect("open db");
        db.insert_account(&NewAccount {
            name: "Empty Account".into(),
            institution: None,
            account_type: AccountType::Current,
            currency: "GBP".into(),
            sort_code: None,
            account_number: None,
        })
        .expect("insert account");

        let statuses = db.account_statuses().expect("account statuses");
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].transaction_count, 0);
        assert_eq!(statuses[0].balance_minor, None);
        assert_eq!(statuses[0].earliest_transaction, None);
        assert_eq!(statuses[0].last_imported_at, None);
    }

    #[test]
    fn status_reports_balance_from_the_latest_anchor_and_finds_date_range() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
            })
            .expect("insert account");

        for (date, amount) in [("2026-01-05", -500), ("2026-03-10", 2000)] {
            db.insert_transaction(&NewTransaction {
                account_id,
                import_id: None,
                posted_at: date.into(),
                amount_minor: amount,
                currency: "GBP".into(),
                description: "test".into(),
                raw_description: None,
                trn_type: None,
                external_id: None,
                notes: None,
            })
            .expect("insert transaction");
        }
        db.insert_balance_snapshot(account_id, None, 1_500, "2026-03-10")
            .expect("insert snapshot");

        let statuses = db.account_statuses().expect("account statuses");
        assert_eq!(statuses.len(), 1);
        let status = &statuses[0];
        assert_eq!(status.transaction_count, 2);
        assert_eq!(status.balance_minor, Some(1_500));
        assert_eq!(status.balance_as_of.as_deref(), Some("2026-03-10"));
        assert_eq!(status.earliest_transaction.as_deref(), Some("2026-01-05"));
        assert_eq!(status.latest_transaction.as_deref(), Some("2026-03-10"));
    }

    #[test]
    fn status_reports_last_imported_at_from_imports() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
            })
            .expect("insert account");

        db.insert_import(account_id, "/tmp/a.ofx", "hash-a", None, None)
            .expect("insert import");

        let statuses = db.account_statuses().expect("account statuses");
        assert!(statuses[0].last_imported_at.is_some());
    }
}
