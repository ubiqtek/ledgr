// One-off backfill: `BarclaysOfxParser::parse` always set `trn_type`
// correctly, but `Db::insert_transaction`'s dedup-by-external_id (FITID) is a
// no-op on an already-imported row, so transactions imported before this
// field existed/was wired up were left with trn_type = NULL forever — a
// re-run of `ledgr import` never revisits them. This re-parses every
// processed OFX file in the inbox and updates any matching (account,
// external_id) row that's still NULL, without touching anything else.
//
// Not part of the crate proper — a throwaway maintenance script, run once
// against the real database with a backup taken first.

use rusqlite::{params, Connection};
use std::path::Path;

fn main() {
    let db_path = std::env::args().nth(1).expect("usage: backfill_trn_type <db-path> <processed-dir>");
    let processed_dir = std::env::args().nth(2).expect("usage: backfill_trn_type <db-path> <processed-dir>");

    let conn = Connection::open(&db_path).expect("open db");

    let mut total_updated = 0;

    for entry in std::fs::read_dir(&processed_dir).expect("read processed dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ofx") {
            continue;
        }
        let updated = backfill_one_file(&conn, &path);
        println!("{}: {} updated", path.display(), updated);
        total_updated += updated;
    }

    println!("total: {total_updated} row(s) updated");
}

fn backfill_one_file(conn: &Connection, path: &Path) -> usize {
    let contents = std::fs::read_to_string(path).expect("read ofx file");
    let Ok(doc) = ofx_rs::parse(&contents) else {
        println!("  (skipped: not valid OFX)");
        return 0;
    };
    let Some(banking) = doc.banking() else {
        return 0;
    };

    let mut updated = 0;
    for wrapper in banking.statement_responses() {
        let Some(stmt) = wrapper.response() else { continue };
        let bank_account = stmt.bank_account();
        let acct_id = bank_account.account_id().as_str();
        let (sort_code, account_number) = if acct_id.len() == 14 {
            (Some(acct_id[..6].to_string()), acct_id[6..].to_string())
        } else {
            (None, acct_id.to_string())
        };

        let account_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM accounts WHERE sort_code IS ?1 AND account_number = ?2",
                params![sort_code, account_number],
                |row| row.get(0),
            )
            .ok();
        let Some(account_id) = account_id else {
            println!("  (no matching account for sort {sort_code:?} / acct {account_number})");
            continue;
        };

        let Some(txn_list) = stmt.transaction_list() else { continue };
        for txn in txn_list.transactions() {
            let external_id = txn.fit_id().as_str().to_string();
            let trn_type = txn.transaction_type().to_string();
            let rows = conn
                .execute(
                    "UPDATE transactions SET trn_type = ?1
                     WHERE account_id = ?2 AND external_id = ?3 AND trn_type IS NULL",
                    params![trn_type, account_id, external_id],
                )
                .expect("update");
            updated += rows;
        }
    }
    updated
}
