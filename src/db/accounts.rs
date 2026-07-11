use super::Db;
use crate::model::{Account, AccountType, Id, NewAccount};
use rusqlite::{params, OptionalExtension};

impl Db {
    pub fn insert_account(&self, new: &NewAccount) -> rusqlite::Result<Id> {
        self.conn().execute(
            "INSERT INTO accounts (name, institution, account_type, currency)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                new.name,
                new.institution,
                new.account_type.as_str(),
                new.currency
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    pub fn get_account(&self, id: Id) -> rusqlite::Result<Option<Account>> {
        self.conn()
            .query_row(
                "SELECT id, name, institution, account_type, currency
                 FROM accounts WHERE id = ?1",
                params![id],
                Self::row_to_account,
            )
            .optional()
    }

    pub fn list_accounts(&self) -> rusqlite::Result<Vec<Account>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, name, institution, account_type, currency
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
                account_type: AccountType::Checking,
                currency: "GBP".into(),
            })
            .expect("insert account");

        let fetched = db.get_account(id).expect("get account").expect("found");
        assert_eq!(fetched.name, "Current Account");
        assert_eq!(fetched.account_type, AccountType::Checking);

        let all = db.list_accounts().expect("list accounts");
        assert_eq!(all.len(), 1);
    }
}
