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

    /// Total row count in `spend_entries`, for `ledgr status`'s Spend Ledger
    /// section.
    pub fn spend_ledger_summary(&self) -> rusqlite::Result<SpendLedgerSummary> {
        let entries = self
            .conn()
            .query_row("SELECT COUNT(*) FROM spend_entries", [], |row| row.get(0))?;
        Ok(SpendLedgerSummary { entries })
    }

    /// Row/pairing counts across `transfer_entries`, for `ledgr status`'s
    /// Transfer Ledger section. `card_payments_matched`/
    /// `card_payments_unmatched` break out the credit-card-payment rule
    /// (`rule_name = 'credit_card_payment'`) specifically — matched means its
    /// card-side counterpart has been found (`in_transaction_id` set),
    /// unmatched means it's still an open one-sided leg (see
    /// `open_card_payment_entries`), e.g. because the matching card statement
    /// hasn't been imported yet.
    pub fn transfer_ledger_summary(&self) -> rusqlite::Result<TransferLedgerSummary> {
        self.conn().query_row(
            "SELECT
                 COUNT(*),
                 COUNT(*) FILTER (WHERE out_transaction_id IS NOT NULL AND in_transaction_id IS NOT NULL),
                 COUNT(*) FILTER (WHERE out_transaction_id IS NULL OR in_transaction_id IS NULL),
                 COUNT(*) FILTER (WHERE rule_name = 'credit_card_payment' AND in_transaction_id IS NOT NULL),
                 COUNT(*) FILTER (WHERE rule_name = 'credit_card_payment' AND in_transaction_id IS NULL)
             FROM transfer_entries",
            [],
            |row| {
                Ok(TransferLedgerSummary {
                    entries: row.get(0)?,
                    paired: row.get(1)?,
                    unpaired: row.get(2)?,
                    card_payments_matched: row.get(3)?,
                    card_payments_unmatched: row.get(4)?,
                })
            },
        )
    }

    /// The missing side's decoded `(sort_code, account_number)` for every
    /// unpaired `transfer_entries` row — the raw digits needed to tell
    /// "counterpart is a Reference Household Account, permanently unpairable
    /// by design" (see `Config::household_account_matches`) apart from "no
    /// counterpart found". `None` for a credit-card-payment leg, which
    /// carries no `NAME`-decoded counterpart at all (see
    /// `derive::run_derivation`'s card payment handling).
    pub fn unpaired_transfer_counterparties(
        &self,
    ) -> rusqlite::Result<Vec<(Option<String>, Option<String>)>> {
        let mut stmt = self.conn().prepare(
            "SELECT
                 CASE WHEN out_transaction_id IS NULL THEN NULLIF(out_sort_code, '') ELSE NULLIF(in_sort_code, '') END,
                 CASE WHEN out_transaction_id IS NULL THEN NULLIF(out_account_number, '') ELSE NULLIF(in_account_number, '') END
             FROM transfer_entries
             WHERE out_transaction_id IS NULL OR in_transaction_id IS NULL",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }
}

/// See `Db::spend_ledger_summary`.
#[derive(Debug, Clone)]
pub struct SpendLedgerSummary {
    pub entries: i64,
}

/// See `Db::transfer_ledger_summary`.
#[derive(Debug, Clone)]
pub struct TransferLedgerSummary {
    pub entries: i64,
    pub paired: i64,
    pub unpaired: i64,
    pub card_payments_matched: i64,
    pub card_payments_unmatched: i64,
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

    #[test]
    fn spend_ledger_summary_counts_entries() {
        let db = Db::open_in_memory().expect("open db");
        assert_eq!(db.spend_ledger_summary().expect("summary").entries, 0);

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
        let transaction_id = db
            .insert_transaction(&NewTransaction {
                account_id,
                import_id: None,
                posted_at: "2026-06-01".into(),
                amount_minor: -1_000,
                currency: "GBP".into(),
                description: "COFFEE SHOP".into(),
                raw_description: None,
                trn_type: None,
                external_id: None,
                notes: None,
            })
            .expect("insert transaction")
            .expect("not a duplicate");

        db.insert_spend_entry_with_source(
            &crate::model::NewSpendEntry {
                occurred_on: "2026-06-01".into(),
                amount_minor: -1_000,
                currency: "GBP".into(),
                counterparty: Some("Coffee Shop".into()),
                description: "COFFEE SHOP".into(),
                note: None,
                category_id: None,
                refunds_spend_entry_id: None,
                classified_by: crate::model::ClassifiedBy::Rule,
                confidence: Some(0.4),
                rule_name: Some("fallback".into()),
            },
            transaction_id,
        )
        .expect("insert spend entry");

        assert_eq!(db.spend_ledger_summary().expect("summary").entries, 1);
    }

    #[test]
    fn transfer_ledger_summary_counts_paired_unpaired_and_card_payments() {
        let db = Db::open_in_memory().expect("open db");
        let summary = db.transfer_ledger_summary().expect("summary");
        assert_eq!(summary.entries, 0);
        assert_eq!(summary.paired, 0);
        assert_eq!(summary.unpaired, 0);
        assert_eq!(summary.card_payments_matched, 0);
        assert_eq!(summary.card_payments_unmatched, 0);

        let current_account = db
            .insert_account(&NewAccount {
                name: "Jims Premier Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209912".into()),
                account_number: Some("12345678".into()),
            })
            .expect("insert current account");

        // An unmatched card payment: a bank-side debit with no credit card
        // account registered yet to pair it against — see
        // `crate::derive::run_derivation_records_an_unmatched_card_payment_as_an_open_transfer_entry`
        // for the equivalent full derivation-path test.
        db.insert_transaction(&NewTransaction {
            account_id: current_account,
            import_id: None,
            posted_at: "2026-06-01".into(),
            amount_minor: -29581,
            currency: "GBP".into(),
            description: "MR JAMES BARRITT 49291328548900".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert bank-side payment");

        crate::derive::run_derivation(&db, &[]).expect("derive");

        let summary = db.transfer_ledger_summary().expect("summary");
        assert_eq!(summary.entries, 1);
        assert_eq!(summary.paired, 0);
        assert_eq!(summary.unpaired, 1);
        assert_eq!(summary.card_payments_matched, 0);
        assert_eq!(summary.card_payments_unmatched, 1);

        // The credit card statement arrives later, completing the pair.
        let credit_card_account = db
            .insert_account(&NewAccount {
                name: "Barclaycard".into(),
                institution: Some("Barclaycard".into()),
                account_type: AccountType::CreditCard,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
            })
            .expect("insert credit card account");
        db.insert_transaction(&NewTransaction {
            account_id: credit_card_account,
            import_id: None,
            posted_at: "2026-06-01".into(),
            amount_minor: 29581,
            currency: "GBP".into(),
            description: "PAYMENT, THANK YOU".into(),
            raw_description: None,
            trn_type: Some("PAYMENT".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert card-side payment");

        crate::derive::run_derivation(&db, &[]).expect("derive");

        let summary = db.transfer_ledger_summary().expect("summary");
        assert_eq!(summary.entries, 1);
        assert_eq!(summary.paired, 1);
        assert_eq!(summary.unpaired, 0);
        assert_eq!(summary.card_payments_matched, 1);
        assert_eq!(summary.card_payments_unmatched, 0);
    }
}
