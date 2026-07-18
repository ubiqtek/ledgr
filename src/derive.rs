//! Spend ledger derivation pass — turns raw transactions into
//! `spend_entries`, or (for internal transfers) `transfer_entries`, per
//! doc/implementation-notes/spend-ledger-design.md and
//! doc/implementation-notes/transfer-ledger-design.md.
//!
//! Deliberately scoped to what the design doc's derivation rules table
//! covers for data ledgr can actually import today (Barclays OFX): rules
//! 1-7. Rules 8-10 (Barclaycard CSV `Subcategory`) have no code path yet —
//! no parser produces that field (Credit Card Transaction Import Task 1 is
//! still TODO). Spend enrichment (copying a transfer's reference onto a
//! later spend entry) is deferred — see the design doc's Summary.

use crate::config::HouseholdAccountRef;
use crate::db::Db;
use crate::model::{
    Account, ClassifiedBy, NewIncomeEntry, NewSpendEntry, NewTransferLeg, TransferLegRole,
    TransferPairMethod,
};
use std::collections::HashSet;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DerivationSummary {
    pub spend_entries_created: usize,
    pub income_entries_created: usize,
    pub transfers_detected: usize,
    pub transfers_paired: usize,
    pub out_of_scope: usize,
    pub card_payments_matched: usize,
    /// Card payment legs recorded this run with no counterpart found yet
    /// (the credit card statement hasn't been imported) — persisted as a
    /// one-sided `transfer_entries` row, same "stay visible" treatment as
    /// any other unpaired transfer leg, retried on future runs rather than
    /// becoming a permanent low-confidence spend entry. See
    /// doc/implementation-notes/transfer-ledger-critique.md.
    pub card_payments_unmatched: usize,
    /// Pairings completed this run where at least one leg was already
    /// persisted from an earlier run — either a genuinely new leg
    /// completing an old one (see
    /// doc/implementation-notes/transfer-ledger-design.md, "Pairing can
    /// complete retroactively"), or two already-persisted legs that only
    /// became pairable once a new pairing tier was added (e.g. tier 3's
    /// rollout, which paired legs neither newly imported nor newly
    /// classified this run). A subset of `transfers_paired`, not an
    /// addition to it.
    pub transfers_backfilled: usize,
}

/// Confidence assigned to a tier-3 (`SelfReferenceMatch`) transfer pairing.
/// Deliberately below tier 2's `AmountDateMatch` (0.75): tier 2 at least
/// gets a mutual cross-check (both legs' own `NAME` decodes agree with each
/// other), whereas tier 3 has no cross-check from the candidate's own decode
/// at all — self-reference plus an exact amount+date match is the entire
/// signal. Still a judgement call, like tier 2's 0.75 was (see the design
/// doc's open questions) — chosen to sit clearly below tier 2 without
/// dropping into the same range as the low-confidence spend fallbacks
/// (0.4-0.6) that flag something as actually uncertain, since the
/// *classification* here (this leg is an internal transfer) is still
/// deterministic; only the *pairing* is weaker.
const SELF_REFERENCE_MATCH_CONFIDENCE: f64 = 0.6;

