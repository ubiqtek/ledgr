//! Persistence for the derived spend ledger — `spend_entries` and
//! `spend_entry_sources` — plus the derived transfer ledger
//! (`transfer_entries`). See doc/implementation-notes/spend-ledger-design.md
//! and doc/implementation-notes/transfer-ledger-design.md. Classification
//! logic itself lives in `crate::derive`, kept separate from persistence so
//! the rules stay unit-testable without a database.

use super::Db;
use crate::model::{
    ClassifiedBy, Id, MonthlySpend, MonthlyTransfer, NewSpendEntry, NewTransferLeg,
    OpenTransferEntry, SpendEntry, SpendEntrySourceRole, SpendEntryWithAccount, Transaction,
    TransferEntry, TransferLegRole, TransferPairMethod,
};
use rusqlite::{params, OptionalExtension};

impl Db {
    /// Transactions not yet linked to a spend entry as their `source`, not
    /// yet recorded in `transfer_entries`, and not yet linked to an income
    /// entry either — the candidate set for a derivation pass. An
    /// out-of-scope transaction (cash) gains none of these, so it stays in
    /// this set forever; re-processing it is harmless (re-classifying it is
    /// a no-op). An internal transfer or a piece of income gets its own row
    /// on first derivation, so it's excluded from future runs — see
    /// doc/implementation-notes/transfer-ledger-design.md, "Integration into
    /// run_derivation".
    pub fn pending_derivation_transactions(&self) -> rusqlite::Result<Vec<Transaction>> {
        let mut stmt = self.conn().prepare(
            "SELECT t.id, t.account_id, t.import_id, t.posted_at, t.amount_minor,
                    t.currency, t.description, t.raw_description, t.trn_type, t.external_id,
                    t.notes
             FROM transactions t
             LEFT JOIN spend_entry_sources s
                    ON s.transaction_id = t.id AND s.role = 'source'
             LEFT JOIN transfer_entries te
                    ON te.out_transaction_id = t.id OR te.in_transaction_id = t.id
             LEFT JOIN income_entry_sources ies
                    ON ies.transaction_id = t.id
             WHERE s.transaction_id IS NULL AND te.id IS NULL AND ies.transaction_id IS NULL
             ORDER BY t.posted_at, t.id",
        )?;
        let rows = stmt.query_map([], Self::row_to_transaction)?;
        rows.collect()
    }

    /// Inserts a new spend entry and records `source_transaction_id` as its
    /// `source` provenance row — the common case for the derivation pass
    /// (one raw transaction -> one spend entry).
    pub fn insert_spend_entry_with_source(
        &self,
        new: &NewSpendEntry,
        source_transaction_id: Id,
    ) -> rusqlite::Result<Id> {
        let spend_entry_id = self.insert_spend_entry(new)?;
        self.insert_spend_entry_source(
            spend_entry_id,
            source_transaction_id,
            SpendEntrySourceRole::Source,
        )?;
        Ok(spend_entry_id)
    }

