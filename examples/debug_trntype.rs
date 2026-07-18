fn main() {
    let path = std::env::args().nth(1).expect("usage: debug_trntype <path.ofx>");
    let contents = std::fs::read_to_string(&path).expect("read file");
    let doc = ofx_rs::parse(&contents).expect("parse ofx");
    let banking = doc.banking().expect("banking");
    for wrapper in banking.statement_responses() {
        let Some(stmt) = wrapper.response() else { continue };
        let Some(txn_list) = stmt.transaction_list() else { continue };
        for txn in txn_list.transactions() {
            let name = txn.name().or(txn.memo()).unwrap_or_default();
            if name.trim_end().ends_with("BGC") {
                println!(
                    "trn_type={:?} name={:?} amount={:?}",
                    txn.transaction_type().to_string(),
                    name,
                    txn.amount().as_decimal()
                );
            }
        }
    }
}
