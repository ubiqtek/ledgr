use super::{ImportError, StatementParser};
use crate::model::{AccountType, Id, NewAccount, NewTransaction};
use std::path::Path;

/// Parses a Barclays "Download transactions" OFX export (desktop online
/// banking only). Maps the OFX `FITID` to `Transaction::external_id` so
/// re-importing an overlapping date range — Barclays only offers the last
/// 60-90 days — can be de-duplicated without a fragile content hash. See
/// `doc/adr/0002-use-ofx-for-barclays-statement-import.md`.
pub struct BarclaysOfxParser;

impl StatementParser for BarclaysOfxParser {
    fn name(&self) -> &'static str {
        "Barclays OFX"
    }

    /// Barclays "Download transactions" exports one account per file, with
    /// that account identified by `BANKACCTFROM`. A user with multiple
    /// accounts (current, savings, ...) downloads a separate file per
    /// account, so this must not collapse them into one shared account —
    /// each gets its own `NewAccount`, keyed by the last 4 digits of
    /// `ACCTID` so re-importing the same account resolves to the same row.
    fn account_identity(&self, path: &Path) -> Result<Option<NewAccount>, ImportError> {
        let contents = std::fs::read_to_string(path)?;
        let doc =
            ofx_rs::parse(&contents).map_err(|e| ImportError::Parse(format!("invalid OFX: {e}")))?;
        let banking = doc
            .banking()
            .ok_or_else(|| ImportError::Parse("no banking statement in OFX file".into()))?;
        let stmt = banking
            .statement_responses()
            .iter()
            .find_map(|wrapper| wrapper.response())
            .ok_or_else(|| ImportError::Parse("no statement response in OFX file".into()))?;

        let bank_account = stmt.bank_account();
        let acct_id = bank_account.account_id().as_str();
        let last4 = &acct_id[acct_id.len().saturating_sub(4)..];
        let account_type = match bank_account.account_type() {
            ofx_rs::types::AccountType::Checking => AccountType::Current,
            ofx_rs::types::AccountType::Savings | ofx_rs::types::AccountType::MoneyMarket => {
                AccountType::Savings
            }
            ofx_rs::types::AccountType::CreditLine => AccountType::CreditCard,
            _ => AccountType::Other,
        };
        let type_label = match account_type {
            AccountType::Current => "Current Account",
            AccountType::Savings => "Savings Account",
            AccountType::CreditCard => "Credit Card",
            AccountType::Pension | AccountType::Investment | AccountType::Other => "Account",
        };

        Ok(Some(NewAccount {
            name: format!("Barclays {type_label} (...{last4})"),
            institution: Some("Barclays".into()),
            account_type,
            currency: stmt.currency_default().as_str().to_string(),
        }))
    }

    /// Reads OFX's `LEDGERBAL` — the balance per Barclays' own records as of
    /// a point in time — as an anchor for `Db::balance_as_of`. This is
    /// distinct from (and more trustworthy than) summing the transactions in
    /// the same file, since `BANKTRANLIST` may only cover a recent window.
    fn balance_snapshot(&self, path: &Path) -> Result<Option<(i64, String)>, ImportError> {
        let contents = std::fs::read_to_string(path)?;
        let doc =
            ofx_rs::parse(&contents).map_err(|e| ImportError::Parse(format!("invalid OFX: {e}")))?;
        let banking = doc
            .banking()
            .ok_or_else(|| ImportError::Parse("no banking statement in OFX file".into()))?;
        let stmt = banking
            .statement_responses()
            .iter()
            .find_map(|wrapper| wrapper.response())
            .ok_or_else(|| ImportError::Parse("no statement response in OFX file".into()))?;

        let Some(ledger_balance) = stmt.ledger_balance() else {
            return Ok(None);
        };
        let balance_minor = ofx_amount_to_minor(ledger_balance.amount());
        let as_of = ledger_balance.as_of().as_offset_date_time().date().to_string();
        Ok(Some((balance_minor, as_of)))
    }

    fn parse(&self, path: &Path, account_id: Id) -> Result<Vec<NewTransaction>, ImportError> {
        let contents = std::fs::read_to_string(path)?;
        let doc =
            ofx_rs::parse(&contents).map_err(|e| ImportError::Parse(format!("invalid OFX: {e}")))?;

        let banking = doc
            .banking()
            .ok_or_else(|| ImportError::Parse("no banking statement in OFX file".into()))?;

        let mut transactions = Vec::new();
        for wrapper in banking.statement_responses() {
            let Some(stmt) = wrapper.response() else {
                continue;
            };
            let Some(txn_list) = stmt.transaction_list() else {
                continue;
            };
            let currency = stmt.currency_default().as_str().to_string();

            for txn in txn_list.transactions() {
                let posted_at = txn.date_posted().as_offset_date_time().date().to_string();

                let amount_minor = ofx_amount_to_minor(txn.amount());

                let description = clean_description(txn.name().or(txn.memo()).unwrap_or_default());

                transactions.push(NewTransaction {
                    account_id,
                    statement_id: None,
                    posted_at,
                    amount_minor,
                    currency: currency.clone(),
                    description: description.clone(),
                    raw_description: Some(description),
                    category_id: None,
                    external_id: Some(txn.fit_id().as_str().to_string()),
                });
            }
        }
        Ok(transactions)
    }
}

