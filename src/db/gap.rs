//! Persistence for the Gap screen — combines the spend and income ledgers
//! per month. See doc/planning/plan.md, Delta: The Gap, Task 2.

use super::Db;
use crate::model::MonthlyGap;
use chrono::NaiveDate;
use rusqlite::params;

/// `"2026-03"` -> (end of February, end of March) — the balance-as-of dates
/// bracketing the month, i.e. its opening and closing cash position. Shared
/// with `app::open_selected_gap_month`, which needs the same two dates to
/// fetch the per-account breakdown behind a `cash_start_minor`/
/// `cash_end_minor` total.
pub(crate) fn month_bounds(month: &str) -> (String, String) {
    let year: i32 = month[..4].parse().expect("valid year");
    let m: u32 = month[5..7].parse().expect("valid month");
    let first_of_month = NaiveDate::from_ymd_opt(year, m, 1).expect("valid date");
    let month_start = first_of_month
        .pred_opt()
        .expect("month always has a preceding day");
    let month_end = if m == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, m + 1, 1)
    }
    .expect("valid date")
    .pred_opt()
    .expect("month always has a preceding day");
    (
        month_start.format("%Y-%m-%d").to_string(),
        month_end.format("%Y-%m-%d").to_string(),
    )
}

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

    /// Per-account breakdown behind `cash_balance_as_of` — same accounts,
    /// same exclusions, same "skip if no anchor yet" rule, but returned one
    /// row per account instead of summed, for the Gap month drill-down
    /// (`Screen::GapMonth`) where the user wants to see *which* account a
    /// cash figure is made up of.
    pub fn cash_balances_by_account_as_of(
        &self,
        date: &str,
    ) -> rusqlite::Result<Vec<(crate::model::Id, i64)>> {
        let mut stmt = self.conn().prepare(
            "SELECT id FROM accounts WHERE account_type IN ('current', 'savings')",
        )?;
        let account_ids: Vec<crate::model::Id> = stmt
            .query_map(params![], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let mut balances = Vec::new();
        for account_id in account_ids {
            if let Some(balance) = self.balance_as_of(account_id, date)? {
                balances.push((account_id, balance));
            }
        }
        Ok(balances)
    }

    /// One row per calendar month that has spend and/or income entries,
    /// earliest first. A month present in only one ledger (e.g. spend but no
    /// income yet) still gets a row, with the other side `0` — a `LEFT JOIN`
    /// off a `UNION` of both ledgers' months rather than joining one ledger
    /// onto the other, so neither side can silently drop a month the other
    /// doesn't have.
    ///
    /// Also fills in each row's `cash_start_minor`/`cash_end_minor`/
    /// `untracked_minor` — one `cash_balance_as_of` call per month boundary,
    /// since that reconstruction isn't expressible as a plain SQL query (it
    /// walks `balance_snapshots` per account in Rust). Fine at today's data
    /// volume (a handful of months); would need caching if this ever ran
    /// per-keystroke rather than once per screen load.
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
        let rows: Vec<(String, i64, i64, i64)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<rusqlite::Result<_>>()?;

        rows.into_iter()
            .map(|(month, income_minor, salary_minor, spend_minor)| {
                let gap_minor = income_minor + spend_minor;
                let (month_start, month_end) = month_bounds(&month);
                let cash_start_minor = self.cash_balance_as_of(&month_start)?;
                let cash_end_minor = self.cash_balance_as_of(&month_end)?;
                let untracked_minor = (cash_end_minor - cash_start_minor) - gap_minor;
                Ok(MonthlyGap {
                    month,
                    income_minor,
                    salary_minor,
                    spend_minor,
                    gap_minor,
                    cash_start_minor,
                    cash_end_minor,
                    untracked_minor,
                })
            })
            .collect()
    }
}