    pub fn insert_spend_entry(&self, new: &NewSpendEntry) -> rusqlite::Result<Id> {
        self.conn().execute(
            "INSERT INTO spend_entries
                (occurred_on, amount_minor, currency, counterparty, description, note,
                 category_id, refunds_spend_entry_id, classified_by, confidence, rule_name,
                 classified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                new.occurred_on,
                new.amount_minor,
                new.currency,
                new.counterparty,
                new.description,
                new.note,
                new.category_id,
                new.refunds_spend_entry_id,
                new.classified_by.as_str(),
                new.confidence,
                new.rule_name,
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Sets (or, given `None`, clears) a spend entry's free-text `note` —
    /// e.g. the user's own record of having looked into an unrecognised
    /// merchant and decided it's legitimate. Deliberately doesn't touch
    /// `classified_by`/`confidence`/`rule_name`: a note is just an
    /// annotation on top of whatever classified the entry, not a
    /// reclassification (that's still `classified_by = 'manual'`, unused by
    /// this method).
    pub fn set_spend_entry_note(&self, id: Id, note: Option<&str>) -> rusqlite::Result<()> {
        self.conn().execute(
            "UPDATE spend_entries SET note = ?1 WHERE id = ?2",
            params![note, id],
        )?;
        Ok(())
    }

    /// All spend entries, most recent first — the derivation pass's output,
    /// for `ledgr status`-style summaries and the future review queue TUI
    /// (Spend Ledger Task 3).
    pub fn list_spend_entries(&self) -> rusqlite::Result<Vec<SpendEntry>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, occurred_on, amount_minor, currency, counterparty, description,
                    note, category_id, refunds_spend_entry_id, classified_by, confidence,
                    rule_name, classified_at
             FROM spend_entries
             ORDER BY occurred_on DESC, id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let classified_by_str: String = row.get(9)?;
            Ok(SpendEntry {
                id: row.get(0)?,
                occurred_on: row.get(1)?,
                amount_minor: row.get(2)?,
                currency: row.get(3)?,
                counterparty: row.get(4)?,
                description: row.get(5)?,
                note: row.get(6)?,
                category_id: row.get(7)?,
                refunds_spend_entry_id: row.get(8)?,
                classified_by: ClassifiedBy::parse(&classified_by_str)
                    .unwrap_or(ClassifiedBy::Rule),
                confidence: row.get(10)?,
                rule_name: row.get(11)?,
                classified_at: row.get(12)?,
            })
        })?;
        rows.collect()
    }

    /// All spend entries for one calendar month (`month` as `YYYY-MM`),
    /// oldest first, alongside the id of the account each entry's source
    /// transaction was posted to — backs the TUI's per-month drill-down, so
    /// the user can eyeball whether anything that isn't real spend (a missed
    /// transfer, say) has slipped into the ledger, and verify spend against
    /// the account it actually came from. Joins through `spend_entry_sources`
    /// (`role = 'source'`) since `spend_entries` itself carries no
    /// `account_id` — see `SpendEntryWithAccount`'s doc comment.
    pub fn spend_entries_for_month(
        &self,
        month: &str,
    ) -> rusqlite::Result<Vec<SpendEntryWithAccount>> {
        let mut stmt = self.conn().prepare(
            "SELECT se.id, se.occurred_on, se.amount_minor, se.currency, se.counterparty,
                    se.description, se.note, se.category_id, se.refunds_spend_entry_id,
                    se.classified_by, se.confidence, se.rule_name, se.classified_at, t.account_id
             FROM spend_entries se
             JOIN spend_entry_sources ses ON ses.spend_entry_id = se.id AND ses.role = 'source'
             JOIN transactions t ON t.id = ses.transaction_id
             WHERE substr(se.occurred_on, 1, 7) = ?1
             ORDER BY se.occurred_on, se.id",
        )?;
        let rows = stmt.query_map([month], |row| {
            let classified_by_str: String = row.get(9)?;
            Ok(SpendEntryWithAccount {
                entry: SpendEntry {
                    id: row.get(0)?,
                    occurred_on: row.get(1)?,
                    amount_minor: row.get(2)?,
                    currency: row.get(3)?,
                    counterparty: row.get(4)?,
                    description: row.get(5)?,
                    note: row.get(6)?,
                    category_id: row.get(7)?,
                    refunds_spend_entry_id: row.get(8)?,
                    classified_by: ClassifiedBy::parse(&classified_by_str)
                        .unwrap_or(ClassifiedBy::Rule),
                    confidence: row.get(10)?,
                    rule_name: row.get(11)?,
                    classified_at: row.get(12)?,
                },
                account_id: row.get(13)?,
            })
        })?;
        rows.collect()
    }

