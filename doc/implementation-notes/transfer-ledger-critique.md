# Transfer ledger / `transaction_links` — design critique

**Status: §1 and §3 resolved 2026-07-13 (Delta: Transfer Ledger, Task 4)**
— credit card payment matching now writes `transfer_entries`
(`pair_method = 'credit_card_payment_match'`), the `relation='transfer'`
writer and its legacy rows are gone, and unmatched payments no longer
permanently double-count as spend (see
[transfer-ledger-design.md](transfer-ledger-design.md), "Credit card
payment matching"). §2's narrower `transaction_links` (refund-only, still
write-only) and §4/§5's remaining cleanup are unaffected — see Delta:
Transfer Ledger, Task 5 in `doc/planning/plan.md`.

External review (2026-07-13, fable-model agent, read-only — no files
edited), requested after the user found the `transaction_links`
explanation in
[transfer-ledger-design.md](transfer-ledger-design.md) confusing.
Reproduced here verbatim (lightly reformatted) so the finding has a
durable home instead of living only in conversation. Directly informed
the now-complete **Delta: Transfer Ledger, Task 4** work — see
`doc/planning/plan.md`.

## 1. The flagged confusion is a symptom: credit card payment matching is an internal transfer that never migrated to `transfer_entries`

This is the root finding. The project's own ubiquitous language already
settles the question (`doc/domain/ubiquitous-language.md`, "Internal
Transfer" — established, user-confirmed):

> "Internal Transfer — Money moving between the user's own accounts
> (**including credit-card repayments**) ... does produce a **transfer
> entry** in the **transfer ledger**."

The code contradicts this. `Classification::CardPayment`
(`src/derive.rs`) never touches `transfer_entries`; a matched payment
writes `transaction_links` with `relation='transfer'`, `confidence=0.85`
instead. Consequences:

- The Monthly Transfers screen queries only `transfer_entries`, so
  credit card repayments — typically the largest monthly internal
  transfer — are invisible in the transfer ledger, contrary to the
  agreed domain definition.
- The two mechanisms overlap because they *are the same concept*:
  pairing two legs of an intra-household money movement. Bank debit =
  out-leg, card-account credit = in-leg — both are real transactions in
  tracked accounts, exactly the shape `transfer_entries` was built for.

**Recommendation:** migrate credit card payment pairing into
`transfer_entries` (a new `pair_method` value, e.g.
`'credit_card_payment_match'`), retire the `relation='transfer'` writer
entirely, and one-time-delete the legacy rows (both the old 0.9
transfer-pairing rows, already superseded, and the 0.85 card-payment
rows). The confusing paragraph in the design doc then ceases to exist
rather than needing better prose.

## 2. `transaction_links` is write-only — nothing in the codebase reads it

Verified: there is no `SELECT ... FROM transaction_links` anywhere in
`src/`. Writers: refund links (`LinkRelation::Refund`, confidence
`NULL`) and credit card payments (`LinkRelation::Transfer`,
confidence `0.85`). `duplicate_of` and `related` have **zero** writers.
`TransactionLink` in `src/model.rs` is never constructed from a row.

So the 0.9-vs-0.85 distinction the flagged passage laboured over
mattered to no code path at all — only to a human poking the database,
where it's maximally fragile: `confidence` is documented as a quality
measure, not a type tag, and retuning either constant would silently
destroy the discrimination.

After the migration in §1, `transaction_links`' only live purpose is the
refund audit trail. Either give refunds a reader (e.g. a "show original
charge" drill-down) or shrink the table/`CHECK` to `refund` only and say
plainly in `schema.sql` that it's audit-only. Keeping `duplicate_of`/
`related` in the `CHECK` with no writer is speculative surface area.

**So: is credit card payment matching "the only reason `transaction_links`
exists"? No — refund linking is a second, independent live writer.**
Migrating credit card payments out shrinks the table to one purpose
(refunds); it doesn't make the table killable outright.

## 3. Credit card payment derivation has two real bugs, both cured by moving it into `transfer_entries`

- **Endless reprocessing.** `pending_derivation_transactions` excludes
  only transactions with a `spend_entry_sources` row or a
  `transfer_entries` row. A *matched* credit card payment gains
  neither today, so it is re-classified and re-matched on every `ledgr
  import`, and `DerivationSummary.card_payments_matched` is
  re-incremented each run — the import summary over-reports forever.
  The `INSERT OR IGNORE` on `transaction_links` hides this rather than
  fixing it.
- **No retroactive completion — permanent double-count.** An unmatched
  credit card payment (statement not yet imported) becomes a
  confidence-0.5 spend entry. Once the card statement arrives, that
  transaction now *has* a `spend_entry_sources` row, so it's excluded
  from `pending_derivation_transactions` and **never re-matched**: the
  provisional spend entry is permanent, double-counting the repayment
  against the card's own line items until manually noticed (no review
  UX exists yet). `transfer_entries` already solved exactly this shape
  for other transfers (one-sided rows, later completion, the sweep) —
  credit card payments just never got hooked into it.

## 4. Three linking patterns — two are justified, the third is the leftover

- `spend_entry_sources`: derived-row → N source transactions, junction
  table with `role`. Right shape for 1-to-many provenance.
- `transfer_entries`: exactly-two slots inlined as `out_*`/`in_*`
  columns with per-slot null-until-known state. Right shape, and the
  design doc's "Why not extend `transaction_links`" section argues it
  well — genuine justification, not drift.
- `transaction_links`: the pre-ADR-0009 generic edge table, now (after
  §1's migration) carrying one live relation (refund) that nothing
  reads.

The answer to "which do I reach for when adding a new derived relation"
is derivable — derived ledger row with provenance → its own table;
provenance of that row → a sources junction — but currently only
discoverable by reading all three plus the design doc. A short note in
`schema.sql` above `transaction_links` stating its narrowed, post-0009
role would help even before any code changes.

## 5. Vocabulary and stale-pointer debt

- **"Credit Card Payment" is now recorded** in
  `doc/domain/ubiquitous-language.md` (added 2026-07-13, candidate
  status) — this closes the vocabulary gap that made the flagged
  passage so hard to write clearly in the first place.
- **Stale comments pointing at the retired mechanism:**
  `src/db/schema.sql` (the `spend_entries` table comment, "Internal
  transfers ... produce no row here at all (see transaction_links,
  relation='transfer')") and `src/model.rs` (`SpendEntry` doc comment,
  "see `TransactionLink` with `LinkRelation::Transfer`") both still cite
  the superseded pairing; they should cite `transfer_entries` once §1
  lands. `LinkRelation::Transfer` is itself now a misnomer — its only
  writer is credit card payments.

## Bottom line

This was not a documentation problem — the fix is not better prose. The
confidence-as-discriminator hack, the write-only edge table, the
re-run miscounting, the unmatched-payment double-count, and the
originally-flagged paragraph are all downstream of one omission: credit
card payment pairing is, by the project's own established domain
definition, an internal transfer, and it was left behind when transfers
got their proper table. Migrate it into `transfer_entries`, purge the
legacy `relation='transfer'` rows, shrink `transaction_links` to its one
honest remaining job (refund audit), and treat "Credit Card Payment" as
an agreed term.