/// Runs the derivation pass over every raw transaction not yet linked to a
/// spend entry. `extra_household_accounts` are known-but-not-imported
/// accounts (e.g. a partner's) — all imported accounts are household
/// members automatically (see the design doc's "Account registry" section).
/// An entry with a `name` set is also matched against person-to-person
/// `NAME` fields that carry no account digits at all (see
/// `matches_person_name`). `income_sources` and `registered_people` are
/// `config.toml`'s Income Source / Registered Person lists (see the
/// ubiquitous language doc).
pub fn run_derivation(
    db: &Db,
    extra_household_accounts: &[HouseholdAccountRef],
    income_sources: &[crate::config::IncomeSourceRef],
    registered_people: &[crate::config::RegisteredPersonRef],
    reimbursement_sources: &[crate::config::ReimbursementSourceRef],
) -> anyhow::Result<DerivationSummary> {
    let accounts = db.list_accounts()?;
    let (household, household_names) = build_household(&accounts, extra_household_accounts);
    let income_sources: Vec<(&str, crate::config::IncomeSourceKind)> = income_sources
        .iter()
        .map(|s| (s.name.as_str(), s.kind))
        .collect();
    let registered_people: Vec<&str> = registered_people.iter().map(|p| p.name.as_str()).collect();
    let reimbursement_sources: Vec<&str> = reimbursement_sources
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    let mut summary = DerivationSummary::default();
    // Transaction ids given a `transfer_entries` row *this run*, via any
    // path (a fresh leg recorded standalone, one completing an existing
    // row, or one directly paired with a counterpart found by tier 1) —
    // used to decide whether a pairing counts as `transfers_backfilled`
    // (only when the OTHER side predates this exact operation) and to
    // guard against reprocessing a transaction later in this same
    // iteration (`pending_derivation_transactions` is a fixed snapshot
    // taken once at the top of this loop, so a transaction whose row gets
    // created mid-loop — e.g. as the counterpart tier 1 finds for an
    // *earlier* transaction in this same snapshot — can still appear later
    // in the very same iteration).
    let mut this_run_ids: HashSet<crate::model::Id> = HashSet::new();
    let mut card_payment_candidates: Vec<crate::model::Transaction> = Vec::new();

    for txn in db.pending_derivation_transactions()? {
        let Some(account) = accounts.iter().find(|a| a.id == txn.account_id) else {
            continue;
        };

        match classify(
            &txn.description,
            txn.trn_type.as_deref(),
            txn.amount_minor,
            &household,
            &household_names,
            &income_sources,
            &registered_people,
            &reimbursement_sources,
        ) {
            Classification::InternalTransfer {
                counterpart_sort,
                counterpart_account,
            } => {
                summary.transfers_detected += 1;
                // Already given a row this run — either this transaction
                // itself was processed earlier in this same snapshot, or it
                // was consumed as another leg's tier-1 counterpart. Still
                // worth counting as `transfers_detected` above (classify()
                // legitimately recognised it as an internal transfer), just
                // not worth recording/pairing again.
                if this_run_ids.contains(&txn.id) {
                    continue;
                }
                if account.sort_code.is_some() && account.account_number.is_some() {
                    this_run_ids.insert(txn.id);
                    // Truncation-tolerant match, same rule as
                    // `household_contains`, so this resolves to the same
                    // account classification already settled on.
                    let counterpart_account_id = accounts
                        .iter()
                        .find(|a| {
                            a.sort_code.as_deref() == Some(counterpart_sort.as_str())
                                && a.account_number
                                    .as_deref()
                                    .is_some_and(|full| full.starts_with(&counterpart_account))
                        })
                        .map(|a| a.id);
                    let role = if txn.amount_minor < 0 {
                        TransferLegRole::Out
                    } else {
                        TransferLegRole::In
                    };
                    let leg = NewTransferLeg {
                        transaction_id: txn.id,
                        account_id: account.id,
                        role,
                        occurred_on: txn.posted_at.clone(),
                        amount_minor: txn.amount_minor.abs(),
                        currency: txn.currency.clone(),
                        description: txn.description.clone(),
                        counterpart_sort_code: counterpart_sort.clone(),
                        counterpart_account_number: counterpart_account.clone(),
                        counterpart_account_id,
                        classified_by: ClassifiedBy::Rule,
                        confidence: Some(1.0),
                        rule_name: Some("household_transfer".to_string()),
                    };

                    // Tier 1: description cross-reference against raw
                    // transactions (manual transfers) — works regardless of
                    // whether the counterpart has itself been classified
                    // yet, since it searches `transactions` directly, not
                    // `transfer_entries`.
                    let tier1 = match counterpart_account_id {
                        Some(_) => db.find_transfer_counterpart(
                            txn.id,
                            account.sort_code.as_deref().unwrap(),
                            account.account_number.as_deref().unwrap(),
                            &counterpart_sort,
                            &counterpart_account,
                            txn.amount_minor,
                            &txn.posted_at,
                        )?,
                        None => None,
                    };

                    if let Some(counterpart_transaction_id) = tier1 {
                        let counterpart_predates_this_run =
                            !this_run_ids.contains(&counterpart_transaction_id);
                        if let Some(existing_row) =
                            db.transfer_row_for_transaction(counterpart_transaction_id)?
                        {
                            db.complete_transfer_leg(
                                existing_row,
                                &leg,
                                TransferPairMethod::DescriptionMatch,
                                0.9,
                            )?;
                            summary.transfers_paired += 1;
                            if counterpart_predates_this_run {
                                summary.transfers_backfilled += 1;
                            }
                        } else if let Some(counterpart) =
                            db.get_transaction(counterpart_transaction_id)?
                        {
                            db.create_paired_transfer(
                                &leg,
                                &counterpart,
                                TransferPairMethod::DescriptionMatch,
                                0.9,
                            )?;
                            summary.transfers_paired += 1;
                        }
                        this_run_ids.insert(counterpart_transaction_id);
                        continue;
                    }

                    // Tiers 2/3: search open (one-sided) transfer entries —
                    // see the design doc's "Pairing algorithm". `role` here
                    // is the slot *this* leg fills (i.e. the open row's
                    // missing side). Both tiers share one query
                    // (`find_open_transfer_candidate`); the returned
                    // prediction (what the open row's known leg itself
                    // decoded as *its* counterpart) decides which tier
                    // actually fired.
                    if let Some(counterpart_account_id) = counterpart_account_id {
                        if let Some((row_id, known_transaction_id, predicted)) = db
                            .find_open_transfer_candidate(
                                role,
                                counterpart_account_id,
                                leg.amount_minor,
                                &txn.posted_at,
                            )?
                        {
                            let pair_method = if predicted == Some(account.id) {
                                Some((TransferPairMethod::AmountDateMatch, 0.75))
                            } else if predicted == Some(counterpart_account_id) {
                                Some((
                                    TransferPairMethod::SelfReferenceMatch,
                                    SELF_REFERENCE_MATCH_CONFIDENCE,
                                ))
                            } else {
                                None
                            };
                            if let Some((pair_method, pair_confidence)) = pair_method {
                                db.complete_transfer_leg(
                                    row_id,
                                    &leg,
                                    pair_method,
                                    pair_confidence,
                                )?;
                                summary.transfers_paired += 1;
                                if !this_run_ids.contains(&known_transaction_id) {
                                    summary.transfers_backfilled += 1;
                                }
                                this_run_ids.insert(known_transaction_id);
                                continue;
                            }
                        }
                    }

                    // No match at any tier: record this leg alone.
                    db.insert_transfer_leg(&leg)?;
                }
            }
            Classification::CardPayment => {
                card_payment_candidates.push(txn.clone());
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
                        refunds_spend_entry_id: None,
                        classified_by: ClassifiedBy::Rule,
                        confidence: Some(confidence),
                        rule_name: Some(rule_name.to_string()),
                    },
                    txn.id,
                )?;
                summary.spend_entries_created += 1;
            }
            Classification::Income {
                counterparty,
                rule_name,
                confidence,
            } => {
                db.insert_income_entry_with_source(
                    &NewIncomeEntry {
                        occurred_on: txn.posted_at.clone(),
                        amount_minor: txn.amount_minor,
                        currency: txn.currency.clone(),
                        counterparty,
                        description: txn.description.clone(),
                        note: None,
                        classified_by: ClassifiedBy::Rule,
                        confidence: Some(confidence),
                        rule_name: Some(rule_name.to_string()),
                    },
                    txn.id,
                )?;
                summary.income_entries_created += 1;
            }
            Classification::Refund {
                counterparty,
                rule_name,
            } => {
                // Best-effort: the original charge may not be found (no
                // matching prefix/amount/date), or may itself predate the
                // spend ledger existing — either way the refund is still
                // recorded as its own spend entry, just with
                // `refunds_spend_entry_id` left `NULL`.
                let refunds_spend_entry_id = match &counterparty {
                    Some(prefix) => db
                        .find_refund_original(account.id, prefix, txn.amount_minor, &txn.posted_at)?
                        .map(|original_id| db.spend_entry_id_for_transaction(original_id))
                        .transpose()?
                        .flatten(),
                    None => None,
                };

                db.insert_spend_entry_with_source(
                    &NewSpendEntry {
                        occurred_on: txn.posted_at.clone(),
                        amount_minor: txn.amount_minor,
                        currency: txn.currency.clone(),
                        counterparty,
                        description: txn.description.clone(),
                        note: None,
                        category_id: None,
                        refunds_spend_entry_id,
                        classified_by: ClassifiedBy::Rule,
                        confidence: Some(0.7),
                        rule_name: Some(rule_name.to_string()),
                    },
                    txn.id,
                )?;
                summary.spend_entries_created += 1;
            }
            Classification::OutOfScope => {
                summary.out_of_scope += 1;
            }
        }
    }

    // Re-pairing sweep: re-attempts pairing for every currently open
    // (one-sided) transfer entry, independent of whether a new transaction
    // arrived this run. A row can become pairable purely because *another*
    // already-open row's own decode correctly names it — no new
    // transaction needs to arrive to trigger that (e.g. two legs
    // persisted, unpaired, by an earlier run, before a pairing tier that
    // could match them existed at all). See the design doc's "Pairing
    // algorithm". Symmetric to the inline tiers 2/3 above, just searching
    // among persisted open rows on both sides instead of a fresh leg
    // against persisted rows.
    let open_entries = db.open_transfer_entries()?;
    let open_by_id: std::collections::HashMap<crate::model::Id, &crate::model::OpenTransferEntry> =
        open_entries.iter().map(|o| (o.id, o)).collect();
    let mut swept: HashSet<crate::model::Id> = HashSet::new();
    for open in &open_entries {
        if swept.contains(&open.id) {
            continue;
        }
        // `known_role`/`known_account_id`: the side `open` already has.
        // `predicted_account_id`: `open`'s own decoded guess for its
        // missing side — used as the search key, exactly like a fresh
        // leg's own decode is in the inline tiers above.
        let (known_role, known_account_id, predicted_account_id) =
            if open.out_transaction_id.is_some() {
                (
                    TransferLegRole::Out,
                    open.out_account_id,
                    open.in_account_id,
                )
            } else {
                (TransferLegRole::In, open.in_account_id, open.out_account_id)
            };
        let (Some(known_account_id), Some(predicted_account_id)) =
            (known_account_id, predicted_account_id)
        else {
            continue;
        };
        let Some((other_id, other_known_transaction_id, other_predicted)) = db
            .find_open_transfer_candidate(
                known_role,
                predicted_account_id,
                open.amount_minor,
                &open.occurred_on,
            )?
        else {
            continue;
        };
        if other_id == open.id || swept.contains(&other_id) {
            continue;
        }
        let Some(other) = open_by_id.get(&other_id) else {
            continue;
        };
        // Tier determination mirrors the inline case: does the *other*
        // row's own decode correctly name `open` (mutual, tier 2) or does
        // it self-reference (tier 3)? `other`'s own known account is
        // `predicted_account_id` by construction (that's what we searched
        // by), so no extra lookup is needed to check the self-reference
        // case.
        let pair_method = if other_predicted == Some(known_account_id) {
            Some((TransferPairMethod::AmountDateMatch, 0.75))
        } else if other_predicted == Some(predicted_account_id) {
            Some((
                TransferPairMethod::SelfReferenceMatch,
                SELF_REFERENCE_MATCH_CONFIDENCE,
            ))
        } else {
            None
        };
        let Some((pair_method, pair_confidence)) = pair_method else {
            continue;
        };

        // `other`'s known side is the leg that completes `open`; only the
        // fields `complete_transfer_leg` actually reads (transaction id,
        // account, description, occurred_on, role) matter here.
        let other_role = if other.out_transaction_id.is_some() {
            TransferLegRole::Out
        } else {
            TransferLegRole::In
        };
        let other_description = if other_role == TransferLegRole::Out {
            other.out_description.clone()
        } else {
            other.in_description.clone()
        }
        .unwrap_or_default();
        let leg = NewTransferLeg {
            transaction_id: other_known_transaction_id,
            account_id: predicted_account_id,
            role: other_role,
            occurred_on: other.occurred_on.clone(),
            amount_minor: other.amount_minor,
            currency: other.currency.clone(),
            description: other_description,
            counterpart_sort_code: String::new(),
            counterpart_account_number: String::new(),
            counterpart_account_id: None,
            classified_by: ClassifiedBy::Rule,
            confidence: None,
            rule_name: None,
        };
        // Delete `other` first: `in_transaction_id`/`out_transaction_id`
        // are each UNIQUE across the table, so completing `open` with a
        // transaction id `other` still holds would momentarily violate
        // that constraint if done the other way round.
        db.delete_transfer_entry(other.id)?;
        db.complete_transfer_leg(open.id, &leg, pair_method, pair_confidence)?;
        swept.insert(open.id);
        swept.insert(other.id);
        summary.transfers_paired += 1;
        // Both sides necessarily predate this exact pairing operation —
        // both were already fully-persisted open rows before the sweep
        // began, so this is always a backfill.
        summary.transfers_backfilled += 1;
    }

    // A card payment is, by definition, an internal transfer (see
    // "Credit Card Payment" in doc/domain/ubiquitous-language.md) — it gets
    // a `transfer_entries` row like any other, never a spend entry. A
    // card-payment reference alone isn't a reliable match (see
    // `looks_like_card_payment_reference`'s doc comment), so the leg is only
    // ever fully paired once a date+amount match on a credit card account
    // confirms it; an unmatched candidate (e.g. the card statement for that
    // period hasn't been imported yet) is recorded as a one-sided row
    // instead, exactly like any other unpaired transfer leg, and retried on
    // future runs below rather than becoming a permanent spend entry — see
    // doc/implementation-notes/transfer-ledger-critique.md.
    for txn in card_payment_candidates {
        let leg = NewTransferLeg {
            transaction_id: txn.id,
            account_id: txn.account_id,
            role: TransferLegRole::Out,
            occurred_on: txn.posted_at.clone(),
            amount_minor: txn.amount_minor.abs(),
            currency: txn.currency.clone(),
            description: txn.description.clone(),
            counterpart_sort_code: String::new(),
            counterpart_account_number: String::new(),
            counterpart_account_id: None,
            classified_by: ClassifiedBy::Rule,
            confidence: Some(1.0),
            rule_name: Some("credit_card_payment".to_string()),
        };
        match db.find_card_payment_counterpart(txn.id, txn.amount_minor, &txn.posted_at)? {
            Some(counterpart_id) => {
                if let Some(counterpart) = db.get_transaction(counterpart_id)? {
                    db.create_paired_transfer(
                        &leg,
                        &counterpart,
                        TransferPairMethod::CreditCardPaymentMatch,
                        0.85,
                    )?;
                    summary.card_payments_matched += 1;
                }
            }
            None => {
                db.insert_transfer_leg(&leg)?;
                summary.card_payments_unmatched += 1;
            }
        }
    }

    // Retry every still-open card payment leg from an earlier run: the
    // credit card statement may have been imported since, and its
    // counterpart transaction was never excluded from re-matching (an
    // unmatched credit-card-side line just stays classified `OutOfScope`,
    // which writes nothing), so a fresh lookup can succeed where an earlier
    // run's couldn't.
    for (row_id, out_transaction_id, amount_minor, occurred_on) in db.open_card_payment_entries()? {
        if let Some(counterpart_id) =
            db.find_card_payment_counterpart(out_transaction_id, -amount_minor, &occurred_on)?
        {
            if let Some(counterpart) = db.get_transaction(counterpart_id)? {
                let leg = NewTransferLeg {
                    transaction_id: counterpart.id,
                    account_id: counterpart.account_id,
                    role: TransferLegRole::In,
                    occurred_on: counterpart.posted_at.clone(),
                    amount_minor: counterpart.amount_minor.abs(),
                    currency: counterpart.currency.clone(),
                    description: counterpart.description.clone(),
                    counterpart_sort_code: String::new(),
                    counterpart_account_number: String::new(),
                    counterpart_account_id: None,
                    classified_by: ClassifiedBy::Rule,
                    confidence: None,
                    rule_name: None,
                };
                db.complete_transfer_leg(
                    row_id,
                    &leg,
                    TransferPairMethod::CreditCardPaymentMatch,
                    0.85,
                )?;
                summary.card_payments_matched += 1;
            }
        }
    }

    Ok(summary)
}