    /// Net spend per calendar month, most recent first — backs the TUI's
    /// Monthly Gap screen. `occurred_on` is `YYYY-MM-DD`, so a plain string
    /// prefix groups correctly without needing SQLite's `strftime`.
    pub fn monthly_spend_totals(&self) -> rusqlite::Result<Vec<MonthlySpend>> {
        let mut stmt = self.conn().prepare(
            "SELECT substr(occurred_on, 1, 7) AS month, SUM(amount_minor)
             FROM spend_entries
             GROUP BY month
             ORDER BY month DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MonthlySpend {
                month: row.get(0)?,
                spend_minor: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    pub fn insert_spend_entry_source(
        &self,
        spend_entry_id: Id,
        transaction_id: Id,
        role: SpendEntrySourceRole,
    ) -> rusqlite::Result<()> {
        self.conn().execute(
            "INSERT OR IGNORE INTO spend_entry_sources (spend_entry_id, transaction_id, role)
             VALUES (?1, ?2, ?3)",
            params![spend_entry_id, transaction_id, role.as_str()],
        )?;
        Ok(())
    }

    /// The spend entry a transaction is the `source` of, if any — used to
    /// resolve a refund's original *transaction* (found via
    /// `find_refund_original`) to the original charge's own spend entry, so
    /// `refunds_spend_entry_id` can point at the ledger row directly rather
    /// than the raw transaction.
    pub fn spend_entry_id_for_transaction(
        &self,
        transaction_id: Id,
    ) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT spend_entry_id FROM spend_entry_sources
                 WHERE transaction_id = ?1 AND role = 'source'",
                params![transaction_id],
                |row| row.get(0),
            )
            .optional()
    }

    /// Best-effort search for a transfer's counterpart: the other leg,
    /// belonging to the account this transaction's `NAME` prefix points at,
    /// for the equal-and-opposite amount, whose own `NAME` prefix points
    /// back at this transaction's account, within a 3-day window (transfers
    /// are usually same-day; the extra slack covers weekends). See
    /// doc/implementation-notes/spend-ledger-design.md, "Transfer pairing".
    #[allow(clippy::too_many_arguments)]
    pub fn find_transfer_counterpart(
        &self,
        transaction_id: Id,
        own_sort_code: &str,
        own_account_number: &str,
        counterpart_sort_code: &str,
        counterpart_account_number: &str,
        amount_minor: i64,
        posted_at: &str,
    ) -> rusqlite::Result<Option<Id>> {
        let own_prefix = format!("{own_sort_code} {own_account_number}");
        self.conn()
            .query_row(
                "SELECT t.id
                 FROM transactions t
                 JOIN accounts a ON a.id = t.account_id
                 WHERE a.sort_code = ?1 AND a.account_number = ?2
                   AND t.amount_minor = ?3
                   AND t.description LIKE (?4 || '%')
                   AND t.id != ?5
                   AND julianday(t.posted_at) BETWEEN julianday(?6) - 3 AND julianday(?6) + 3
                 ORDER BY ABS(julianday(t.posted_at) - julianday(?6))
                 LIMIT 1",
                params![
                    counterpart_sort_code,
                    counterpart_account_number,
                    -amount_minor,
                    own_prefix,
                    transaction_id,
                    posted_at,
                ],
                |row| row.get(0),
            )
            .optional()
    }

    /// Whether `transaction_id` already belongs to a transfer entry, as
    /// either leg. Used when a tier-1 description match finds a counterpart
    /// transaction directly (`find_transfer_counterpart` searches raw
    /// `transactions`, independent of `transfer_entries`) — if that
    /// transaction already has a row (inserted earlier, one-sided), this
    /// leg completes it; otherwise a brand new fully-paired row is created.
    pub fn transfer_row_for_transaction(&self, transaction_id: Id) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT id FROM transfer_entries
                 WHERE out_transaction_id = ?1 OR in_transaction_id = ?1",
                params![transaction_id],
                |row| row.get(0),
            )
            .optional()
    }

    /// Inserts a new, one-sided transfer entry: only `leg`'s own side is
    /// known; the other side's account is recorded as a *prediction* (this
    /// leg's own decoded counterpart identity) until a later call
    /// (`complete_transfer_leg`) fills in the real transaction, if one is
    /// ever found. See doc/implementation-notes/transfer-ledger-design.md.
    pub fn insert_transfer_leg(&self, leg: &NewTransferLeg) -> rusqlite::Result<Id> {
        let (out_transaction_id, out_account_id, out_sort, out_account_no, out_description) =
            match leg.role {
                TransferLegRole::Out => (
                    Some(leg.transaction_id),
                    Some(leg.account_id),
                    None,
                    None,
                    Some(leg.description.as_str()),
                ),
                TransferLegRole::In => (
                    None,
                    leg.counterpart_account_id,
                    Some(leg.counterpart_sort_code.as_str()),
                    Some(leg.counterpart_account_number.as_str()),
                    None,
                ),
            };
        let (in_transaction_id, in_account_id, in_sort, in_account_no, in_description) =
            match leg.role {
                TransferLegRole::In => (
                    Some(leg.transaction_id),
                    Some(leg.account_id),
                    None,
                    None,
                    Some(leg.description.as_str()),
                ),
                TransferLegRole::Out => (
                    None,
                    leg.counterpart_account_id,
                    Some(leg.counterpart_sort_code.as_str()),
                    Some(leg.counterpart_account_number.as_str()),
                    None,
                ),
            };
        self.conn().execute(
            "INSERT INTO transfer_entries
                (occurred_on, amount_minor, currency,
                 out_transaction_id, out_account_id, out_sort_code, out_account_number, out_description,
                 in_transaction_id, in_account_id, in_sort_code, in_account_number, in_description,
                 classified_by, confidence, rule_name, classified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                leg.occurred_on,
                leg.amount_minor,
                leg.currency,
                out_transaction_id,
                out_account_id,
                out_sort,
                out_account_no,
                out_description,
                in_transaction_id,
                in_account_id,
                in_sort,
                in_account_no,
                in_description,
                leg.classified_by.as_str(),
                leg.confidence,
                leg.rule_name,
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Fills in the previously-empty side of an existing one-sided transfer
    /// entry (found either as an open row awaiting pairing, tier 2/3, or as
    /// a transaction that already has a row, tier 1) with `leg` and records
    /// how the pairing was made. The outgoing leg's date becomes the row's
    /// canonical `occurred_on` if this call supplies it.
    pub fn complete_transfer_leg(
        &self,
        row_id: Id,
        leg: &NewTransferLeg,
        pair_method: TransferPairMethod,
        pair_confidence: f64,
    ) -> rusqlite::Result<()> {
        match leg.role {
            TransferLegRole::Out => {
                self.conn().execute(
                    "UPDATE transfer_entries
                     SET out_transaction_id = ?1, out_account_id = ?2, out_description = ?3,
                         occurred_on = ?4, pair_method = ?5, pair_confidence = ?6
                     WHERE id = ?7",
                    params![
                        leg.transaction_id,
                        leg.account_id,
                        leg.description,
                        leg.occurred_on,
                        pair_method.as_str(),
                        pair_confidence,
                        row_id,
                    ],
                )?;
            }
            TransferLegRole::In => {
                self.conn().execute(
                    "UPDATE transfer_entries
                     SET in_transaction_id = ?1, in_account_id = ?2, in_description = ?3,
                         pair_method = ?4, pair_confidence = ?5
                     WHERE id = ?6",
                    params![
                        leg.transaction_id,
                        leg.account_id,
                        leg.description,
                        pair_method.as_str(),
                        pair_confidence,
                        row_id,
                    ],
                )?;
            }
        }
        Ok(())
    }

    /// Creates a brand-new, fully-paired transfer entry directly — tier 1's
    /// "counterpart transaction found, and it doesn't already have a row"
    /// case: both legs are known at once, so there's no one-sided
    /// intermediate state to create and then complete.
    pub fn create_paired_transfer(
        &self,
        leg: &NewTransferLeg,
        counterpart: &Transaction,
        pair_method: TransferPairMethod,
        pair_confidence: f64,
    ) -> rusqlite::Result<Id> {
        let (out_transaction_id, out_account_id, out_description, out_occurred_on) = match leg.role
        {
            TransferLegRole::Out => (
                leg.transaction_id,
                leg.account_id,
                leg.description.as_str(),
                leg.occurred_on.as_str(),
            ),
            TransferLegRole::In => (
                counterpart.id,
                counterpart.account_id,
                counterpart.description.as_str(),
                counterpart.posted_at.as_str(),
            ),
        };
        let (in_transaction_id, in_account_id, in_description) = match leg.role {
            TransferLegRole::In => (leg.transaction_id, leg.account_id, leg.description.as_str()),
            TransferLegRole::Out => (
                counterpart.id,
                counterpart.account_id,
                counterpart.description.as_str(),
            ),
        };
        self.conn().execute(
            "INSERT INTO transfer_entries
                (occurred_on, amount_minor, currency,
                 out_transaction_id, out_account_id, out_description,
                 in_transaction_id, in_account_id, in_description,
                 pair_method, pair_confidence,
                 classified_by, confidence, rule_name, classified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![
                out_occurred_on,
                leg.amount_minor,
                leg.currency,
                out_transaction_id,
                out_account_id,
                out_description,
                in_transaction_id,
                in_account_id,
                in_description,
                pair_method.as_str(),
                pair_confidence,
                leg.classified_by.as_str(),
                leg.confidence,
                leg.rule_name,
            ],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Every `transfer_entries` row still missing a side, oldest first —
    /// the candidate set for `run_derivation`'s re-pairing sweep. A
    /// row can become pairable purely because *another* already-open row's
    /// own decode names it, independent of any new transaction arriving —
    /// see doc/implementation-notes/transfer-ledger-design.md, "Pairing
    /// algorithm".
    pub fn open_transfer_entries(&self) -> rusqlite::Result<Vec<OpenTransferEntry>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, occurred_on, amount_minor, currency,
                    out_transaction_id, out_account_id, out_description,
                    in_transaction_id, in_account_id, in_description
             FROM transfer_entries
             WHERE out_transaction_id IS NULL OR in_transaction_id IS NULL
             ORDER BY occurred_on, id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(OpenTransferEntry {
                id: row.get(0)?,
                occurred_on: row.get(1)?,
                amount_minor: row.get(2)?,
                currency: row.get(3)?,
                out_transaction_id: row.get(4)?,
                out_account_id: row.get(5)?,
                out_description: row.get(6)?,
                in_transaction_id: row.get(7)?,
                in_account_id: row.get(8)?,
                in_description: row.get(9)?,
            })
        })?;
        rows.collect()
    }

    /// Removes a transfer entry outright — used only by the re-pairing
    /// sweep, to drop the now-redundant row once two separately-persisted
    /// open rows are merged into one (`complete_transfer_leg`) by that
    /// sweep.
    pub fn delete_transfer_entry(&self, id: Id) -> rusqlite::Result<()> {
        self.conn()
            .execute("DELETE FROM transfer_entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Tiers 2/3 of transfer pairing (see the design doc's "Pairing
    /// algorithm"): an open transfer entry — missing exactly `missing_role`,
    /// from this run or an earlier one, it makes no difference to this
    /// query — whose known side belongs to `known_account_id` (the account
    /// this leg's own decode identified as its counterpart), with a
    /// matching amount and a date within a 3-day window. Returns the row
    /// id, the known side's own transaction id (so the caller can tell
    /// whether that leg was itself classified this run, for
    /// `DerivationSummary::transfers_backfilled`'s bookkeeping), and that
    /// row's *prediction* for the missing side (the known leg's own
    /// decoded counterpart, recorded when the row was created) — the
    /// caller compares the prediction against its own account id to tell
    /// tier 2 (mutual agreement) from tier 3 (self-reference) apart; `None`
    /// means the prediction was never resolved to a tracked account at all.
    pub fn find_open_transfer_candidate(
        &self,
        missing_role: TransferLegRole,
        known_account_id: Id,
        amount_minor: i64,
        posted_at: &str,
    ) -> rusqlite::Result<Option<(Id, Id, Option<Id>)>> {
        let sql = match missing_role {
            // Missing the "out" side means the "in" side is known.
            TransferLegRole::Out => {
                "SELECT id, in_transaction_id, out_account_id
                 FROM transfer_entries
                 WHERE out_transaction_id IS NULL
                   AND in_transaction_id IS NOT NULL
                   AND in_account_id = ?1
                   AND amount_minor = ?2
                   AND julianday(occurred_on) BETWEEN julianday(?3) - 3 AND julianday(?3) + 3
                 ORDER BY ABS(julianday(occurred_on) - julianday(?3))
                 LIMIT 1"
            }
            TransferLegRole::In => {
                "SELECT id, out_transaction_id, in_account_id
                 FROM transfer_entries
                 WHERE in_transaction_id IS NULL
                   AND out_transaction_id IS NOT NULL
                   AND out_account_id = ?1
                   AND amount_minor = ?2
                   AND julianday(occurred_on) BETWEEN julianday(?3) - 3 AND julianday(?3) + 3
                 ORDER BY ABS(julianday(occurred_on) - julianday(?3))
                 LIMIT 1"
            }
        };
        self.conn()
            .query_row(
                sql,
                params![known_account_id, amount_minor, posted_at],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
    }

    /// The counterpart transaction id recorded for a transfer leg, if any —
    /// `None` both when the transaction has no `transfer_entries` row at all
    /// and when it has one but the other side isn't known yet.
    pub fn get_transfer_counterpart_transaction_id(
        &self,
        transaction_id: Id,
    ) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT CASE
                            WHEN out_transaction_id = ?1 THEN in_transaction_id
                            WHEN in_transaction_id = ?1 THEN out_transaction_id
                        END
                 FROM transfer_entries
                 WHERE out_transaction_id = ?1 OR in_transaction_id = ?1",
                params![transaction_id],
                |row| row.get(0),
            )
            .optional()
            .map(|outer: Option<Option<Id>>| outer.flatten())
    }

    /// Net transferred in/out per calendar month, most recent first — backs
    /// the TUI's Monthly Transfers screen. Same shape as
    /// `monthly_spend_totals`, but keeps out/in separate (not netted) per
    /// `MonthlyTransfer`'s doc comment. A row contributes to "out" whenever
    /// its outgoing side is known (paired or not) and to "in" whenever its
    /// incoming side is known — independent of each other, since a
    /// one-sided row still represents real money having moved on its known
    /// side.
    pub fn monthly_transfer_totals(&self) -> rusqlite::Result<Vec<MonthlyTransfer>> {
        let mut stmt = self.conn().prepare(
            "SELECT substr(occurred_on, 1, 7) AS month,
                    -SUM(CASE WHEN out_transaction_id IS NOT NULL THEN amount_minor ELSE 0 END),
                    SUM(CASE WHEN in_transaction_id IS NOT NULL THEN amount_minor ELSE 0 END)
             FROM transfer_entries
             GROUP BY month
             ORDER BY month DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MonthlyTransfer {
                month: row.get(0)?,
                transferred_out_minor: row.get(1)?,
                transferred_in_minor: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    /// All transfer entries for one calendar month (`month` as `YYYY-MM`),
    /// oldest first — backs the TUI's per-month drill-down. One row per
    /// `transfer_entries` row, i.e. one row **per real-world transfer**,
    /// not per leg — the schema itself now guarantees this (see the design
    /// doc), so no display-layer deduplication is needed.
    pub fn transfer_entries_for_month(&self, month: &str) -> rusqlite::Result<Vec<TransferEntry>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, occurred_on, amount_minor, currency,
                    out_transaction_id, out_account_id, out_sort_code, out_account_number, out_description,
                    in_transaction_id, in_account_id, in_sort_code, in_account_number, in_description,
                    pair_method, pair_confidence
             FROM transfer_entries
             WHERE substr(occurred_on, 1, 7) = ?1
             ORDER BY occurred_on, id",
        )?;
        let rows = stmt.query_map([month], |row| {
            let pair_method: Option<String> = row.get(14)?;
            Ok(TransferEntry {
                id: row.get(0)?,
                occurred_on: row.get(1)?,
                amount_minor: row.get(2)?,
                currency: row.get(3)?,
                out_transaction_id: row.get(4)?,
                out_account_id: row.get(5)?,
                out_sort: row.get(6)?,
                out_account: row.get(7)?,
                out_description: row.get(8)?,
                in_transaction_id: row.get(9)?,
                in_account_id: row.get(10)?,
                in_sort: row.get(11)?,
                in_account: row.get(12)?,
                in_description: row.get(13)?,
                pair_method: pair_method.map(|m| match m.as_str() {
                    "description_match" => TransferPairMethod::DescriptionMatch,
                    "amount_date_match" => TransferPairMethod::AmountDateMatch,
                    "credit_card_payment_match" => TransferPairMethod::CreditCardPaymentMatch,
                    _ => TransferPairMethod::SelfReferenceMatch,
                }),
                pair_confidence: row.get(15)?,
            })
        })?;
        rows.collect()
    }

    /// Best-effort search for a credit card bill payment's counterpart: any
    /// `CreditCard` account's transaction for the equal-and-opposite amount
    /// within a 3-day window (see
    /// doc/kb/barclaycard/pdf-export-structure.md's recommended
    /// date+amount matching strategy — deliberately doesn't key off the
    /// card number, which isn't stable across a reissue). With only one
    /// registered credit card today this can't yet be ambiguous between
    /// several cards; that will need revisiting once a second card
    /// (Credit Card Transaction Import Task 3) exists.
    pub fn find_card_payment_counterpart(
        &self,
        transaction_id: Id,
        amount_minor: i64,
        posted_at: &str,
    ) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT t.id
                 FROM transactions t
                 JOIN accounts a ON a.id = t.account_id
                 WHERE a.account_type = 'credit_card'
                   AND t.amount_minor = ?1
                   AND t.id != ?2
                   AND julianday(t.posted_at) BETWEEN julianday(?3) - 3 AND julianday(?3) + 3
                 ORDER BY ABS(julianday(t.posted_at) - julianday(?3))
                 LIMIT 1",
                params![-amount_minor, transaction_id, posted_at],
                |row| row.get(0),
            )
            .optional()
    }

    /// Every one-sided `transfer_entries` row still awaiting its credit card
    /// counterpart — i.e. a bank-side card payment debit recorded by
    /// `run_derivation` before the matching card statement had been
    /// imported. Retried on every subsequent run
    /// (`find_card_payment_counterpart` searches raw `transactions`, so a
    /// counterpart that's since been imported is found without needing the
    /// original leg reprocessed) rather than only at the moment the debit
    /// was first classified — see
    /// doc/implementation-notes/transfer-ledger-critique.md, "no
    /// retroactive completion — permanent double-count".
    pub fn open_card_payment_entries(&self) -> rusqlite::Result<Vec<(Id, Id, i64, String)>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, out_transaction_id, amount_minor, occurred_on
             FROM transfer_entries
             WHERE in_transaction_id IS NULL
               AND out_transaction_id IS NOT NULL
               AND rule_name = 'credit_card_payment'",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        rows.collect()
    }

    /// Best-effort search for the original charge a card refund pays back:
    /// same account, negative amount of the same magnitude, sharing the
    /// merchant prefix (text before " ON "), posted on or before the refund.
    /// `None` if nothing matches — the refund entry is still recorded (see
    /// the design doc: "linked ... when findable").
    pub fn find_refund_original(
        &self,
        account_id: Id,
        merchant_prefix: &str,
        refund_amount_minor: i64,
        posted_on_or_before: &str,
    ) -> rusqlite::Result<Option<Id>> {
        self.conn()
            .query_row(
                "SELECT id FROM transactions
                 WHERE account_id = ?1
                   AND amount_minor = ?2
                   AND description LIKE (?3 || '%')
                   AND posted_at <= ?4
                 ORDER BY posted_at DESC
                 LIMIT 1",
                params![
                    account_id,
                    -refund_amount_minor,
                    merchant_prefix,
                    posted_on_or_before
                ],
                |row| row.get(0),
            )
            .optional()
    }
}
