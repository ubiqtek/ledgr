//! Persistence for the Gap screen — combines the spend and income ledgers
//! per month. See doc/planning/plan.md, Delta: The Gap, Task 2.

use super::Db;
use crate::model::MonthlyGap;
use rusqlite::params;

impl Db {
    /// Total balance across "cash" accounts (`Current`/`Savings`) as of
    /// `date` (`YYYY-MM-DD`) — deliberately excludes `CreditCard` accounts,
    /// since their balance is already accounted for via the spend ledger's
    /// card-payment matching (Delta: Credit Card Transaction Import, Task
    /// 5); double-counting it here would misrepresent the drawdown. Skips
    /// any account with no balance anchor at all yet (`balance_as_of`
    /// returns `None`) rather than treating it as zero, since a real but
    /// un-anchored balance would understate the total.
    pub fn cash_balance_as_of(&self, date: &str) -> rusqlite::Result<i64> {
        let mut stmt = self.conn().prepare(
            "SELECT id FROM accounts WHERE account_type IN ('current', 'savings')",
        )?;
        let account_ids: Vec<crate::model::Id> = stmt
            .query_map(params![], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let mut total = 0;
        for account_id in account_ids {
            if let Some(balance) = self.balance_as_of(account_id, date)? {
                total += balance;
            }
        }
        Ok(total)
    }

    /// One row per calendar month that has spend and/or income entries,
    /// earliest first. A month present in only one ledger (e.g. spend but no
    /// income yet) still gets a row, with the other side `0` — a `LEFT JOIN`
    /// off a `UNION` of both ledgers' months rather than joining one ledger
    /// onto the other, so neither side can silently drop a month the other
    /// doesn't have.
    pub fn monthly_gap_totals(&self) -> rusqlite::Result<Vec<MonthlyGap>> {
        let mut stmt = self.conn().prepare(
            "WITH income AS (
                 SELECT substr(occurred_on, 1, 7) AS month,
                        SUM(amount_minor) AS income_minor,
                        SUM(CASE WHEN rule_name = 'employment_income' THEN amount_minor ELSE 0 END) AS salary_minor
                 FROM income_entries
                 GROUP BY month
             ),
             spend AS (
                 SELECT substr(occurred_on, 1, 7) AS month,
                        SUM(amount_minor) AS spend_minor
                 FROM spend_entries
                 GROUP BY month
             ),
             months AS (
                 SELECT month FROM income
                 UNION
                 SELECT month FROM spend
             )
             SELECT months.month,
                    COALESCE(income.income_minor, 0),
                    COALESCE(income.salary_minor, 0),
                    COALESCE(spend.spend_minor, 0)
             FROM months
             LEFT JOIN income ON income.month = months.month
             LEFT JOIN spend ON spend.month = months.month
             ORDER BY months.month ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let income_minor: i64 = row.get(1)?;
            let spend_minor: i64 = row.get(3)?;
            Ok(MonthlyGap {
                month: row.get(0)?,
                income_minor,
                salary_minor: row.get(2)?,
                spend_minor,
                gap_minor: income_minor + spend_minor,
            })
        })?;
        rows.collect()
    }
}
