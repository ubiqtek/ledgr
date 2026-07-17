mod accounts;
mod balances;
mod cards;
mod imports;
mod spend;
mod status;
mod transactions;

pub use status::AccountStatus;

use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

const SCHEMA: &str = include_str!("schema.sql");

/// Handle to the local database. Wraps a single SQLite connection.
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open (creating if needed) the database at `path` and apply the schema.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// Open a private in-memory database. Useful for tests.
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        conn.pragma_update(None, "foreign_keys", true)?;
        Self::migrate_statements_to_imports(&conn)?;
        Self::migrate_add_transactions_notes(&conn)?;
        let renamed_leg_shaped_transfer_entries =
            Self::migrate_rename_leg_shaped_transfer_entries(&conn)?;
        let renamed_narrow_pair_method_transfer_entries =
            Self::migrate_rename_narrow_pair_method_transfer_entries(&conn)?;
        conn.execute_batch(SCHEMA)?;
        if renamed_leg_shaped_transfer_entries {
            Self::migrate_merge_leg_shaped_transfer_entries(&conn)?;
        }
        if renamed_narrow_pair_method_transfer_entries {
            Self::migrate_copy_narrow_pair_method_transfer_entries(&conn)?;
        }
        Self::migrate_delete_legacy_transfer_links(&conn)?;
        Ok(Self { conn })
    }

    /// One-off migration for databases created before "statement" was
    /// renamed to "import" (see doc/domain/ubiquitous-language.md). Renames
    /// the `statements` table to `imports` and its referencing
    /// `statement_id` columns to `import_id`, so `SCHEMA`'s
    /// `CREATE TABLE IF NOT EXISTS imports` finds existing data rather than
    /// creating an empty table alongside the old one. No-op on a fresh or
    /// already-migrated database.
    fn migrate_statements_to_imports(conn: &Connection) -> rusqlite::Result<()> {
        let has_old_table: bool = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'statements'",
            [],
            |row| row.get::<_, i64>(0),
        )? > 0;
        if !has_old_table {
            return Ok(());
        }
        conn.execute_batch(
            "ALTER TABLE statements RENAME TO imports;
             ALTER TABLE transactions RENAME COLUMN statement_id TO import_id;
             ALTER TABLE balance_snapshots RENAME COLUMN statement_id TO import_id;",
        )
    }

    /// One-off migration for databases created before `notes` was added to
    /// `transactions` (see Credit Card Transaction Import in
    /// doc/planning/plan.md). `CREATE TABLE IF NOT EXISTS` never alters an
    /// existing table, so this adds the column by hand. No-op if it's
    /// already present.
    fn migrate_add_transactions_notes(conn: &Connection) -> rusqlite::Result<()> {
        let has_table: bool = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'transactions'",
            [],
            |row| row.get::<_, i64>(0),
        )? > 0;
        if !has_table {
            return Ok(());
        }
        let has_notes_column: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('transactions') WHERE name = 'notes'")?
            .exists([])?;
        if has_notes_column {
            return Ok(());
        }
        conn.execute_batch("ALTER TABLE transactions ADD COLUMN notes TEXT;")
    }

    /// One-off migration for databases whose `transfer_entries` table still
    /// has the earlier **one row per leg** shape (a `transaction_id`
    /// column, two rows per paired transfer, linked via
    /// `counterpart_transaction_id`) rather than the current **one row per
    /// real-world transfer** shape (`out_*`/`in_*` columns on a single row)
    /// — see doc/implementation-notes/transfer-ledger-history.md for why
    /// the shape changed (a "one row per transaction" table doesn't
    /// represent a transfer, which is inherently two transactions linked
    /// together). Detected by column presence, not `CHECK` constraint text
    /// (works regardless of which pairing-tier version the old table had).
    /// SQLite can't restructure a table's columns in place, so the old
    /// table is renamed aside here, letting `SCHEMA`'s
    /// `CREATE TABLE IF NOT EXISTS` create a fresh one in the new shape;
    /// `migrate_merge_leg_shaped_transfer_entries` then merges the old
    /// data across and drops the renamed table. Returns whether a rename
    /// happened, so `init` knows whether the merge step is needed. No-op
    /// (returns `false`) on a fresh database or one already in the new
    /// shape.
    fn migrate_rename_leg_shaped_transfer_entries(conn: &Connection) -> rusqlite::Result<bool> {
        let has_old_shape: bool = conn
            .prepare(
                "SELECT 1 FROM pragma_table_info('transfer_entries') WHERE name = 'transaction_id'",
            )?
            .exists([])?;
        if !has_old_shape {
            return Ok(false);
        }
        conn.execute_batch("ALTER TABLE transfer_entries RENAME TO transfer_entries_pre_leg_merge;")?;
        Ok(true)
    }

    /// Second half of `migrate_rename_leg_shaped_transfer_entries`: merges
    /// every row from the renamed-aside old (one-row-per-leg) table into
    /// the freshly-created `transfer_entries` (one-row-per-transfer),
    /// pairing each old row with its `counterpart_transaction_id` row (when
    /// present) into a single new row, and drops the old table. Must run
    /// after `SCHEMA` has (re-)created `transfer_entries`. Done in Rust
    /// rather than a single SQL statement — merging two old rows'
    /// `out_*`/`in_*` sides based on which is negative/positive is
    /// straightforward per-row logic, and this only ever runs once per
    /// database, over at most a few hundred rows.
    fn migrate_merge_leg_shaped_transfer_entries(conn: &Connection) -> rusqlite::Result<()> {
        struct OldLeg {
            transaction_id: i64,
            account_id: i64,
            occurred_on: String,
            amount_minor: i64,
            currency: String,
            description: String,
            counterpart_sort_code: String,
            counterpart_account_number: String,
            counterpart_account_id: Option<i64>,
            counterpart_transaction_id: Option<i64>,
            pair_method: Option<String>,
            pair_confidence: Option<f64>,
            classified_by: String,
            confidence: Option<f64>,
            rule_name: Option<String>,
            classified_at: String,
        }

        let legs: Vec<OldLeg> = conn
            .prepare(
                "SELECT transaction_id, account_id, occurred_on, amount_minor, currency,
                        description, counterpart_sort_code, counterpart_account_number,
                        counterpart_account_id, counterpart_transaction_id, pair_method,
                        pair_confidence, classified_by, confidence, rule_name, classified_at
                 FROM transfer_entries_pre_leg_merge
                 ORDER BY id",
            )?
            .query_map([], |row| {
                Ok(OldLeg {
                    transaction_id: row.get(0)?,
                    account_id: row.get(1)?,
                    occurred_on: row.get(2)?,
                    amount_minor: row.get(3)?,
                    currency: row.get(4)?,
                    description: row.get(5)?,
                    counterpart_sort_code: row.get(6)?,
                    counterpart_account_number: row.get(7)?,
                    counterpart_account_id: row.get(8)?,
                    counterpart_transaction_id: row.get(9)?,
                    pair_method: row.get(10)?,
                    pair_confidence: row.get(11)?,
                    classified_by: row.get(12)?,
                    confidence: row.get(13)?,
                    rule_name: row.get(14)?,
                    classified_at: row.get(15)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;

        let by_transaction_id: std::collections::HashMap<i64, &OldLeg> =
            legs.iter().map(|l| (l.transaction_id, l)).collect();
        let mut processed = std::collections::HashSet::new();

        for leg in &legs {
            if processed.contains(&leg.transaction_id) {
                continue;
            }
            processed.insert(leg.transaction_id);
            let counterpart = leg
                .counterpart_transaction_id
                .and_then(|id| by_transaction_id.get(&id).copied());
            if let Some(c) = counterpart {
                processed.insert(c.transaction_id);
            }

            let (out_leg, in_leg): (Option<&OldLeg>, Option<&OldLeg>) = if leg.amount_minor < 0 {
                (Some(leg), counterpart)
            } else {
                (counterpart, Some(leg))
            };

            let occurred_on = out_leg
                .or(in_leg)
                .map(|l| l.occurred_on.clone())
                .unwrap_or_default();
            let (out_transaction_id, out_account_id, out_sort, out_account_no, out_description) =
                match out_leg {
                    Some(l) => (
                        Some(l.transaction_id),
                        Some(l.account_id),
                        None,
                        None,
                        Some(l.description.clone()),
                    ),
                    None => {
                        let known = in_leg.expect("at least one side is always known");
                        (
                            None,
                            known.counterpart_account_id,
                            Some(known.counterpart_sort_code.clone()),
                            Some(known.counterpart_account_number.clone()),
                            None,
                        )
                    }
                };
            let (in_transaction_id, in_account_id, in_sort, in_account_no, in_description) =
                match in_leg {
                    Some(l) => (
                        Some(l.transaction_id),
                        Some(l.account_id),
                        None,
                        None,
                        Some(l.description.clone()),
                    ),
                    None => {
                        let known = out_leg.expect("at least one side is always known");
                        (
                            None,
                            known.counterpart_account_id,
                            Some(known.counterpart_sort_code.clone()),
                            Some(known.counterpart_account_number.clone()),
                            None,
                        )
                    }
                };
            let primary = out_leg.or(in_leg).expect("at least one side is always known");

            conn.execute(
                "INSERT INTO transfer_entries
                    (occurred_on, amount_minor, currency,
                     out_transaction_id, out_account_id, out_sort_code, out_account_number, out_description,
                     in_transaction_id, in_account_id, in_sort_code, in_account_number, in_description,
                     pair_method, pair_confidence, classified_by, confidence, rule_name, classified_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
                rusqlite::params![
                    occurred_on,
                    leg.amount_minor.abs(),
                    leg.currency,
                    out_transaction_id,
                    out_account_id,
                    out_sort,
                    out_account_no,
                    out_description,
                    in_transaction_id,
                    in_account_id,
                    in_sort,
                    in_account_no,
                    in_description,
                    primary.pair_method,
                    primary.pair_confidence,
                    primary.classified_by,
                    primary.confidence,
                    primary.rule_name,
                    primary.classified_at,
                ],
            )?;
        }

        conn.execute_batch("DROP TABLE transfer_entries_pre_leg_merge;")?;
        Ok(())
    }

    /// One-off migration for databases whose `transfer_entries.pair_method`
    /// `CHECK` constraint predates `'credit_card_payment_match'` (Delta:
    /// Transfer Ledger, Task 4 — migrating credit card payment pairing off
    /// the legacy `transaction_links` mechanism). SQLite can't `ALTER` a
    /// `CHECK` constraint, so — same pattern as
    /// `migrate_rename_leg_shaped_transfer_entries` above — the existing
    /// table is renamed aside here, letting `SCHEMA`'s
    /// `CREATE TABLE IF NOT EXISTS` create a fresh one with the widened
    /// `CHECK`; `migrate_copy_narrow_pair_method_transfer_entries` then
    /// copies every row across unchanged (no shape change this time) and
    /// drops the renamed table. Detected by inspecting the stored `CREATE
    /// TABLE` text directly, since column presence can't distinguish a
    /// `CHECK`-only change. No-op on a fresh database or one already
    /// migrated.
    fn migrate_rename_narrow_pair_method_transfer_entries(conn: &Connection) -> rusqlite::Result<bool> {
        let existing_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'transfer_entries'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let Some(existing_sql) = existing_sql else {
            return Ok(false);
        };
        if existing_sql.contains("credit_card_payment_match") {
            return Ok(false);
        }
        conn.execute_batch(
            "ALTER TABLE transfer_entries RENAME TO transfer_entries_pre_pair_method_widen;",
        )?;
        Ok(true)
    }

    /// Second half of `migrate_rename_narrow_pair_method_transfer_entries`:
    /// copies every row from the renamed-aside table into the freshly
    /// widened `transfer_entries` unchanged, then drops the old table. Must
    /// run after `SCHEMA` has (re-)created `transfer_entries`.
    fn migrate_copy_narrow_pair_method_transfer_entries(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "INSERT INTO transfer_entries
                (id, occurred_on, amount_minor, currency,
                 out_transaction_id, out_account_id, out_sort_code, out_account_number, out_description,
                 in_transaction_id, in_account_id, in_sort_code, in_account_number, in_description,
                 pair_method, pair_confidence, classified_by, confidence, rule_name, classified_at)
             SELECT id, occurred_on, amount_minor, currency,
                    out_transaction_id, out_account_id, out_sort_code, out_account_number, out_description,
                    in_transaction_id, in_account_id, in_sort_code, in_account_number, in_description,
                    pair_method, pair_confidence, classified_by, confidence, rule_name, classified_at
             FROM transfer_entries_pre_pair_method_widen;
             DROP TABLE transfer_entries_pre_pair_method_widen;",
        )
    }

    /// One-off cleanup: deletes any `transaction_links` rows left over from
    /// the retired `relation='transfer'` credit card payment matching
    /// (Delta: Transfer Ledger, Task 4 — see
    /// doc/implementation-notes/transfer-ledger-critique.md, "credit card
    /// payment matching is an internal transfer that never migrated to
    /// `transfer_entries`"). Every such payment now has its own
    /// `transfer_entries` row instead, so these are pure duplicates. Run
    /// unconditionally on every open — cheap (indexed, small table) and a
    /// no-op once the legacy rows are gone, so no separate "already
    /// migrated" check is needed.
    fn migrate_delete_legacy_transfer_links(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "DELETE FROM transaction_links WHERE relation = 'transfer'",
            [],
        )?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_in_memory_and_applies_schema() {
        let db = Db::open_in_memory().expect("open in-memory db");
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'accounts'",
                [],
                |row| row.get(0),
            )
            .expect("query sqlite_master");
        assert_eq!(count, 1);
    }

    #[test]
    fn schema_is_idempotent() {
        // Opening twice against the same file must not fail on re-applying
        // `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF NOT EXISTS`.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ledgr.db");
        Db::open(&path).expect("first open");
        Db::open(&path).expect("second open");
    }

    /// A database whose `transfer_entries` table still has the earlier
    /// one-row-per-leg shape must be able to open, keep its existing data
    /// (merged into the new one-row-per-transfer shape), and accept a
    /// write using the new shape's columns — `CREATE TABLE IF NOT EXISTS`
    /// alone can't restructure an already-existing table's columns, so
    /// this exercises `migrate_rename_leg_shaped_transfer_entries` +
    /// `migrate_merge_leg_shaped_transfer_entries`.
    #[test]
    fn migrates_a_leg_shaped_transfer_entries_table_into_the_per_transfer_shape() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ledgr.db");

        // Real accounts/transactions rows for transfer_entries' foreign keys
        // to reference — created via a normal `Db::open` first, then the
        // table is dropped and recreated in the old one-row-per-leg shape
        // below, matching what the real database was in before this
        // session's schema rework landed. Three old rows: one paired pair
        // (transactions 1 and 2, mutually linked) and one unpaired leg
        // (transaction 3).
        {
            let db = Db::open(&path).expect("initial open to seed accounts/transactions");
            db.conn()
                .execute_batch(
                    "INSERT INTO accounts (id, name, account_type, currency)
                        VALUES (1, 'Jims Premier Account', 'current', 'GBP'),
                               (2, 'Bills Account', 'current', 'GBP');
                     INSERT INTO transactions
                        (id, account_id, posted_at, amount_minor, currency, description)
                     VALUES
                        (1, 1, '2026-01-01', -100, 'GBP', 'out leg'),
                        (2, 2, '2026-01-01', 100, 'GBP', 'in leg'),
                        (3, 1, '2026-01-05', -50, 'GBP', 'unpaired out leg');",
                )
                .expect("seed accounts/transactions");
            db.conn()
                .execute_batch(
                    "DROP TABLE transfer_entries;
                     CREATE TABLE transfer_entries (
                        id INTEGER PRIMARY KEY,
                        transaction_id INTEGER NOT NULL UNIQUE
                            REFERENCES transactions(id) ON DELETE CASCADE,
                        account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
                        occurred_on TEXT NOT NULL,
                        amount_minor INTEGER NOT NULL,
                        currency TEXT NOT NULL,
                        description TEXT NOT NULL,
                        counterpart_sort_code TEXT NOT NULL,
                        counterpart_account_number TEXT NOT NULL,
                        counterpart_account_id INTEGER,
                        counterpart_transaction_id INTEGER,
                        pair_method TEXT CHECK (pair_method IN
                            ('description_match', 'amount_date_match')),
                        pair_confidence REAL,
                        classified_by TEXT NOT NULL,
                        confidence REAL,
                        rule_name TEXT,
                        classified_at TEXT NOT NULL
                    );
                    INSERT INTO transfer_entries
                        (transaction_id, account_id, occurred_on, amount_minor, currency,
                         description, counterpart_sort_code, counterpart_account_number,
                         counterpart_account_id, counterpart_transaction_id,
                         pair_method, pair_confidence, classified_by, classified_at)
                     VALUES
                        (1, 1, '2026-01-01', -100, 'GBP', 'out leg', '222222', '2',
                         2, 2, 'description_match', 0.9, 'rule', '2026-01-01T00:00:00.000Z'),
                        (2, 2, '2026-01-01', 100, 'GBP', 'in leg', '111111', '1',
                         1, 1, 'description_match', 0.9, 'rule', '2026-01-01T00:00:00.000Z'),
                        (3, 1, '2026-01-05', -50, 'GBP', 'unpaired out leg', '999999', '99999999',
                         NULL, NULL, NULL, NULL, 'rule', '2026-01-05T00:00:00.000Z');",
                )
                .expect("seed old-shape transfer_entries table");
        }

        let db = Db::open(&path).expect("open database with a leg-shaped transfer_entries table");

        let row_count: i64 = db
            .conn()
            .query_row("SELECT count(*) FROM transfer_entries", [], |row| {
                row.get(0)
            })
            .expect("query transfer_entries");
        assert_eq!(
            row_count, 2,
            "the paired legs must merge into one row, plus one row for the unpaired leg"
        );

        let (out_id, in_id, amount): (i64, i64, i64) = db
            .conn()
            .query_row(
                "SELECT out_transaction_id, in_transaction_id, amount_minor
                 FROM transfer_entries WHERE out_transaction_id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("query the merged pair");
        assert_eq!((out_id, in_id, amount), (1, 2, 100));

        let (out_id, in_id): (Option<i64>, Option<i64>) = db
            .conn()
            .query_row(
                "SELECT out_transaction_id, in_transaction_id
                 FROM transfer_entries WHERE out_transaction_id = 3",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("query the unpaired leg");
        assert_eq!((out_id, in_id), (Some(3), None), "the unpaired leg's in side stays empty");

        db.conn()
            .execute(
                "UPDATE transfer_entries SET pair_method = 'self_reference_match' WHERE out_transaction_id = 3",
                [],
            )
            .expect("new table must accept 'self_reference_match'");
    }
}
