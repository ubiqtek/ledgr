//! Spend ledger derivation pass — turns raw transactions into
//! `spend_entries`, or (for internal transfers) `transaction_links`, per
//! doc/implementation-notes/spend-ledger-design.md.
//!
//! Deliberately scoped to what the design doc's derivation rules table
//! covers for data ledgr can actually import today (Barclays OFX): rules
//! 1-7. Rules 8-10 (Barclaycard CSV `Subcategory`) have no code path yet —
//! no parser produces that field (Credit Card Transaction Import Task 1 is
//! still TODO). Spend enrichment (copying a transfer's reference onto a
//! later spend entry) is deferred — see the design doc's Summary.

use crate::db::Db;
use crate::model::{ClassifiedBy, LinkRelation, NewSpendEntry};
use std::collections::HashSet;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DerivationSummary {
    pub spend_entries_created: usize,
    pub transfers_detected: usize,
    pub transfers_paired: usize,
    pub out_of_scope: usize,
}

/// Runs the derivation pass over every raw transaction not yet linked to a
/// spend entry. `extra_household_accounts` are known-but-not-imported
/// accounts (e.g. a partner's) as `(sort_code, account_number)` pairs — all
/// imported accounts are household members automatically (see the design
/// doc's "Account registry" section).
pub fn derive_spend_entries(
    db: &Db,
    extra_household_accounts: &[(String, String)],
) -> anyhow::Result<DerivationSummary> {
    let accounts = db.list_accounts()?;
    let mut household: HashSet<(String, String)> = accounts
        .iter()
        .filter_map(|a| Some((a.sort_code.clone()?, a.account_number.clone()?)))
        .collect();
    household.extend(extra_household_accounts.iter().cloned());

    let mut summary = DerivationSummary::default();
    #[allow(clippy::type_complexity)]
    let mut transfer_candidates: Vec<(
        crate::model::Id,
        String,
        String,
        String,
        String,
        i64,
        String,
    )> = Vec::new();

    for txn in db.pending_derivation_transactions()? {
        let Some(account) = accounts.iter().find(|a| a.id == txn.account_id) else {
            continue;
        };

        match classify(
            &txn.description,
            txn.trn_type.as_deref(),
            txn.amount_minor,
            &household,
        ) {
            Classification::InternalTransfer {
                counterpart_sort,
                counterpart_account,
            } => {
                summary.transfers_detected += 1;
                if let (Some(own_sort), Some(own_account)) =
                    (&account.sort_code, &account.account_number)
                {
                    transfer_candidates.push((
                        txn.id,
                        own_sort.clone(),
                        own_account.clone(),
                        counterpart_sort,
                        counterpart_account,
                        txn.amount_minor,
                        txn.posted_at.clone(),
                    ));
                }
            }
            Classification::Spend {
                counterparty,
                rule_name,
                confidence,
            } => {
                db.insert_spend_entry_with_source(
                    &NewSpendEntry {
                        occurred_on: txn.posted_at.clone(),
                        amount_minor: txn.amount_minor,
                        currency: txn.currency.clone(),
                        counterparty,
                        description: txn.description.clone(),
                        note: None,
                        category_id: None,
                        classified_by: ClassifiedBy::Rule,
                        confidence: Some(confidence),
                        rule_name: Some(rule_name.to_string()),
                    },
                    txn.id,
                )?;
                summary.spend_entries_created += 1;
            }
            Classification::Refund {
                counterparty,
                rule_name,
            } => {
                db.insert_spend_entry_with_source(
                    &NewSpendEntry {
                        occurred_on: txn.posted_at.clone(),
                        amount_minor: txn.amount_minor,
                        currency: txn.currency.clone(),
                        counterparty: counterparty.clone(),
                        description: txn.description.clone(),
                        note: None,
                        category_id: None,
                        classified_by: ClassifiedBy::Rule,
                        confidence: Some(0.7),
                        rule_name: Some(rule_name.to_string()),
                    },
                    txn.id,
                )?;
                summary.spend_entries_created += 1;

                if let Some(prefix) = counterparty {
                    if let Some(original_id) = db.find_refund_original(
                        account.id,
                        &prefix,
                        txn.amount_minor,
                        &txn.posted_at,
                    )? {
                        db.insert_transaction_link(
                            original_id,
                            txn.id,
                            LinkRelation::Refund,
                            None,
                        )?;
                    }
                }
            }
            Classification::OutOfScope => {
                summary.out_of_scope += 1;
            }
        }
    }

    // Both legs of a transfer show up as their own candidate (each side's
    // NAME points at the other), so a naive pass would record the pairing
    // twice, once per direction — track which transactions are already
    // paired within this run to record each transfer once.
    let mut already_paired = std::collections::HashSet::new();
    for (
        from_id,
        own_sort,
        own_account,
        counterpart_sort,
        counterpart_account,
        amount_minor,
        posted_at,
    ) in transfer_candidates
    {
        if already_paired.contains(&from_id) {
            continue;
        }
        if let Some(to_id) = db.find_transfer_counterpart(
            from_id,
            &own_sort,
            &own_account,
            &counterpart_sort,
            &counterpart_account,
            amount_minor,
            &posted_at,
        )? {
            db.insert_transaction_link(from_id, to_id, LinkRelation::Transfer, Some(0.9))?;
            already_paired.insert(from_id);
            already_paired.insert(to_id);
            summary.transfers_paired += 1;
        }
    }

    Ok(summary)
}

