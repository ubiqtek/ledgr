use super::Db;
use crate::model::{Account, AccountType, Id, NewAccount};
use rusqlite::{params, OptionalExtension};

impl Db {
    pub fn insert_account(&self, new: &NewAccount) -> rusqlite::Result<Id> {
        self.conn().execute(
            "INSERT INTO accounts (name, institution, account_type, currency, sort_code, account_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                new.name,
                new.institution,
                new.account_type.as_str(),
                new.currency,
                new.sort_code,
                new.account_number,
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Returns the existing account matching `institution`/`name`, or
    /// creates one from `new` if none exists yet. Backfills `sort_code`/
    /// `account_number` on the existing row if `new` carries them and the
    /// existing row doesn't yet (e.g. an account created before those
    /// columns existed).
    pub fn find_or_create_account(&self, new: &NewAccount) -> rusqlite::Result<Id> {
        let existing = self
            .conn()
            .query_row(
                "SELECT id FROM accounts WHERE institution IS ?1 AND name = ?2",
                params![new.institution, new.name],
                |row| row.get(0),
            )
            .optional()?;
        match existing {
            Some(id) => {
                self.conn().execute(
                    "UPDATE accounts SET sort_code = COALESCE(sort_code, ?1),
                                          account_number = COALESCE(account_number, ?2)
                     WHERE id = ?3",
                    params![new.sort_code, new.account_number, id],
                )?;
                Ok(id)
            }
            None => self.insert_account(new),
        }
    }

    /// Basic single-account lookup — currently only exercised by tests;
    /// kept as a natural CRUD primitive for whatever next needs it (e.g. a
    /// future web frontend, per ADR 0003).
    #[allow(dead_code)]
    pub fn get_account(&self, id: Id) -> rusqlite::Result<Option<Account>> {
        self.conn()
            .query_row(
                "SELECT id, name, institution, account_type, currency, sort_code, account_number
                 FROM accounts WHERE id = ?1",
                params![id],
                Self::row_to_account,
            )
            .optional()
    }

    pub fn list_accounts(&self) -> rusqlite::Result<Vec<Account>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, name, institution, account_type, currency, sort_code, account_number
             FROM accounts ORDER BY name",
        )?;
        let rows = stmt.query_map([], Self::row_to_account)?;
        rows.collect()
    }

    fn row_to_account(row: &rusqlite::Row) -> rusqlite::Result<Account> {
        let account_type_str: String = row.get(3)?;
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            institution: row.get(2)?,
            account_type: AccountType::parse(&account_type_str).unwrap_or(AccountType::Other),
            currency: row.get(4)?,
            sort_code: row.get(5)?,
            account_number: row.get(6)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_list_accounts() {
        let db = Db::open_in_memory().expect("open db");
        let id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: Some("Some Bank".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
            })
            .expect("insert account");

        let fetched = db.get_account(id).expect("get account").expect("found");
        assert_eq!(fetched.name, "Current Account");
        assert_eq!(fetched.account_type, AccountType::Current);

        let all = db.list_accounts().expect("list accounts");
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn find_or_create_account_reuses_a_matching_account() {
        let db = Db::open_in_memory().expect("open db");
        let new = NewAccount {
            name: "Barclays Current Account".into(),
            institution: Some("Barclays".into()),
            account_type: AccountType::Current,
            currency: "GBP".into(),
            sort_code: None,
            account_number: None,
        };

        let first = db.find_or_create_account(&new).expect("first call");
        let second = db.find_or_create_account(&new).expect("second call");

        assert_eq!(first, second);
        assert_eq!(db.list_accounts().expect("list accounts").len(), 1);
    }
}
