# 9. Persisted ledgers, built once at import — the UI only ever queries them

Date: 2026-07-13

## Status

Accepted

## Context

The Monthly Transfers screen (built under TUI Analysis Views Task 4, moved
into Delta: Transfer Ledger on reopening) was designed and built as a
**read-only preview**: it re-derives its entire dataset from raw
transactions on every screen open, by calling
`derive::find_internal_transfers` — itself a thin wrapper re-running the
same `classify()` logic the real `ledgr import` derivation pass uses,
deliberately with **no new persisted schema**. That choice was written into
the session's "What's Next" entry and treated as settled at the time, but
was never actually agreed with the user in those terms and was never
recorded as an ADR — a gap only noticed while investigating an unrelated
naming bug on that same screen.

Investigating that bug surfaced a second, more consequential problem:
`Db::find_transfer_counterpart` — the query real `ledgr import` derivation
already uses to pair both legs of a transfer into a `transaction_links` row
— only reliably pairs **manual** transfers (both sides' `NAME` field
cross-reference each other's account number). It silently fails to pair
**automated** transfers (standing orders/direct debits), where the
receiving leg's description references its *own* account instead of the
sender's. Confirmed against the real database: 142 `'transfer'`
`transaction_links` rows exist, covering zero of 7 real recurring SHARED
BILLS ACCO ↔ Bills Account standing-order pairs — despite both legs being
independently and correctly classified as internal transfers, so spend
ledger correctness was never at risk. This is a missing-audit-trail gap,
not a wrong-numbers bug — but it is exactly the kind of gap that a
live-recompute design makes easy to miss: nothing fails loudly when a
pairing query's assumptions don't hold for a whole category of real data,
because there's no persisted state to check against, only whatever the
query happens to return this time.

Both findings point at the same root cause: **deriving a relation live, on
every read, instead of once, at write time, hides exactly the kind of gap
that matters** — a query that's silently wrong for a subset of cases looks
identical, from the UI's perspective, to "there's nothing there," on every
single screen open, forever. `spend_entries` (ADR 0005,
`doc/implementation-notes/spend-ledger-design.md`) already established the
opposite pattern for spend — raw transactions are immutable evidence, a
derivation pass at import time writes the derived ledger, the UI only ever
queries it — and that pattern would have caught this class of bug the same
way it already surfaces spend-classification issues: as a visible,
queryable, persisted gap (an unpaired row) rather than a silent one.

## Decision

**Every derived relation the UI needs is built once, during `ledgr
import`'s derivation pass, into its own persisted table. The UI (`app.rs`,
`ui.rs`) never re-derives a relation live — it only ever queries what
derivation already wrote.** This generalises the pattern `spend_entries`
established for spend/income classification to every future derived
ledger, not just that one table.

Concretely, in support of this ADR:

- **Transfer ledger** (first concrete application, this delta): a new
  `transfer_entries` table, one transfer entry per transaction classified
  as an internal transfer, written during `derive_spend_entries`, with
  classification provenance (mirroring `spend_entries`) and a
  multi-tier pairing algorithm that closes the automated-transfer gap
  above (full design, including the final tier count and real-data
  results:
  `doc/implementation-notes/transfer-ledger-design.md`). The
  Monthly Transfers screen migrates from calling `derive::classify()` live
  on every screen open to querying `transfer_entries` (tracked as Task 3 of
  Delta: Transfer Ledger — schema/derivation build-out and migration of the
  real database is separate work from this decision).
- **Income ledger** (expected next application, not yet designed — Delta:
  The Gap, Task 1): when built, `income_entries` should follow the same
  rule — derived once at import, queried thereafter — rather than being
  designed fresh on its own merits and re-litigating this question.
- **Any future derived relation** (reconciliation checks, spend
  enrichment, or anything else that reads "what does the data mean," as
  opposed to "what did the bank report") defaults to this pattern unless a
  specific reason is found not to follow it, in which case that reason
  should be written down (an ADR or a design doc note), not silently
  assumed.

This does not mean every read-only view needs a table — `ledgr status`'s
account summaries, for instance, read `accounts`/`balance_snapshots`
directly, because those aren't *derived* relations, they're direct
projections of already-persisted facts. The rule applies specifically to
relations that require re-running classification/matching logic (rules,
pattern matching, cross-referencing) to reconstruct — the class of thing
`classify()` and transfer pairing both are.

## Consequences

- **A gap in a derivation rule becomes visible and queryable** (an
  unpaired or missing row, checkable with a plain `SELECT`) instead of
  silently absent from every live recomputation — the concrete benefit
  that motivated this decision. Delta: Reconciliation (added the same
  session) is a direct beneficiary: reconciliation checks are much easier
  to write and trust against a set of persisted derived tables than
  against relations that only exist transiently inside a screen-open call.
- **Every new derived relation costs a table, a derivation step, and (per
  the spend ledger precedent) a provenance/confidence convention** —
  more upfront design than "just recompute it in the TUI layer," which is
  real cost, not free. Accepted as the same trade-off ADR 0005 already
  accepted for spend: proportionate to a genuine data-quality problem
  (evidenced twice now — spend/income conflation, and transfer pairing),
  not speculative engineering.
- **Re-derivation must stay idempotent and support retroactive
  completion**, not just idempotent re-insertion. `spend_entries` only
  ever needed the former (an already-derived transaction is skipped on
  the next run); the transfer ledger is the first case needing the
  latter (an earlier-imported leg's pairing can only be completed once
  its counterpart is imported later, requiring an `UPDATE` to the earlier
  row, not just an `INSERT` for the new one) — future derived relations
  should check whether they have the same shape of problem rather than
  assuming idempotent insert-only derivation is always sufficient.
- **The TUI's job narrows further**: `app.rs`/`ui.rs` read state and
  render it; they must not contain classification/matching logic of their
  own, even for "just a read-only preview" screens — the Monthly Transfers
  screen's original design was exactly this kind of exception, and it's
  the one this ADR closes off. A future screen that wants to preview a
  not-yet-persisted derivation should instead motivate building that
  derivation properly, not re-implement it inline.
- **Migration discipline**: each concrete application (this transfer
  ledger, the future income ledger, anything else) needs its own rollout
  note for the real local `ledgr.db` — schema addition is free
  (`CREATE TABLE IF NOT EXISTS`), but backfilling a derived table over
  already-imported data is a one-time pass worth verifying against known
  real-data facts (e.g. this delta's known 7-pair gap), the same way this
  ADR's motivating bug was found by checking real data directly rather
  than trusting the code to be correct by construction.
