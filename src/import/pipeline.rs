//! Scans the inbox for statement files, imports any not seen before, and
//! moves each into `processed/` once handled.

use super::{BarclaysOfxParser, GenericCsvParser, StatementParser};
use crate::db::Db;
use crate::inbox::Inbox;
use crate::model::{AccountType, NewAccount};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub files_imported: usize,
    pub files_skipped: usize,
    pub transactions_imported: usize,
}

pub fn import_inbox(db: &Db, inbox: &Inbox) -> anyhow::Result<ImportSummary> {
    inbox.ensure_dirs()?;
    let mut summary = ImportSummary::default();

    for path in inbox.pending_files()? {
        let file_hash = hash_file(&path)?;

        if db.find_statement_by_hash(&file_hash)?.is_some() {
            summary.files_skipped += 1;
            inbox.mark_processed(&path)?;
            continue;
        }

        let Some(parser) = parser_for(&path) else {
            summary.files_skipped += 1;
            continue;
        };

        // Formats that identify their own account (e.g. OFX's BANKACCTFROM)
        // resolve to that specific account. Formats that don't (e.g. a
        // generic CSV, which carries no account identity of its own) fall
        // back to a single default account until multi-institution/format
        // account matching exists (see doc/planning/plan.md).
        let account_id = match parser.account_identity(&path)? {
            Some(identity) => db.find_or_create_account(&identity)?,
            None => db.find_or_create_account(&NewAccount {
                name: "Barclays Current Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Checking,
                currency: "GBP".into(),
            })?,
        };

        let Some(statement_id) =
            db.insert_statement(account_id, &path.to_string_lossy(), &file_hash, None, None)?
        else {
            // Another statement already claimed this hash between the check
            // above and now; treat it as already imported.
            summary.files_skipped += 1;
            inbox.mark_processed(&path)?;
            continue;
        };

        let mut transactions = parser.parse(&path, account_id)?;
        for txn in &mut transactions {
            txn.statement_id = Some(statement_id);
            db.insert_transaction(txn)?;
        }

        if let Some((balance_minor, as_of)) = parser.balance_snapshot(&path)? {
            db.insert_balance_snapshot(account_id, Some(statement_id), balance_minor, &as_of)?;
        }

        summary.transactions_imported += transactions.len();
        summary.files_imported += 1;
        inbox.mark_processed(&path)?;
    }

    Ok(summary)
}

fn parser_for(path: &Path) -> Option<Box<dyn StatementParser>> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("ofx" | "qfx") => Some(Box::new(BarclaysOfxParser)),
        Some("csv") => Some(Box::new(GenericCsvParser {
            currency: "GBP".into(),
        })),
        _ => None,
    }
}

fn hash_file(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
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
</STMTTRN>
</BANKTRANLIST>
<LEDGERBAL>
<BALAMT>974.01</BALAMT>
<DTASOF>20260701120000</DTASOF>
</LEDGERBAL>
</STMTRS>
</STMTTRNRS>
</BANKMSGSRSV1>
</OFX>"#;

    #[test]
    fn imports_a_pending_ofx_file_and_moves_it_to_processed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inbox = Inbox::new(dir.path().to_path_buf());
        inbox.ensure_dirs().expect("ensure_dirs");
        std::fs::write(dir.path().join("statement.ofx"), SAMPLE_OFX).expect("write file");

        let db = Db::open_in_memory().expect("open db");
        let summary = import_inbox(&db, &inbox).expect("import_inbox");

        assert_eq!(
            summary,
            ImportSummary {
                files_imported: 1,
                files_skipped: 0,
                transactions_imported: 1,
            }
        );
        assert!(!dir.path().join("statement.ofx").exists());
        assert!(inbox.processed_dir().join("statement.ofx").exists());

        let accounts = db.list_accounts().expect("list accounts");
        assert_eq!(accounts.len(), 1);
        let txs = db
            .list_transactions_for_account(accounts[0].id)
            .expect("list transactions");
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].external_id.as_deref(), Some("202607010001"));
    }

    #[test]
    fn re_running_import_skips_already_imported_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inbox = Inbox::new(dir.path().to_path_buf());
        inbox.ensure_dirs().expect("ensure_dirs");
        std::fs::write(dir.path().join("statement.ofx"), SAMPLE_OFX).expect("write file");

        let db = Db::open_in_memory().expect("open db");
        import_inbox(&db, &inbox).expect("first import");

        // Drop the same file back into the inbox, as if re-downloaded.
        std::fs::write(dir.path().join("statement.ofx"), SAMPLE_OFX).expect("write file again");
        let summary = import_inbox(&db, &inbox).expect("second import");

        assert_eq!(summary.files_imported, 0);
        assert_eq!(summary.files_skipped, 1);
        let accounts = db.list_accounts().expect("list accounts");
        let txs = db
            .list_transactions_for_account(accounts[0].id)
            .expect("list transactions");
        assert_eq!(txs.len(), 1, "should not have duplicated the transaction");
    }

    #[test]
    fn import_records_a_balance_snapshot_from_ledgerbal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inbox = Inbox::new(dir.path().to_path_buf());
        inbox.ensure_dirs().expect("ensure_dirs");
        std::fs::write(dir.path().join("statement.ofx"), SAMPLE_OFX).expect("write file");

        let db = Db::open_in_memory().expect("open db");
        import_inbox(&db, &inbox).expect("import_inbox");

        let accounts = db.list_accounts().expect("list accounts");
        let (balance, as_of) = db
            .latest_balance_snapshot(accounts[0].id)
            .expect("latest_balance_snapshot")
            .expect("SAMPLE_OFX carries a LEDGERBAL");
        assert_eq!(balance, 97_401);
        assert_eq!(as_of, "2026-07-01");
    }

    #[test]
    fn ofx_files_for_different_accounts_import_into_separate_accounts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inbox = Inbox::new(dir.path().to_path_buf());
        inbox.ensure_dirs().expect("ensure_dirs");

        // Same shape as SAMPLE_OFX but a different ACCTID, as if downloaded
        // for a second Barclays account.
        let other_ofx = SAMPLE_OFX.replace("12345678", "99998888");
        std::fs::write(dir.path().join("a.ofx"), SAMPLE_OFX).expect("write file");
        std::fs::write(dir.path().join("b.ofx"), other_ofx).expect("write file");

        let db = Db::open_in_memory().expect("open db");
        let summary = import_inbox(&db, &inbox).expect("import_inbox");

        assert_eq!(summary.files_imported, 2);
        let accounts = db.list_accounts().expect("list accounts");
        assert_eq!(
            accounts.len(),
            2,
            "two OFX files for different accounts must not collapse into one account"
        );
    }
}