#[derive(Debug, PartialEq)]
enum Classification {
    InternalTransfer {
        counterpart_sort: String,
        counterpart_account: String,
    },
    Spend {
        counterparty: Option<String>,
        rule_name: &'static str,
        confidence: f64,
    },
    Refund {
        counterparty: Option<String>,
        rule_name: &'static str,
    },
    OutOfScope,
}

/// Classifies one raw transaction per the derivation rules table in
/// doc/implementation-notes/spend-ledger-design.md. Rules are checked in
/// order, first match wins — critical for rule 1 (own-account transfer) to
/// beat TRNTYPE-based rules like `REPEATPMT`, which a standing order into a
/// household savings account would otherwise match (see the design doc's
/// note on rule precedence). Runs uniformly across every account regardless
/// of type — transfer pairing/reconciliation (rules 1-2) is what keeps
/// internal movement out of the ledger, not a pre-filter by account type
/// (see ADR 0006).
fn classify(
    description: &str,
    trn_type: Option<&str>,
    amount_minor: i64,
    household: &HashSet<(String, String)>,
) -> Classification {
    // Rules 1-2: NAME starts "<sort code> <account no>".
    if let Some((sort, account, _rest)) = parse_account_prefix(description) {
        return if household_contains(household, sort, account) {
            Classification::InternalTransfer {
                counterpart_sort: sort.to_string(),
                counterpart_account: account.to_string(),
            }
        } else if amount_minor < 0 {
            Classification::Spend {
                counterparty: None,
                rule_name: "external_account_payment",
                confidence: 0.6,
            }
        } else {
            Classification::OutOfScope
        };
    }

    // Rule 1b: NAME ends "<label> <sort code> <account no>" instead — the
    // same account reference, but with a human label first (e.g. "ADVENTURE
    // FUND 208794 33893693"). Only treated as internal transfer if the
    // trailing pair actually resolves to a household account; otherwise
    // falls through to the rules below rather than guessing.
    if let Some((sort, account)) = parse_trailing_account_suffix(description) {
        if household_contains(household, sort, account) {
            return Classification::InternalTransfer {
                counterpart_sort: sort.to_string(),
                counterpart_account: account.to_string(),
            };
        }
    }

    // Rules 3-5: NAME suffix (card payment/refund, or person "FT" payment).
    match suffix_token(description) {
        Some("CPM") if amount_minor < 0 => {
            return Classification::Spend {
                counterparty: merchant_prefix(description),
                rule_name: "card_payment",
                confidence: 0.95,
            };
        }
        Some("CRM") | Some("CRE") | Some("BCC") if amount_minor > 0 => {
            return Classification::Refund {
                counterparty: merchant_prefix(description),
                rule_name: "card_refund",
            };
        }
        Some("FT") => {
            return if amount_minor < 0 {
                Classification::Spend {
                    counterparty: None,
                    rule_name: "person_payment",
                    confidence: 0.7,
                }
            } else {
                // Reimbursements and Refunds: inbound money paying back
                // earlier spend from a person outside the household — a
                // sign-reversed spend entry, never income. See the
                // ubiquitous language doc.
                Classification::Spend {
                    counterparty: None,
                    rule_name: "reimbursement",
                    confidence: 0.6,
                }
            };
        }
        _ => {}
    }

    // Rules 6-7: fall back to TRNTYPE.
    match trn_type {
        Some("DIRECTDEBIT") if amount_minor < 0 => Classification::Spend {
            counterparty: None,
            rule_name: "direct_debit",
            confidence: 0.85,
        },
        Some("PAYMENT") if amount_minor < 0 => Classification::Spend {
            counterparty: None,
            rule_name: "payment",
            confidence: 0.85,
        },
        Some("REPEATPMT") if amount_minor < 0 => Classification::Spend {
            counterparty: None,
            rule_name: "repeat_payment",
            confidence: 0.85,
        },
        // DIRECTDEP = income (out of scope until the income ledger exists).
        // CASH = cash withdrawal (out of scope — see the design doc's rule 7).
        Some("DIRECTDEP") | Some("CASH") => Classification::OutOfScope,
        _ => {
            if amount_minor < 0 {
                Classification::Spend {
                    counterparty: None,
                    rule_name: "fallback",
                    confidence: 0.4,
                }
            } else {
                Classification::OutOfScope
            }
        }
    }
}