/// Collapses whitespace (including the literal tab characters Barclays'
/// OFX export embeds between the merchant name and its trailing "ON DD MON"
/// suffix, which would otherwise jump to the next terminal tab stop and
/// wreck column alignment) down to single spaces.
fn clean_description(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Converts an OFX amount (arbitrary-precision decimal) to signed minor
/// currency units (e.g. pence), the representation `ledgr` stores.
fn ofx_amount_to_minor(amount: ofx_rs::types::OfxAmount) -> i64 {
    let mut decimal = amount.as_decimal();
    decimal.rescale(2);
    decimal.mantissa() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OFX: &str = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?>
<OFX>
<SIGNONMSGSRSV1>
<SONRS>
<STATUS><CODE>0</CODE><SEVERITY>INFO</SEVERITY></STATUS>
<DTSERVER>20260701120000</DTSERVER>
<LANGUAGE>ENG</LANGUAGE>
</SONRS>
</SIGNONMSGSRSV1>
<BANKMSGSRSV1>
<STMTTRNRS>
<TRNUID>1001</TRNUID>
<STATUS><CODE>0</CODE><SEVERITY>INFO</SEVERITY></STATUS>
<STMTRS>
<CURDEF>GBP</CURDEF>
<BANKACCTFROM>
<BANKID>203040</BANKID>
<ACCTID>12345678</ACCTID>
<ACCTTYPE>CHECKING</ACCTTYPE>
</BANKACCTFROM>
<BANKTRANLIST>
<DTSTART>20260601</DTSTART>
<DTEND>20260701</DTEND>
<STMTTRN>
<TRNTYPE>DEBIT</TRNTYPE>
<DTPOSTED>20260701</DTPOSTED>
<TRNAMT>-25.99</TRNAMT>
<FITID>202607010001</FITID>
<NAME>TESCO STORES</NAME>
<MEMO>Groceries</MEMO>
</STMTTRN>
<STMTTRN>
<TRNTYPE>CREDIT</TRNTYPE>
<DTPOSTED>20260702</DTPOSTED>
<TRNAMT>1500.00</TRNAMT>
<FITID>202607020001</FITID>
<NAME>SALARY</NAME>
</STMTTRN>
</BANKTRANLIST>
<LEDGERBAL>
<BALAMT>1474.01</BALAMT>
<DTASOF>20260702120000</DTASOF>
</LEDGERBAL>
</STMTRS>
</STMTTRNRS>
</BANKMSGSRSV1>
</OFX>"#;

    #[test]
    fn parses_barclays_style_ofx() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("statement.ofx");
        std::fs::write(&path, SAMPLE_OFX).expect("write file");

        let txs = BarclaysOfxParser.parse(&path, 1).expect("parse");
        assert_eq!(txs.len(), 2);

        assert_eq!(txs[0].posted_at, "2026-07-01");
        assert_eq!(txs[0].amount_minor, -2599);
        assert_eq!(txs[0].currency, "GBP");
        assert_eq!(txs[0].description, "TESCO STORES");
        assert_eq!(txs[0].external_id.as_deref(), Some("202607010001"));

        assert_eq!(txs[1].posted_at, "2026-07-02");
        assert_eq!(txs[1].amount_minor, 150000);
        assert_eq!(txs[1].external_id.as_deref(), Some("202607020001"));
    }

    #[test]
    fn balance_snapshot_reads_ledgerbal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("statement.ofx");
        std::fs::write(&path, SAMPLE_OFX).expect("write file");

        let (balance_minor, as_of) = BarclaysOfxParser
            .balance_snapshot(&path)
            .expect("balance_snapshot")
            .expect("SAMPLE_OFX has a LEDGERBAL");

        assert_eq!(balance_minor, 147_401);
        assert_eq!(as_of, "2026-07-02");
    }

    #[test]
    fn account_identity_reads_bankacctfrom() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("statement.ofx");
        std::fs::write(&path, SAMPLE_OFX).expect("write file");

        let identity = BarclaysOfxParser
            .account_identity(&path)
            .expect("account_identity")
            .expect("Barclays OFX always identifies its account");

        assert_eq!(identity.name, "Barclays Current Account (...5678)");
        assert_eq!(identity.institution.as_deref(), Some("Barclays"));
        assert_eq!(identity.account_type, AccountType::Current);
        assert_eq!(identity.currency, "GBP");
    }

    #[test]
    fn account_identity_distinguishes_different_accounts() {
        // Same shape as SAMPLE_OFX but a different ACCTID, as if downloaded
        // for a second Barclays account.
        let other_ofx = SAMPLE_OFX.replace("12345678", "99998888");
        let dir = tempfile::tempdir().expect("tempdir");
        let path_a = dir.path().join("a.ofx");
        let path_b = dir.path().join("b.ofx");
        std::fs::write(&path_a, SAMPLE_OFX).expect("write file");
        std::fs::write(&path_b, other_ofx).expect("write file");

        let identity_a = BarclaysOfxParser
            .account_identity(&path_a)
            .expect("account_identity")
            .expect("identified");
        let identity_b = BarclaysOfxParser
            .account_identity(&path_b)
            .expect("account_identity")
            .expect("identified");

        assert_ne!(identity_a.name, identity_b.name);
    }
}
