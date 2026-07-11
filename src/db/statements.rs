use super::Db;
use crate::model::Id;
use rusqlite::{params, OptionalExtension};

impl Db {
    /// Records that a statement file has been imported. Returns `Ok(None)`
    /// without inserting if a statement with the same `file_hash` already
    /// exists, so re-running an import over the same file is a no-op.
    pub fn insert_statement(
        &self,
        account_id: Id,
        source_path: &str,
        file_hash: &str,
        period_start: Option<&str>,
        period_end: Option<&str>,
    ) -> rusqlite::Result<Option<Id>> {
        if self.find_statement_by_hash(file_hash)?.is_some() {
            return Ok(None);
        }
        self.conn().execute(
            "INSERT INTO statements (account_id, source_path, file_hash, period_start, period_end)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![account_id, source_path, file_hash, period_start, period_end],
        )?;
        Ok(Some(self.conn().last_insert_rowid()))
    }

    pub fn find_statement_by_hash(&self, file_hash: &str) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT id FROM statements WHERE file_hash = ?1",
                params![file_hash],
                |row| row.get(0),
            )
            .optional()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AccountType, NewAccount};

    fn test_account(db: &Db) -> Id {
        db.insert_account(&NewAccount {
            name: "Current Account".into(),
            institution: None,
            account_type: AccountType::Current,
            currency: "GBP".into(),
        })
        .expect("insert account")
    }

    #[test]
    fn insert_statement_then_find_by_hash() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = test_account(&db);

        let id = db
            .insert_statement(account_id, "/tmp/statement.ofx", "hash1", None, None)
            .expect("insert statement")
            .expect("not a duplicate");

        assert_eq!(db.find_statement_by_hash("hash1").expect("find"), Some(id));
        assert_eq!(db.find_statement_by_hash("missing").expect("find"), None);
    }

    #[test]
    fn re_importing_the_same_hash_is_a_no_op() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = test_account(&db);

        db.insert_statement(account_id, "/tmp/statement.ofx", "hash1", None, None)
            .expect("insert statement")
            .expect("not a duplicate");

        let second = db
            .insert_statement(account_id, "/tmp/statement.ofx", "hash1", None, None)
            .expect("insert statement");
        assert_eq!(second, None);
    }
}