/// Recognises Barclays' `<sort code> <account no>` `NAME` prefix (6 then 8
/// ASCII digits, space-separated) — see doc/kb/ofx/structure.md. Returns
/// `(sort_code, account_number, rest_of_description)`.
fn parse_account_prefix(description: &str) -> Option<(&str, &str, &str)> {
    let mut parts = description.splitn(3, ' ');
    let sort = parts.next()?;
    let account = parts.next()?;
    let rest = parts.next().unwrap_or("");
    let is_digits = |s: &str, len: usize| s.len() == len && s.bytes().all(|b| b.is_ascii_digit());
    if is_digits(sort, 6) && is_digits(account, 8) {
        Some((sort, account, rest))
    } else {
        None
    }
}

/// Recognises Barclays' other `NAME` shape for a transfer: a human label
/// followed by `<sort code> <account no>` at the *end* of the description
/// (e.g. `"ADVENTURE FUND 208794 33893693"`), rather than at the start. The
/// account number is sometimes truncated to 6 digits when the label pushes
/// the whole `NAME` field past Barclays' length limit (e.g.
/// `"SHARED BILLS ACCO 208794 231650"` — real account is `...23165086`, cut
/// to `231650`) — `household_contains` handles matching a truncated account
/// number against the full one on file. Returns `(sort_code, account_no)`,
/// which may itself be truncated.
fn parse_trailing_account_suffix(description: &str) -> Option<(&str, &str)> {
    let tokens: Vec<&str> = description.split_whitespace().collect();
    let account = *tokens.last()?;
    let sort = *tokens.get(tokens.len().checked_sub(2)?)?;
    let is_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if sort.len() == 6 && is_digits(sort) && (6..=8).contains(&account.len()) && is_digits(account)
    {
        Some((sort, account))
    } else {
        None
    }
}

/// Household membership check tolerant of a `NAME`-truncated account number
/// (see `parse_trailing_account_suffix`): matches if `account` is either the
/// full account number on file or a prefix of it.
fn household_contains(household: &HashSet<(String, String)>, sort: &str, account: &str) -> bool {
    household
        .iter()
        .any(|(hs, ha)| hs == sort && (ha == account || ha.starts_with(account)))
}

/// Last whitespace-separated token, e.g. `"CPM"` in `"TESCO ON 09 JUL CPM"`.
fn suffix_token(description: &str) -> Option<&str> {
    description.split_whitespace().last()
}