/// Builds the household lookups `classify` needs from every imported
/// account's own sort code/account number plus any `extra_household_accounts`
/// (e.g. a partner's, never imported).
#[allow(clippy::type_complexity)]
fn build_household<'a>(
    accounts: &[Account],
    extra_household_accounts: &'a [HouseholdAccountRef],
) -> (HashSet<(String, String)>, Vec<(&'a str, &'a str, &'a str)>) {
    let mut household: HashSet<(String, String)> = accounts
        .iter()
        .filter_map(|a| Some((a.sort_code.clone()?, a.account_number.clone()?)))
        .collect();
    household.extend(
        extra_household_accounts
            .iter()
            .map(|a| (a.sort_code.clone(), a.account_number.clone())),
    );
    let household_names: Vec<(&str, &str, &str)> = extra_household_accounts
        .iter()
        .filter_map(|a| {
            Some((
                a.name.as_deref()?,
                a.sort_code.as_str(),
                a.account_number.as_str(),
            ))
        })
        .collect();
    (household, household_names)
}

#[derive(Debug, PartialEq)]
enum Classification {
    InternalTransfer {
        counterpart_sort: String,
        counterpart_account: String,
    },
    /// Looks like a bank-side payment to a credit card (cardholder name
    /// followed by a truncated PAN, e.g. `"MR JAMES BARRITT
    /// 49291328548900"` — see
    /// doc/kb/barclaycard/pdf-export-structure.md). The truncated PAN alone
    /// isn't a reliable matching key (it's missing digits and isn't stable
    /// across a reissue), so this is provisional pending a date+amount
    /// match against the credit card account in `run_derivation`.
    CardPayment,
    Spend {
        counterparty: Option<String>,
        rule_name: &'static str,
        confidence: f64,
    },
    /// Real-world money crossing the household boundary inward — see
    /// **Income Ledger** in doc/domain/ubiquitous-language.md.
    Income {
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
#[allow(clippy::too_many_arguments)]
fn classify(
    description: &str,
    trn_type: Option<&str>,
    amount_minor: i64,
    household: &HashSet<(String, String)>,
    household_names: &[(&str, &str, &str)],
    income_sources: &[(&str, crate::config::IncomeSourceKind)],
    registered_people: &[&str],
    reimbursement_sources: &[&str],
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

    // Rule 1c: a person-to-person NAME carrying no account digits at all
    // (Barclays shows these as either the full registered payee name or
    // "<Surname> <initial>" — see `matches_household_member_name`), matched
    // against a household member registered by name (`config.toml`'s
    // `household_accounts[].name`). Checked before the FT/card-payment rules
    // below so a household member's name always wins over guessing
    // "reimbursement"/"person_payment" — those rules exist for genuine
    // external people, not family.
    for (name, sort, account) in household_names {
        if matches_person_name(description, name) {
            return Classification::InternalTransfer {
                counterpart_sort: sort.to_string(),
                counterpart_account: account.to_string(),
            };
        }
    }

    // Rule 1d: Income Source — a registered external payer (employer, tax
    // authority) named at the very start of the description
    // (`config.toml`'s `income_sources`). Checked before Registered Person/
    // the generic suffix rules so a known payer always wins over a lower-
    // confidence guess. Inbound only — see the Income Source ubiquitous
    // language entry.
    if amount_minor > 0 {
        let upper = description.to_ascii_uppercase();
        for (name, kind) in income_sources {
            if starts_with_word(&upper, &name.to_ascii_uppercase()) {
                return Classification::Income {
                    counterparty: Some((*name).to_string()),
                    rule_name: kind.rule_name(),
                    confidence: kind.confidence(),
                };
            }
        }
    }

    // Rule 1e: Registered Person — an external individual (family/friend,
    // `config.toml`'s `registered_people`) recognised by name (see
    // `matches_person_name`). Unlike a household member (rule 1c), this is
    // NOT an internal transfer — the person is outside the household — but
    // an unexplained inbound payment from them defaults to a spend-ledger
    // **Reimbursement** rather than Income, since settling up a shared cost
    // is more common than a windfall gift; see the Registered Person
    // ubiquitous language entry.
    if amount_minor > 0 {
        for name in registered_people {
            if matches_person_name(description, name) {
                return Classification::Refund {
                    counterparty: Some((*name).to_string()),
                    rule_name: "person_reimbursement",
                };
            }
        }
    }

    // Rule 1f: a registered external institution/scheme (e.g. a health cash
    // plan like SimplyHealth, `config.toml`'s `reimbursement_sources`)
    // named at the very start of the description. Not a person, so plain
    // prefix matching (like Income Source) rather than `matches_person_name`.
    // A claim payout reverses spend already in the spend ledger (the
    // original medical/dental bill), so this is a Reimbursement, never
    // income — same reasoning as cashback.
    if amount_minor > 0 {
        let upper = description.to_ascii_uppercase();
        for name in reimbursement_sources {
            if starts_with_word(&upper, &name.to_ascii_uppercase()) {
                return Classification::Refund {
                    counterparty: Some((*name).to_string()),
                    rule_name: "claim_reimbursement",
                };
            }
        }
    }

    // Rule 2c: NAME looks like "<cardholder name> <truncated PAN>" — a
    // payment to a credit card. Only outbound money is treated this way;
    // see doc/kb/barclaycard/pdf-export-structure.md.
    if amount_minor < 0 && looks_like_card_payment_reference(description) {
        return Classification::CardPayment;
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
                // Reimbursement: inbound money paying back earlier spend
                // from a person outside the household — a sign-reversed
                // spend entry, never income. See the ubiquitous language
                // doc.
                Classification::Spend {
                    counterparty: None,
                    rule_name: "reimbursement",
                    confidence: 0.6,
                }
            };
        }
        // "BGC" = Bank Giro Credit — real Barclays OFX exports carry no
        // TRNTYPE at all for these (confirmed against real data: salary
        // deposits like "AZIMO LTD Pleo Technologies BGC" have an empty
        // TRNTYPE element), so the TRNTYPE-based DIRECTDEP rule below never
        // fires for them and they were silently falling through to
        // OutOfScope. Only reached once rules 1-2c above have ruled out an
        // internal (household) transfer, so this is real money entering the
        // household — matches the Income Ledger's definition exactly.
        // Confidence deliberately lower than the specific rules above
        // (Income Source, Registered Person): once those absorb the
        // explicable cases, whatever lands here genuinely needs a human
        // look — see the income-vs-refund classification proposal.
        Some("BGC") if amount_minor > 0 => {
            return Classification::Income {
                counterparty: None,
                rule_name: "bank_giro_credit",
                confidence: 0.5,
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
        // DIRECTDEP = salary/wages hitting the bank account — income. See
        // Delta: The Gap, Task 1.
        Some("DIRECTDEP") if amount_minor > 0 => Classification::Income {
            counterparty: None,
            rule_name: "direct_deposit",
            confidence: 0.9,
        },
        // "Other" is the Barclaycard PDF export's own type tag (Title
        // case — distinct from the generic OFX "OTHER" fallback used
        // elsewhere), seen so far only as Barclaycard Cashback: a rebate on
        // card spend that's already in the spend ledger, so it reverses
        // spend rather than being new money — a **Reimbursement**, not
        // income (see the income-vs-refund classification proposal, and
        // the Reimbursement ubiquitous language entry). Not a transfer
        // counterpart like "Payment received" is.
        // See doc/kb/barclaycard/pdf-export-structure.md.
        Some("Other") if amount_minor > 0 => Classification::Refund {
            counterparty: None,
            rule_name: "cashback",
        },
        // Any other DIRECTDEP (unexpected sign) and CASH (withdrawal) stay
        // out of scope — see the design doc's rule 7.
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
/// followed by `<sort code> <account no>` at (or very near) the *end* of the
/// description (e.g. `"ADVENTURE FUND 208794 33893693"`), rather than at the
/// start. The account number is sometimes truncated to 6 digits when the
/// label pushes the whole `NAME` field past Barclays' length limit (e.g.
/// `"SHARED BILLS ACCO 208794 231650"` — real account is `...23165086`, cut
/// to `231650`) — `household_contains` handles matching a truncated account
/// number against the full one on file. A short marker word (e.g. `"STO"`)
/// can also follow the account number — real data confirmed this despite
/// earlier assuming automated transfers never carry one (see
/// doc/developer-docs/transfer-detection.md, "The missing STO marker"),
/// tolerated the same way the leading-prefix rule already tolerates
/// trailing text via its `rest` return value. Returns `(sort_code,
/// account_no)`, which may itself be truncated.
fn parse_trailing_account_suffix(description: &str) -> Option<(&str, &str)> {
    let tokens: Vec<&str> = description.split_whitespace().collect();
    if let Some(pair) = trailing_account_pair(&tokens) {
        return Some(pair);
    }

    let (&marker, rest) = tokens.split_last()?;
    let is_short_marker =
        (1..=4).contains(&marker.len()) && marker.bytes().all(|b| b.is_ascii_alphabetic());
    if is_short_marker {
        return trailing_account_pair(rest);
    }
    None
}

/// The last two tokens of `tokens`, if they form a `<sort code> <account
/// no>` pair — the shared check behind `parse_trailing_account_suffix`'s two
/// attempts (with, and without, a trailing marker word).
fn trailing_account_pair<'a>(tokens: &[&'a str]) -> Option<(&'a str, &'a str)> {
    let is_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let &account = tokens.last()?;
    let &sort = tokens.get(tokens.len().checked_sub(2)?)?;
    if sort.len() == 6 && is_digits(sort) && (6..=8).contains(&account.len()) && is_digits(account)
    {
        Some((sort, account))
    } else {
        None
    }
}

/// A full (untruncated) card PAN is at most 16 digits (Visa/Mastercard) —
/// see `looks_like_card_payment_reference`. A truncated reference can be
/// shorter (however much of the 32-char `NAME` field survives after the
/// cardholder's name), but never longer, so this is only an upper bound.
const MAX_PAN_DIGITS: usize = 16;

/// Recognises Barclays' truncated-PAN `NAME` shape for a credit card
/// payment: one or more name words followed by a trailing digit run that
/// matches a known card network's IIN/BIN range (`known_card_network_prefix`)
/// — a truncated card PAN (see
/// doc/kb/barclaycard/pdf-export-structure.md, e.g. `"MR JAMES BARRITT
/// 49291328548900"`). No lower digit-count bound — how many digits Barclays
/// truncates to depends on how much of the `NAME` field the preceding name
/// text uses. There is an upper bound (`MAX_PAN_DIGITS`): more digits than a
/// full PAN can hold is structurally not a card number, whatever its prefix
/// looks like. `known_card_network_prefix` supplies the actual filtering
/// power on the short end, via its own 4-digit floor. Deliberately narrower
/// than `parse_trailing_account_suffix` (6+6..8 digits split across two
/// tokens) so the two shapes can't collide.
fn looks_like_card_payment_reference(description: &str) -> bool {
    let tokens: Vec<&str> = description.split_whitespace().collect();
    let Some((last, name_words)) = tokens.split_last() else {
        return false;
    };
    let is_pan_prefix = last.len() <= MAX_PAN_DIGITS
        && last.bytes().all(|b| b.is_ascii_digit())
        && known_card_network_prefix(last);
    let has_name = !name_words.is_empty()
        && name_words
            .iter()
            .all(|w| w.bytes().all(|b| b.is_ascii_alphabetic()));
    is_pan_prefix && has_name
}

/// Whether a card-number prefix falls in a known card network's IIN/BIN
/// range (ISO/IEC 7812 Major Industry Identifier + issuer ranges). Covers
/// the two 16-digit-PAN networks relevant to Barclaycard: Visa (Barclaycard
/// Rewards, seen in real data — see the KB article) and Mastercard (also
/// issued by Barclaycard for some products). Requires at least 4 digits to
/// have anything to check — without this, the Visa arm (`b'4'`) would match
/// on a bare single digit, since Visa's IIN range has no sub-range to
/// further narrow against (unlike Mastercard's two BIN ranges below).
fn known_card_network_prefix(pan_prefix: &str) -> bool {
    let Some(first4) = pan_prefix.get(..4).and_then(|s| s.parse::<u32>().ok()) else {
        return false;
    };
    match pan_prefix.as_bytes().first() {
        Some(b'4') => true,                            // Visa
        Some(b'5') => (5100..=5599).contains(&first4), // Mastercard (old range)
        Some(b'2') => (2221..=2720).contains(&first4), // Mastercard (new range)
        _ => false,
    }
}

/// Whether `description` names a registered person — a household member or
/// a **Registered Person** (e.g. `"ROMINA SCARAMAGLI"`) — for a
/// person-to-person `NAME`/description with no account digits at all.
/// Barclays' Faster Payments (`FT`) shows these in one of two forms
/// depending on payment direction, both derived from the same registered
/// `full_name`:
/// - the full name, when you're paying them (your saved payee nickname),
///   e.g. `"ROMINA SCARAMAGLI SHORTS FT"`;
/// - `"<Surname> <first initial>"`, when they're paying you (the sender name
///   Faster Payments echoes back), e.g. `"SCARAMAGLI R AMAZON FT"`.
///
/// A Bank Giro Credit (`BGC`) sender name is chosen by the *originating*
/// bank, not Barclays, and real data shows a third, different order for the
/// same "they're paying you" case: `"<first initial> <Surname>"`, e.g.
/// `"F CRICHTON NORWAY CAR BGC"` — the opposite order from Faster Payments'
/// echoed name. All three forms are checked.
///
/// Matches only at the very start of `description`, on a whole-word
/// boundary, so a coincidentally similar name (e.g. `"ARIA SCARAMAGLI-RE
/// CHASE BGC"` against the full name `"ARIA SCARAMAGLI"` alone, missing the
/// `-RE`) isn't mistaken for a match — register the name exactly as it
/// appears in the real description when the two differ.
pub fn matches_person_name(description: &str, full_name: &str) -> bool {
    let description = description.to_ascii_uppercase();
    let full_name = full_name.to_ascii_uppercase();
    let words: Vec<&str> = full_name.split_whitespace().collect();
    let (Some(&first), Some(&last)) = (words.first(), words.last()) else {
        return false;
    };
    let Some(initial) = first.chars().next() else {
        return false;
    };
    let surname_initial = format!("{last} {initial}");
    let initial_surname = format!("{initial} {last}");
    starts_with_word(&description, &full_name)
        || starts_with_word(&description, &surname_initial)
        || starts_with_word(&description, &initial_surname)
}

/// Whether `haystack` starts with `prefix` followed by a word boundary
/// (end-of-string or a space) — a plain `starts_with` would also match a
/// longer word that merely shares the prefix (e.g. `"SCARAMAGLI-RE"`
/// starting with `"SCARAMAGLI"`).
fn starts_with_word(haystack: &str, prefix: &str) -> bool {
    haystack
        .strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
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
            &[],
            &[],
            &[],
            &[],
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
    fn classifies_an_inbound_payment_from_a_named_household_member_as_internal() {
        // Barclays' sender-name form: "<Surname> <first initial>" — no
        // account digits at all, so only the name check can catch this.
        let household = HashSet::new();
        let result = classify(
            "SCARAMAGLI R AMAZON OASIS FT",
            Some("OTHER"),
            700,
            &household,
            &[("ROMINA SCARAMAGLI", "206325", "40531189")],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "206325".into(),
                counterpart_account: "40531189".into(),
            }
        );
    }

    #[test]
    fn classifies_an_outbound_payment_to_a_named_household_member_as_internal() {
        // The payee-nickname form: full name, no account digits.
        let household = HashSet::new();
        let result = classify(
            "ROMINA SCARAMAGLI SHORTS FT",
            Some("OTHER"),
            -11000,
            &household,
            &[("ROMINA SCARAMAGLI", "206325", "40531189")],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "206325".into(),
                counterpart_account: "40531189".into(),
            }
        );
    }

    #[test]
    fn does_not_match_a_similar_but_different_name() {
        // "ARIA SCARAMAGLI-RE" is a different, unrelated person — a naive
        // substring/prefix check on "SCARAMAGLI" alone would wrongly match.
        let household = HashSet::new();
        let result = classify(
            "ARIA SCARAMAGLI-RE CHASE BGC",
            Some("OTHER"),
            2600,
            &household,
            &[("ROMINA SCARAMAGLI", "206325", "40531189")],
            &[],
            &[],
            &[],
        );
        assert_ne!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "206325".into(),
                counterpart_account: "40531189".into(),
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
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "209934".into(),
                counterpart_account: "87654321".into(),
            }
        );
    }

    /// Real data (found during the transfer ledger migration) contradicted
    /// doc/developer-docs/transfer-detection.md's "missing STO marker"
    /// note: an automated transfer's trailing shape *can* carry a marker
    /// word after the account number — `parse_trailing_account_suffix` must
    /// tolerate it, or the receiving leg of every SHARED BILLS ACCO standing
    /// order (this exact real shape) never classifies as a transfer at all.
    #[test]
    fn classifies_a_trailing_account_transfer_with_a_marker_word_after_the_account_number() {
        let household = household_of(&[("208794", "23165086")]);
        let result = classify(
            "BARRITT J 208794 23165086 STO",
            Some("OTHER"),
            341500,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "208794".into(),
                counterpart_account: "23165086".into(),
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
            &[],
            &[],
            &[],
            &[],
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
            &[],
            &[],
            &[],
            &[],
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
            &[],
            &[],
            &[],
            &[],
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
            &[],
            &[],
            &[],
            &[],
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
            &[],
            &[],
            &[],
            &[],
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
        let result = classify(
            "SPOTIFY",
            Some("DIRECTDEBIT"),
            -999,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
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
    fn classifies_a_direct_deposit_as_income() {
        let household = HashSet::new();
        let result = classify(
            "SALARY",
            Some("DIRECTDEP"),
            150000,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::Income {
                counterparty: None,
                rule_name: "direct_deposit",
                confidence: 0.9,
            }
        );
    }

    #[test]
    fn classifies_a_credit_card_cashback_as_a_reimbursement() {
        // Cashback reverses spend that's already in the spend ledger, so it
        // is a Reimbursement, never Income — see the income-vs-refund
        // classification proposal.
        let household = HashSet::new();
        let result = classify(
            "CASHBACK",
            Some("Other"),
            500,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::Refund {
                counterparty: None,
                rule_name: "cashback",
            }
        );
    }

    #[test]
    fn classifies_an_unregistered_bank_giro_credit_as_low_confidence_income() {
        // Real Barclays OFX exports carry no TRNTYPE at all for salary BGC
        // credits, so this can't rely on the DIRECTDEP rule. With no
        // matching Income Source configured, this is the generic residual
        // rule — deliberately low confidence since it's not actually
        // understood which of salary/gift/refund this is.
        let household = HashSet::new();
        let result = classify(
            "AZIMO LTD Pleo Technologies BGC",
            None,
            597912,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::Income {
                counterparty: None,
                rule_name: "bank_giro_credit",
                confidence: 0.5,
            }
        );
    }

    #[test]
    fn classifies_a_registered_income_sources_bgc_credit_as_high_confidence_income() {
        let household = HashSet::new();
        let income_sources = [("AZIMO LTD", crate::config::IncomeSourceKind::Salary)];
        let result = classify(
            "AZIMO LTD Pleo Technologies BGC",
            None,
            597912,
            &household,
            &[],
            &income_sources,
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::Income {
                counterparty: Some("AZIMO LTD".into()),
                rule_name: "employment_income",
                confidence: 0.95,
            }
        );
    }

    #[test]
    fn classifies_a_tax_authority_income_source_as_income() {
        let household = HashSet::new();
        let income_sources = [("HMRC PAYE", crate::config::IncomeSourceKind::TaxAuthority)];
        let result = classify(
            "HMRC PAYE TNY10922037710501 BGC",
            None,
            39515,
            &household,
            &[],
            &income_sources,
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::Income {
                counterparty: Some("HMRC PAYE".into()),
                rule_name: "tax_refund",
                confidence: 0.8,
            }
        );
    }

    #[test]
    fn classifies_an_unexplained_payment_from_a_registered_person_as_a_reimbursement() {
        let household = HashSet::new();
        let registered_people = ["Fraser Crichton"];
        // Bank Giro Credit sender names use "<initial> <Surname>" order,
        // the opposite of Faster Payments' echoed "<Surname> <initial>".
        let result = classify(
            "F Crichton NORWAY CAR BGC",
            None,
            12521,
            &household,
            &[],
            &[],
            &registered_people,
            &[],
        );
        assert_eq!(
            result,
            Classification::Refund {
                counterparty: Some("Fraser Crichton".into()),
                rule_name: "person_reimbursement",
            }
        );
    }

    #[test]
    fn a_household_members_bgc_credit_is_still_an_internal_transfer() {
        // Rule 1c (household name match) runs before the BGC suffix rule,
        // so a household member's own inbound BGC is never misclassified
        // as income.
        let household = HashSet::new();
        let household_names = [("ROMINA SCARAMAGLI", "206325", "40531189")];
        let result = classify(
            "ROMINA SCARAMAGLI pizza BGC",
            None,
            1000,
            &household,
            &household_names,
            &[],
            &[],
            &[],
        );
        assert_eq!(
            result,
            Classification::InternalTransfer {
                counterpart_sort: "206325".to_string(),
                counterpart_account: "40531189".to_string(),
            }
        );
    }

    #[test]
    fn classifies_a_cash_withdrawal_as_out_of_scope() {
        let household = HashSet::new();
        let result = classify(
            "CASH WITHDRAWAL",
            Some("CASH"),
            -5000,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(result, Classification::OutOfScope);
    }

    #[test]
    fn run_derivation_creates_a_spend_entry_for_a_card_payment() {
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
            notes: None,
        })
        .expect("insert transaction");

        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
        assert_eq!(
            summary,
            DerivationSummary {
                spend_entries_created: 1,
                income_entries_created: 0,
                transfers_detected: 0,
                transfers_paired: 0,
                out_of_scope: 0,
                card_payments_matched: 0,
                card_payments_unmatched: 0,
                transfers_backfilled: 0,
            }
        );
    }

    #[test]
    fn run_derivation_is_idempotent() {
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
            notes: None,
        })
        .expect("insert transaction");

        run_derivation(&db, &[], &[], &[], &[]).expect("first derive");
        let second = run_derivation(&db, &[], &[], &[], &[]).expect("second derive");
        assert_eq!(
            second.spend_entries_created, 0,
            "must not double-derive an already-linked transaction"
        );
    }

    #[test]
    fn run_derivation_creates_an_income_entry_for_a_direct_deposit() {
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
            amount_minor: 150000,
            currency: "GBP".into(),
            description: "SALARY".into(),
            raw_description: None,
            trn_type: Some("DIRECTDEP".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert transaction");

        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
        assert_eq!(summary.income_entries_created, 1);
        assert_eq!(summary.spend_entries_created, 0);
        assert_eq!(summary.out_of_scope, 0);

        let month_rows = db.income_entries_for_month("2026-07").expect("query month");
        assert_eq!(month_rows.len(), 1);
        assert_eq!(month_rows[0].entry.amount_minor, 150000);
        assert_eq!(month_rows[0].account_id, account_id);

        let monthly = db.monthly_income_totals().expect("monthly totals");
        assert_eq!(monthly.len(), 1);
        assert_eq!(monthly[0].month, "2026-07");
        assert_eq!(monthly[0].income_minor, 150000);

        let second = run_derivation(&db, &[], &[], &[], &[]).expect("second derive");
        assert_eq!(
            second.income_entries_created, 0,
            "must not double-derive an already-linked transaction"
        );
        assert_eq!(
            db.income_entries_for_month("2026-07")
                .expect("query month again")
                .len(),
            1,
            "re-running derivation must not duplicate the income entry"
        );
    }

    #[test]
    fn run_derivation_links_a_refund_to_its_original_charges_spend_entry() {
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
            posted_at: "2026-02-26".into(),
            amount_minor: -4000,
            currency: "GBP".into(),
            description: "GARAGE SERVICES ON 26 FEB CPM".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert charge");
        db.insert_transaction(&NewTransaction {
            account_id,
            import_id: None,
            posted_at: "2026-02-28".into(),
            amount_minor: 4000,
            currency: "GBP".into(),
            description: "GARAGE SERVICES ON 28 FEB CRM".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert refund");

        run_derivation(&db, &[], &[], &[], &[]).expect("derive");

        let entries = db.list_spend_entries().expect("list spend entries");
        let charge = entries
            .iter()
            .find(|e| e.amount_minor == -4000)
            .expect("charge spend entry");
        let refund = entries
            .iter()
            .find(|e| e.amount_minor == 4000)
            .expect("refund spend entry");
        assert_eq!(refund.refunds_spend_entry_id, Some(charge.id));
        assert_eq!(charge.refunds_spend_entry_id, None);
    }

    #[test]
    fn run_derivation_pairs_both_legs_of_a_real_transfer() {
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
            notes: None,
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
            notes: None,
        })
        .expect("insert spending-side transaction");

        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
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

        // The per-month drill-down shows one row per transfer, not one per
        // leg: the schema itself guarantees this now — a paired transfer is
        // one `transfer_entries` row with both `out_*`/`in_*` sides filled
        // in, not two separate rows — see `Db::transfer_entries_for_month`.
        let month_rows = db
            .transfer_entries_for_month("2026-07")
            .expect("query month");
        assert_eq!(
            month_rows.len(),
            1,
            "a paired transfer must show as a single row, not one per leg"
        );
        assert_eq!(month_rows[0].out_account_id, Some(bills_account));
        assert_eq!(month_rows[0].amount_minor, 8900);
    }

    #[test]
    fn run_derivation_recognises_a_configured_partner_account_as_household() {
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
            notes: None,
        })
        .expect("insert transaction");

        let partner = HouseholdAccountRef {
            sort_code: "609934".into(),
            account_number: "99998888".into(),
            label: Some("Partner".into()),
            name: None,
        };
        let summary = run_derivation(&db, &[partner], &[], &[], &[]).expect("derive");
        assert_eq!(summary.spend_entries_created, 0);
        assert_eq!(summary.transfers_detected, 1);
    }

    #[test]
    fn classifies_a_payment_to_a_credit_card_as_a_provisional_card_payment() {
        let household = HashSet::new();
        let result = classify(
            "MR JAMES BARRITT 49291328548900",
            Some("OTHER"),
            -29581,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_eq!(result, Classification::CardPayment);
    }

    #[test]
    fn does_not_treat_inbound_money_as_a_card_payment() {
        let household = HashSet::new();
        let result = classify(
            "MR JAMES BARRITT 49291328548900",
            Some("OTHER"),
            29581,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_ne!(result, Classification::CardPayment);
    }

    #[test]
    fn does_not_treat_a_short_trailing_digit_as_a_card_payment() {
        let household = HashSet::new();
        let result = classify(
            "COUNCIL TAX REF 4",
            Some("OTHER"),
            -15000,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_ne!(
            result,
            Classification::CardPayment,
            "a bare short digit shouldn't be mistaken for a truncated PAN"
        );
    }

    #[test]
    fn does_not_treat_a_digit_run_longer_than_a_full_pan_as_a_card_payment() {
        let household = HashSet::new();
        // 17 digits — one more than a full 16-digit PAN could ever hold,
        // even though it starts with a Visa-shaped "4".
        let result = classify(
            "MR JAMES BARRITT 40000000000000000",
            Some("OTHER"),
            -15000,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_ne!(
            result,
            Classification::CardPayment,
            "more digits than a full PAN can hold can't be a card number"
        );
    }

    #[test]
    fn does_not_treat_an_unrelated_long_reference_number_as_a_card_payment() {
        let household = HashSet::new();
        let result = classify(
            "CORNWALL WILDLIFE 6060150000007",
            Some("OTHER"),
            -400,
            &household,
            &[],
            &[],
            &[],
            &[],
        );
        assert_ne!(
            result,
            Classification::CardPayment,
            "6... isn't a Visa/Mastercard IIN, even though it's long enough to look like one"
        );
    }

    #[test]
    fn run_derivation_pairs_a_card_payment_with_its_credit_card_counterpart() {
        let db = Db::open_in_memory().expect("open db");
        let current_account = db
            .insert_account(&NewAccount {
                name: "Jims Premier Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209912".into()),
                account_number: Some("12345678".into()),
            })
            .expect("insert current account");
        let credit_card_account = db
            .insert_account(&NewAccount {
                name: "Barclaycard".into(),
                institution: Some("Barclaycard".into()),
                account_type: AccountType::CreditCard,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
            })
            .expect("insert credit card account");

        db.insert_transaction(&NewTransaction {
            account_id: current_account,
            import_id: None,
            posted_at: "2026-06-01".into(),
            amount_minor: -29581,
            currency: "GBP".into(),
            description: "MR JAMES BARRITT 49291328548900".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert bank-side payment");
        db.insert_transaction(&NewTransaction {
            account_id: credit_card_account,
            import_id: None,
            posted_at: "2026-06-01".into(),
            amount_minor: 29581,
            currency: "GBP".into(),
            description: "PAYMENT, THANK YOU".into(),
            raw_description: None,
            trn_type: Some("Payment received".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert card-side payment");

        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
        assert_eq!(
            summary.card_payments_matched, 1,
            "the two legs should be matched by date+amount"
        );
        assert_eq!(
            summary.spend_entries_created, 0,
            "a matched card payment must not leak into spend"
        );

        let month_rows = db
            .transfer_entries_for_month("2026-06")
            .expect("query month");
        assert_eq!(
            month_rows.len(),
            1,
            "a credit card payment is one transfer_entries row like any other transfer"
        );
        assert_eq!(month_rows[0].out_account_id, Some(current_account));
        assert_eq!(month_rows[0].in_account_id, Some(credit_card_account));
        assert_eq!(
            month_rows[0].pair_method,
            Some(TransferPairMethod::CreditCardPaymentMatch)
        );

        let second =
            run_derivation(&db, &[], &[], &[], &[]).expect("second derive must be idempotent");
        assert_eq!(
            second.card_payments_matched, 0,
            "an already-paired card payment must not be rematched on a later run"
        );
        assert_eq!(
            db.transfer_entries_for_month("2026-06")
                .expect("query month again")
                .len(),
            1,
            "re-running derivation must not duplicate the transfer entry"
        );
    }

    #[test]
    fn run_derivation_records_an_unmatched_card_payment_as_an_open_transfer_entry() {
        let db = Db::open_in_memory().expect("open db");
        let current_account = db
            .insert_account(&NewAccount {
                name: "Jims Premier Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209912".into()),
                account_number: Some("12345678".into()),
            })
            .expect("insert current account");

        db.insert_transaction(&NewTransaction {
            account_id: current_account,
            import_id: None,
            posted_at: "2026-06-01".into(),
            amount_minor: -29581,
            currency: "GBP".into(),
            description: "MR JAMES BARRITT 49291328548900".into(),
            raw_description: None,
            trn_type: Some("OTHER".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert bank-side payment");

        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
        assert_eq!(summary.card_payments_matched, 0);
        assert_eq!(summary.card_payments_unmatched, 1);
        assert_eq!(
            summary.spend_entries_created, 0,
            "a card payment is an internal transfer, not spend, even before it's paired"
        );

        let month_rows = db
            .transfer_entries_for_month("2026-06")
            .expect("query month");
        assert_eq!(month_rows.len(), 1);
        assert_eq!(month_rows[0].out_account_id, Some(current_account));
        assert_eq!(
            month_rows[0].in_transaction_id, None,
            "no credit card account exists yet, so this stays visible as an unpaired leg"
        );

        // The credit card statement arrives later, in a separate import.
        let credit_card_account = db
            .insert_account(&NewAccount {
                name: "Barclaycard".into(),
                institution: Some("Barclaycard".into()),
                account_type: AccountType::CreditCard,
                currency: "GBP".into(),
                sort_code: None,
                account_number: None,
            })
            .expect("insert credit card account");
        db.insert_transaction(&NewTransaction {
            account_id: credit_card_account,
            import_id: None,
            posted_at: "2026-06-01".into(),
            amount_minor: 29581,
            currency: "GBP".into(),
            description: "PAYMENT, THANK YOU".into(),
            raw_description: None,
            trn_type: Some("Payment received".into()),
            external_id: None,
            notes: None,
        })
        .expect("insert card-side payment");

        let second = run_derivation(&db, &[], &[], &[], &[]).expect("second derive");
        assert_eq!(
            second.card_payments_matched, 1,
            "the previously-unmatched leg should be retried and paired now"
        );
        assert_eq!(second.card_payments_unmatched, 0);

        let month_rows = db
            .transfer_entries_for_month("2026-06")
            .expect("query month after backfill");
        assert_eq!(
            month_rows.len(),
            1,
            "the same row completes, no duplicate is created"
        );
        assert_eq!(month_rows[0].in_account_id, Some(credit_card_account));
    }

    /// The real gap this delta exists to close: a standing order between two
    /// household accounts where the receiving leg's trailing-suffix `NAME`
    /// shape (a label first, then sort/account) never starts with the
    /// origin's own prefix — so the description cross-reference (tier 1)
    /// can never match it, even though both sides still independently
    /// classify the other as their household counterpart, which the
    /// amount+date fallback (tier 2) uses to close the pair.
    #[test]
    fn run_derivation_pairs_an_automated_transfer_via_amount_date_fallback() {
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
                name: "Jims Premier Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209934".into()),
                account_number: Some("87654321".into()),
            })
            .expect("insert spending account");

        let origin_id = db
            .insert_transaction(&NewTransaction {
                account_id: spending_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: -3000000,
                currency: "GBP".into(),
                description: "209912 12345678 STO".into(),
                raw_description: None,
                trn_type: Some("REPEATPMT".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert origin leg")
            .expect("origin leg inserted");

        // The receiving leg's own NAME is the "label first, sort/account
        // last" trailing shape (see doc/developer-docs/transfer-detection.md)
        // — it doesn't *start with* the origin's prefix, so the description
        // cross-reference query can never match it, even though it does
        // correctly decode the origin as its own household counterpart.
        let receiving_id = db
            .insert_transaction(&NewTransaction {
                account_id: bills_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: 3000000,
                currency: "GBP".into(),
                description: "SHARED BILLS ACCO 209934 87654321".into(),
                raw_description: None,
                trn_type: Some("OTHER".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert receiving leg")
            .expect("receiving leg inserted");

        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
        assert_eq!(summary.spend_entries_created, 0);
        assert_eq!(summary.transfers_detected, 2);
        assert_eq!(summary.transfers_paired, 1);
        assert_eq!(
            summary.transfers_backfilled, 0,
            "both legs land in the same run, so nothing is retroactive"
        );

        let origin_counterpart = db
            .get_transfer_counterpart_transaction_id(origin_id)
            .expect("query counterpart")
            .expect("origin leg paired");
        assert_eq!(origin_counterpart, receiving_id);
        let receiving_counterpart = db
            .get_transfer_counterpart_transaction_id(receiving_id)
            .expect("query counterpart")
            .expect("receiving leg paired");
        assert_eq!(receiving_counterpart, origin_id);
    }

    /// The real gap tier 2 alone can't close: the receiving leg's `NAME`
    /// decodes to its *own* account rather than the sender's (the real
    /// SHARED BILLS ACCO shape — `"BARRITT J 208794 23165086 STO"` on the
    /// Bills Account's own transaction). Tier 2's mutual-agreement check
    /// requires the candidate's own decode to point back at the origin,
    /// which can never hold here, so tier 3 (self-reference + amount+date)
    /// is what has to pair it.
    #[test]
    fn run_derivation_pairs_a_self_referencing_automated_transfer_via_tier_3() {
        let db = Db::open_in_memory().expect("open db");
        let bills_account = db
            .insert_account(&NewAccount {
                name: "Bills Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("208794".into()),
                account_number: Some("23165086".into()),
            })
            .expect("insert bills account");
        let spending_account = db
            .insert_account(&NewAccount {
                name: "Jims Premier Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209934".into()),
                account_number: Some("87654321".into()),
            })
            .expect("insert spending account");

        let origin_id = db
            .insert_transaction(&NewTransaction {
                account_id: spending_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: -3415000,
                currency: "GBP".into(),
                description: "SHARED BILLS ACCO 208794 23165086".into(),
                raw_description: None,
                trn_type: Some("REPEATPMT".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert origin leg")
            .expect("origin leg inserted");

        // The receiving leg's own NAME decodes to its *own* sort/account
        // (208794 23165086 = the Bills Account itself), not the sender's —
        // tier 2's mutual check can never hold for this leg.
        let receiving_id = db
            .insert_transaction(&NewTransaction {
                account_id: bills_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: 3415000,
                currency: "GBP".into(),
                description: "BARRITT J 208794 23165086 STO".into(),
                raw_description: None,
                trn_type: Some("OTHER".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert receiving leg")
            .expect("receiving leg inserted");

        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
        assert_eq!(summary.spend_entries_created, 0);
        assert_eq!(summary.transfers_detected, 2);
        assert_eq!(summary.transfers_paired, 1);

        let origin_counterpart = db
            .get_transfer_counterpart_transaction_id(origin_id)
            .expect("query counterpart")
            .expect("origin leg paired");
        assert_eq!(origin_counterpart, receiving_id);
        let receiving_counterpart = db
            .get_transfer_counterpart_transaction_id(receiving_id)
            .expect("query counterpart")
            .expect("receiving leg paired");
        assert_eq!(receiving_counterpart, origin_id);
    }

    /// The exact real bug found migrating tier 3 onto the real database:
    /// two legs that were *both* already persisted, unpaired, from a run
    /// before a new pairing tier existed — neither is a "newly imported"
    /// leg, so nothing would ever re-trigger pairing for them if the
    /// pairing loop only looked at this run's freshly-classified
    /// candidates. `run_derivation` must re-attempt pairing over
    /// every currently-unpaired `transfer_entries` row on every run, not
    /// just ones tied to a transaction processed this run.
    #[test]
    fn run_derivation_pairs_two_already_persisted_unpaired_legs_on_a_later_run() {
        let db = Db::open_in_memory().expect("open db");
        let bills_account = db
            .insert_account(&NewAccount {
                name: "Bills Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("208794".into()),
                account_number: Some("23165086".into()),
            })
            .expect("insert bills account");
        let spending_account = db
            .insert_account(&NewAccount {
                name: "Jims Premier Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209934".into()),
                account_number: Some("87654321".into()),
            })
            .expect("insert spending account");

        let origin_id = db
            .insert_transaction(&NewTransaction {
                account_id: spending_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: -3415000,
                currency: "GBP".into(),
                description: "SHARED BILLS ACCO 208794 23165086".into(),
                raw_description: None,
                trn_type: Some("REPEATPMT".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert origin leg")
            .expect("origin leg inserted");
        let receiving_id = db
            .insert_transaction(&NewTransaction {
                account_id: bills_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: 3415000,
                currency: "GBP".into(),
                description: "BARRITT J 208794 23165086 STO".into(),
                raw_description: None,
                trn_type: Some("OTHER".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert receiving leg")
            .expect("receiving leg inserted");

        // Simulates both legs already having been recorded, unpaired, by a
        // run that predates tier 3 — bypasses `classify()`/derivation
        // entirely and writes the rows directly, the same shape the real
        // pre-tier-3 database was actually found in: one row per leg, each
        // missing the other side, the origin's own decode correctly
        // predicting Bills Account, the receiving leg's own decode
        // self-referencing (predicting itself).
        db.conn()
            .execute(
                "INSERT INTO transfer_entries
                    (occurred_on, amount_minor, currency,
                     out_transaction_id, out_account_id, out_description,
                     in_account_id,
                     classified_by, confidence, rule_name, classified_at)
                 VALUES
                    ('2026-07-01', 3415000, 'GBP',
                     ?1, ?2, 'SHARED BILLS ACCO 208794 23165086',
                     ?3,
                     'rule', 1.0, 'household_transfer', '2026-07-01T00:00:00.000Z')",
                rusqlite::params![origin_id, spending_account, bills_account],
            )
            .expect("seed origin leg's transfer_entries row directly");
        db.conn()
            .execute(
                "INSERT INTO transfer_entries
                    (occurred_on, amount_minor, currency,
                     in_transaction_id, in_account_id, in_description,
                     out_account_id,
                     classified_by, confidence, rule_name, classified_at)
                 VALUES
                    ('2026-07-01', 3415000, 'GBP',
                     ?1, ?2, 'BARRITT J 208794 23165086 STO',
                     ?2,
                     'rule', 1.0, 'household_transfer', '2026-07-01T00:00:00.000Z')",
                rusqlite::params![receiving_id, bills_account],
            )
            .expect("seed receiving leg's transfer_entries row directly");

        // Neither transaction is "pending" any more (both already have a
        // transfer_entries row), so this run detects nothing new — the
        // pairing must still happen by re-scanning persisted unpaired rows.
        let summary = run_derivation(&db, &[], &[], &[], &[]).expect("derive");
        assert_eq!(summary.transfers_detected, 0);
        assert_eq!(summary.transfers_paired, 1);
        assert_eq!(
            summary.transfers_backfilled, 1,
            "both legs predate this run, so this is a backfill"
        );

        let origin_counterpart = db
            .get_transfer_counterpart_transaction_id(origin_id)
            .expect("query counterpart")
            .expect("origin leg paired");
        assert_eq!(origin_counterpart, receiving_id);
    }

    /// A leg imported before its counterpart (cross-file import timing) must
    /// be recorded immediately, unpaired, and then have its pairing fields
    /// backfilled once the counterpart is imported and derivation runs
    /// again — not just get a fresh, separately-unpaired row for the new leg.
    #[test]
    fn run_derivation_backfills_an_earlier_unpaired_leg_retroactively() {
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
                name: "Jims Premier Account".into(),
                institution: Some("Barclays".into()),
                account_type: AccountType::Current,
                currency: "GBP".into(),
                sort_code: Some("209934".into()),
                account_number: Some("87654321".into()),
            })
            .expect("insert spending account");

        let origin_id = db
            .insert_transaction(&NewTransaction {
                account_id: spending_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: -3000000,
                currency: "GBP".into(),
                description: "209912 12345678 STO".into(),
                raw_description: None,
                trn_type: Some("REPEATPMT".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert origin leg")
            .expect("origin leg inserted");

        // First run: only the origin leg exists — recorded unpaired.
        let first = run_derivation(&db, &[], &[], &[], &[]).expect("first derive");
        assert_eq!(first.transfers_detected, 1);
        assert_eq!(first.transfers_paired, 0);
        assert!(db
            .get_transfer_counterpart_transaction_id(origin_id)
            .expect("query counterpart")
            .is_none());

        // The receiving leg lands in a later import.
        let receiving_id = db
            .insert_transaction(&NewTransaction {
                account_id: bills_account,
                import_id: None,
                posted_at: "2026-07-01".into(),
                amount_minor: 3000000,
                currency: "GBP".into(),
                description: "SHARED BILLS ACCO 209934 87654321".into(),
                raw_description: None,
                trn_type: Some("OTHER".into()),
                external_id: None,
                notes: None,
            })
            .expect("insert receiving leg")
            .expect("receiving leg inserted");

        let second = run_derivation(&db, &[], &[], &[], &[]).expect("second derive");
        assert_eq!(second.transfers_detected, 1, "only the new leg is pending");
        assert_eq!(second.transfers_paired, 1);
        assert_eq!(
            second.transfers_backfilled, 1,
            "the origin leg's row was persisted in an earlier run"
        );

        let origin_counterpart = db
            .get_transfer_counterpart_transaction_id(origin_id)
            .expect("query counterpart")
            .expect("origin leg now paired");
        assert_eq!(origin_counterpart, receiving_id);
    }
}
