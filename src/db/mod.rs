mod accounts;
mod balances;
mod cards;
mod imports;
mod spend;
mod status;
mod transactions;

pub use status::AccountStatus;

use rusqlite::Connection;
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
        conn.execute_batch(SCHEMA)?;
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
}