/// Text before `" ON "` in a card-payment/refund description, e.g.
/// `"PETROL STATION 12"` from `"PETROL STATION 12 ON 09 JUL CPM"`. `None`
/// if the marker isn't present (shouldn't happen for CPM/CRM/CRE/BCC lines).
fn merchant_prefix(description: &str) -> Option<String> {
    description
        .rfind(" ON ")
        .map(|idx| description[..idx].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HouseholdAccountRef;
    use crate::db::Db;
    use crate::model::{AccountType, NewAccount, NewTransaction};

    fn household_of(accounts: &[(&str, &str)]) -> HashSet<(String, String)> {
        accounts
            .iter()
            .map(|(s, a)| (s.to_string(), a.to_string()))
            .collect()
    }

    #[test]
    fn classifies_a_transfer_to_a_known_household_account_as_internal() {
        let household = household_of(&[("209934", "87654321")]);
        let result = classify(
            "209934 87654321 PIZZA OVEN FT",
            Some("OTHER"),
            -8900,
            &household,
        );
        assert_eq!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "209934".into(),
                counterpart_account: "87654321".into(),
            }
        );
    }

    #[test]
    fn account_prefix_rule_beats_repeatpmt_trntype_for_a_standing_order_into_savings() {
        // The exact overlap the design doc's precedence note calls out: a
        // standing order (REPEATPMT) into the user's own savings account
        // must not be misclassified as spend.
        let household = household_of(&[("209934", "87654321")]);
        let result = classify(
            "209934 87654321 STO SAVINGS",
            Some("REPEATPMT"),
            -20000,
            &household,
        );
        assert_eq!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "209934".into(),
                counterpart_account: "87654321".into(),
            }
        );
    }

    #[test]
    fn classifies_a_payment_to_an_unknown_account_as_low_confidence_spend() {
        let household = HashSet::new();
        let result = classify(
            "609934 11112222 RENT FT",
            Some("OTHER"),
            -75000,
            &household,
        );
        assert_eq!(
            result,
            Classification::Spend {
                counterparty: None,
                rule_name: "external_account_payment",
                confidence: 0.6,
            }
        );
    }

    #[test]
    fn classifies_a_card_payment() {
        let household = HashSet::new();
        let result = classify(
            "PETROL STATION 12 ON 09 JUL CPM",
            Some("OTHER"),
            -4550,
            &household,
        );
        assert_eq!(
            result,
            Classification::Spend {
                counterparty: Some("PETROL STATION 12".into()),
                rule_name: "card_payment",
                confidence: 0.95,
            }
        );
    }

    #[test]
    fn classifies_a_card_refund() {
        let household = HashSet::new();
        let result = classify(
            "GARAGE SERVICES ON 26 FEB CRM",
            Some("OTHER"),
            4000,
            &household,
        );
        assert_eq!(
            result,
            Classification::Refund {
                counterparty: Some("GARAGE SERVICES".into()),
                rule_name: "card_refund",
            }
        );
    }

    #[test]
    fn classifies_an_outbound_person_payment_as_spend() {
        let household = HashSet::new();
        let result = classify(
            "J SMITH WINDOW CLEAN FT",
            Some("OTHER"),
            -2500,
            &household,
        );
        assert_eq!(
            result,
            Classification::Spend {
                counterparty: None,
                rule_name: "person_payment",
                confidence: 0.7,
            }
        );
    }

    #[test]
    fn classifies_an_inbound_person_payment_as_a_reimbursement() {
        let household = HashSet::new();
        let result = classify(
            "J SMITH CONCERT TICKET FT",
            Some("OTHER"),
            3000,
            &household,
        );
        assert_eq!(
            result,
            Classification::Spend {
                counterparty: None,
                rule_name: "reimbursement",
                confidence: 0.6,
            }
        );
    }

    #[test]
    fn classifies_a_direct_debit_as_spend() {
        let household = HashSet::new();
        let result = classify("SPOTIFY", Some("DIRECTDEBIT"), -999, &household);
        assert_eq!(
            result,
            Classification::Spend {
                counterparty: None,
                rule_name: "direct_debit",
                confidence: 0.85,
            }
        );
    }

    #[test]
    fn classifies_a_direct_deposit_as_out_of_scope() {
        let household = HashSet::new();
        let result = classify("SALARY", Some("DIRECTDEP"), 150000, &household);
        assert_eq!(result, Classification::OutOfScope);
    }

    #[test]
    fn classifies_a_cash_withdrawal_as_out_of_scope() {
        let household = HashSet::new();
        let result = classify("CASH WITHDRAWAL", Some("CASH"), -5000, &household);
        assert_eq!(result, Classification::OutOfScope);
    }

    #[test]
    fn derive_spend_entries_creates_a_spend_entry_for_a_card_payment() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("203040".into()),
                account_number: Some("12345678".into()),
            })
            .expect("insert account");
        db.insert_transaction(&NewTransaction {
            account_id,
            import_id: None,
            posted_at: "2026-07-01".into(),
            amount_minor: -2599,
            currency: "GBP".into(),
            description: "TESCO STORES ON 01 JUL CPM".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
        })
        .expect("insert transaction");

        let summary = derive_spend_entries(&db, &[]).expect("derive");
        assert_eq!(
            summary,
            DerivationSummary {
                spend_entries_created: 1,
                transfers_detected: 0,
                transfers_paired: 0,
                out_of_scope: 0,
            }
        );
    }

    #[test]
    fn derive_spend_entries_is_idempotent() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("203040".into()),
                account_number: Some("12345678".into()),
            })
            .expect("insert account");
        db.insert_transaction(&NewTransaction {
            account_id,
            import_id: None,
            posted_at: "2026-07-01".into(),
            amount_minor: -2599,
            currency: "GBP".into(),
            description: "TESCO STORES ON 01 JUL CPM".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
        })
        .expect("insert transaction");

        derive_spend_entries(&db, &[]).expect("first derive");
        let second = derive_spend_entries(&db, &[]).expect("second derive");
        assert_eq!(
            second.spend_entries_created, 0,
            "must not double-derive an already-linked transaction"
        );
    }

    #[test]
    fn derive_spend_entries_pairs_both_legs_of_a_real_transfer() {
        let db = Db::open_in_memory().expect("open db");
        let bills_account = db
            .insert_account(&NewAccount {
                name: "Bills Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209912".into()),
                account_number: Some("12345678".into()),
            })
            .expect("insert bills account");
        let spending_account = db
            .insert_account(&NewAccount {
                name: "Spending Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209934".into()),
                account_number: Some("87654321".into()),
            })
            .expect("insert spending account");

        db.insert_transaction(&NewTransaction {
            account_id: bills_account,
            import_id: None,
            posted_at: "2026-07-01".into(),
            amount_minor: -8900,
            currency: "GBP".into(),
            description: "209934 87654321 PIZZA OVEN FT".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
        })
        .expect("insert bills-side transaction");
        db.insert_transaction(&NewTransaction {
            account_id: spending_account,
            import_id: None,
            posted_at: "2026-07-01".into(),
            amount_minor: 8900,
            currency: "GBP".into(),
            description: "209912 12345678 PIZZA OVEN FT".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
        })
        .expect("insert spending-side transaction");

        let summary = derive_spend_entries(&db, &[]).expect("derive");
        assert_eq!(
            summary.spend_entries_created, 0,
            "internal transfers must not become spend"
        );
        assert_eq!(
            summary.transfers_detected, 2,
            "both legs are recognised as internal"
        );
        assert_eq!(
            summary.transfers_paired, 1,
            "exactly one pairing across the two legs"
        );
    }

    #[test]
    fn derive_spend_entries_recognises_a_configured_partner_account_as_household() {
        let db = Db::open_in_memory().expect("open db");
        let account_id = db
            .insert_account(&NewAccount {
                name: "Current Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("203040".into()),
                account_number: Some("12345678".into()),
            })
            .expect("insert account");
        db.insert_transaction(&NewTransaction {
            account_id,
            import_id: None,
            posted_at: "2026-07-01".into(),
            amount_minor: -10000,
            currency: "GBP".into(),
            description: "609934 99998888 SHARED BILLS FT".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
        })
        .expect("insert transaction");

        let partner = HouseholdAccountRef {
            sort_code: "609934".into(),
            account_number: "99998888".into(),
            label: Some("Partner".into()),
        };
        let summary = derive_spend_entries(
            &db,
            &[(partner.sort_code.clone(), partner.account_number.clone())],
        )
        .expect("derive");
        assert_eq!(summary.spend_entries_created, 0);
        assert_eq!(summary.transfers_detected, 1);
    }
}
