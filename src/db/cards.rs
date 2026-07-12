//! Credit card account resolution. See
//! doc/kb/barclaycard/pdf-export-structure.md: a card statement export
//! carries no stable account identity — only a last-4-digits masked number
//! that changes on reissue — so this can't reuse `find_or_create_account`'s
//! institution+name matching (a reissue would just spawn a new account
//! every time). Instead each last4 ever seen is recorded per account, and a
//! fresh last4 with no match becomes a new account until a human links it
//! to an existing one (`link_card_number`) — no automatic reissue guessing.

use super::Db;
use crate::model::{AccountType, CardIdentity, Id, NewAccount};
use rusqlite::{params, OptionalExtension};

impl Db {
    /// Resolves a card identity to an account: reuses the account this
    /// last4 has been seen on before, or creates a new one and records the
    /// last4 as its first entry.
    pub fn find_or_create_credit_card_account(&self, card: &CardIdentity) -> rusqlite::Result<Id> {
        if let Some(id) = self.find_account_by_card_last4(&card.last4)? {
            return Ok(id);
        }

        let name = format!(
            "{} {} (...{})",
            card.institution, card.product_label, card.last4
        );
        let id = self.insert_account(&NewAccount {
            name,
            institution: Some(card.institution.clone()),
            account_type: AccountType::CreditCard,
            currency: card.currency.clone(),
            sort_code: None,
            account_number: None,
        })?;
        self.record_card_number(id, &card.last4)?;
        Ok(id)
    }

    /// Account, if any, that this last4 has ever been recorded against.
    pub fn find_account_by_card_last4(&self, last4: &str) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT account_id FROM account_card_numbers WHERE last4 = ?1",
                params![last4],
                |row| row.get(0),
            )
            .optional()
    }

    /// Records a last4 as belonging to `account_id` — reassigning it away
    /// from whatever account (if any) it was previously recorded against.
    /// A no-op if already recorded against this account.
    pub fn record_card_number(&self, account_id: Id, last4: &str) -> rusqlite::Result<()> {
        self.conn().execute(
            "INSERT INTO account_card_numbers (account_id, last4) VALUES (?1, ?2)
             ON CONFLICT (last4) DO UPDATE SET account_id = excluded.account_id",
            params![account_id, last4],
        )?;
        Ok(())
    }

    /// All last4s ever seen for an account, most recent first — the
    /// current card number is `card_number_history(id)[0]`. Ties on
    /// `first_seen` (SQLite's millisecond timestamp resolution is coarse
    /// enough for two inserts in quick succession to collide) are broken by
    /// `id DESC`, which reflects true insertion order regardless.
    pub fn card_number_history(&self, account_id: Id) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn().prepare(
            "SELECT last4 FROM account_card_numbers
             WHERE account_id = ?1
             ORDER BY first_seen DESC, id DESC",
        )?;
        let rows = stmt.query_map(params![account_id], |row| row.get(0))?;
        rows.collect()
    }

    /// Human-confirmed reissue: links a newly observed last4 to an
    /// already-known account, so future imports carrying that last4 match
    /// this account instead of spawning a new one. Deliberately manual —
    /// nothing in a card statement export ties an old number to a new one
    /// (see the KB article), so this is never inferred automatically.
    pub fn link_card_number(&self, account_id: Id, last4: &str) -> rusqlite::Result<()> {
        self.record_card_number(account_id, last4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_card(last4: &str) -> CardIdentity {
        CardIdentity {
            institution: "Barclaycard".into(),
            product_label: "Rewards".into(),
            last4: last4.into(),
            currency: "GBP".into(),
        }
    }

    #[test]
    fn creates_a_new_account_for_a_never_seen_last4() {
        let db = Db::open_in_memory().expect("open db");
        let id = db
            .find_or_create_credit_card_account(&sample_card("0002"))
            .expect("find_or_create_credit_card_account");

        let account = db.get_account(id).expect("get_account").expect("exists");
        assert_eq!(account.name, "Barclaycard Rewards (...0002)");
        assert_eq!(account.account_type, AccountType::CreditCard);
        assert_eq!(db.card_number_history(id).expect("history"), vec!["0002"]);
    }

    #[test]
    fn reimporting_the_same_last4_reuses_the_same_account() {
        let db = Db::open_in_memory().expect("open db");
        let first = db
            .find_or_create_credit_card_account(&sample_card("0002"))
            .expect("first call");
        let second = db
            .find_or_create_credit_card_account(&sample_card("0002"))
            .expect("second call");

        assert_eq!(first, second);
        assert_eq!(db.list_accounts().expect("list accounts").len(), 1);
    }

    #[test]
    fn a_never_seen_last4_creates_a_separate_account_until_linked() {
        let db = Db::open_in_memory().expect("open db");
        let original = db
            .find_or_create_credit_card_account(&sample_card("0002"))
            .expect("original card");

        // A reissue with no prior link — phase 1 has no automatic way to
        // know this is the same underlying card, so it becomes a new
        // account rather than silently guessing.
        let reissued = db
            .find_or_create_credit_card_account(&sample_card("9999"))
            .expect("reissued card");
        assert_ne!(original, reissued);
        assert_eq!(db.list_accounts().expect("list accounts").len(), 2);

        // Once the user confirms it's the same card, future imports of
        // the new last4 resolve back to the original account.
        db.link_card_number(original, "9999")
            .expect("link_card_number");
        let after_link = db
            .find_or_create_credit_card_account(&sample_card("9999"))
            .expect("find after link");
        assert_eq!(after_link, original);
        assert_eq!(
            db.card_number_history(original).expect("history"),
            vec!["9999", "0002"]
        );
    }
}
