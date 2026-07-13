# Transfer ledger — history

How the transfer ledger (`transfer_entries`) got to its current shape.
Kept separate from [transfer-ledger-design.md](transfer-ledger-design.md)
(the current-state reference) so that doc can stay a clean description of
what exists, not a narrative of how it was built. Companion ADR:
[0009 — Persisted ledgers, built at import, queried never re-derived](../adr/0009-persisted-ledgers-built-at-import.md).

## Why this delta was reopened (2026-07-13)

The Monthly Transfers screen's v1 build (`Screen::MonthlyTransfers`/
`Screen::TransferMonth`) re-derived its entire dataset from raw
transactions on every screen open (`derive::find_internal_transfers`),
with no persisted schema — a deliberate one-off exception to the pattern
`spend_entries` already established (raw transactions immutable, a
derivation pass at import time writes the derived view, the UI only ever
queries it). That exception was never actually agreed with the user and
never recorded as an ADR.

Investigating a naming bug on that screen surfaced a second, more
consequential problem: `Db::find_transfer_counterpart` (the query real
`ledgr import` derivation used to pair both legs of a transfer into a
`transaction_links` row) only reliably paired **manual transfers** — it
required the counterpart's own description to cross-reference this
transaction's account, true only for the leading-`NAME`-shape case.
**Automated transfers** (standing orders/direct debits, trailing-`NAME`
shape) failed silently: the receiving leg's description references its
*own* account, not the origin's. Confirmed against real data: 142
`'transfer'` `transaction_links` rows existed, zero covering 7 real
recurring SHARED BILLS ACCO ↔ Bills Account standing-order pairs, despite
both legs being independently and correctly classified as internal
transfers — a missing audit trail, not a spend-ledger correctness bug.

The user's decision: stop patching the live-derive approach, design a
real persisted transfer ledger analogous to `spend_entries`, and solve
the pairing gap properly at import time. This was also written up as ADR
0009, generalising to a standing principle for all future derived
relations (see the ADR).

## Task 2 — original design (2026-07-13)

The original design proposed a **two-tier** pairing algorithm:

1. **Description cross-reference** (existing `find_transfer_counterpart`
   query, unchanged) — works for manual transfers.
2. **Mutual classification match** (new) — equal-and-opposite amount, a
   ±3 day window, and *both* sides independently resolving the other as a
   household counterpart via `classify()`. The idea: no description
   cross-reference needed, so this should also close the automated-transfer
   gap.

This was believed sufficient to close the 7-pair SHARED BILLS ACCO gap.
It was not — see below.

## Task 3 — implementation found tier 2 doesn't close the motivating gap

Once real data was backfilled, tier 2 produced **zero** pairs. Root
cause: the SHARED BILLS ACCO ↔ Bills Account standing order's receiving
leg has a `NAME` field that references its **own** account
(`"BARRITT J 208794 23165086 STO"` — `208794 23165086` is the Bills
Account's own sort/account, not Jims Premier's), so its own `classify()`
decode resolves to itself. Tier 2's *mutual* agreement requirement can
never hold when one side is self-referential, no matter how the matching
window is tuned — this is a structural property of the data, not a
tuning problem.

Two options were considered: loosen tier 2's mutual-agreement check to
tolerate self-reference, or add a third, separately-tracked tier. Put to
the user directly (rather than decided unilaterally, since it changes a
stated safety property): the user chose a distinct third tier, for
traceability — so self-reference-derived pairs stay independently
auditable/reconsiderable later rather than being folded silently into
tier 2's confidence.

**Tier 3 — `self_reference_match`** was added: fires only after tiers 1
and 2 both fail; requires the *other* leg's own decoded counterpart to
equal *its own* account (the self-reference signature), exact opposite
amount, ±3 day window, unclaimed. Confidence `0.6`, below tier 2's
`0.75` (no mutual cross-check — self-reference + amount/date is the
whole signal). This closed all 7 target pairs (42 of 260 paired legs
overall used this tier on the real dataset).

## A second real bug: retroactive re-scan

Adding tier 3 to the code didn't immediately produce any new pairs on
the first real-data migration run. Root cause: once a leg has a
`transfer_entries` row, `pending_derivation_transactions` excludes it
from all future `ledgr import` runs. The pairing loop only considered
*that run's* freshly-classified legs as candidates — so the 7
already-persisted-but-unpaired SHARED BILLS ACCO legs from the earlier
session had no path back into consideration once tier 3 was added later,
since no *new* transaction was arriving to trigger a backfill for them.

Fixed by having the pairing loop iterate every currently-unpaired
persisted row (`Db::unpaired_transfer_entries()`), not just the current
run's candidates — see the design doc's "Pairing algorithm" section for
the resulting behaviour. This matters beyond this one rollout: any time a
new tier or rule is added later, previously-unpaired legs need a path
back into consideration on the very next `ledgr import`.

## Real-TUI bugs, found only once actually used (2026-07-13, after Task 3 was reported done)

Two further bugs surfaced only once the user looked at the live TUI —
neither was caught by `cargo build`/`test`/`clippy` or real-database SQL
checks, since both were specifically about how persisted data got
rendered, not about the persisted data itself:

1. **Counterparty display bug.** The per-month drill-down query only ever
   selected the raw `counterpart_sort_code`/`counterpart_account_number`
   digits decoded from a leg's own `NAME` field — never the actual
   resolved counterpart from `transfer_entries.counterpart_transaction_id`.
   For a self-referencing leg (exactly the case tier 3 exists to pair),
   this meant the UI showed the account as its own counterparty even
   though the database had the correct pairing all along. Fixed by
   joining `transfer_entries` to itself via `counterpart_transaction_id`
   to fetch the real counterpart's account — see the design doc's current
   query.

2. **Duplicate-row display.** The original design (both here and as
   built) showed **two** rows per paired transfer — one per leg, "each
   from its own account's perspective." This was a design-session
   judgement call that was never actually put to the user, and once
   built, looked like a confusing duplicate in the live TUI: two rows,
   same date and amount, identical "From/To" values. Asked directly, the
   user's decision: **one row per transfer.** The per-month drill-down
   now suppresses the incoming leg of a same-month pair — see the design
   doc's current behaviour.

**Lesson for future design sessions**: schema and naming decisions in
this delta were correctly flagged as open questions for the user; a
display/UX shape decision (one row per leg vs. per transfer) was not,
and turned out to be exactly the kind of thing worth asking about up
front rather than discovering as a live-TUI complaint.

## The real fix: `transfer_entries` was the wrong shape all along (2026-07-13, later the same session)

The "duplicate-row display" fix above was a **display-layer patch**: it
suppressed the incoming leg's row in the per-month drill-down query, but
the underlying schema still stored one row *per leg* (two rows per
paired transfer, `counterpart_transaction_id` linking them), and the
fix's own SQL comment described this suppression as showing "one row
**per transfer**, not per leg" — i.e. it *described* the transfer
correctly in the UI while the schema underneath still modelled a
transfer as two independent, mutually-referencing transaction records.

The user rejected this framing directly: a **transfer entry** is not one
transaction's own row — it is the *link between two transactions*. Two
rows that both individually claim to represent "the transfer," related
only by a foreign key, is exactly the shape that made the duplicate-row
bug possible in the first place; suppressing one of the two rows in a
query is a symptom fix, not a structural one.

**The actual correction**: `transfer_entries` was redesigned so that a
row *is* the transfer, not a leg of it — `out_transaction_id`/
`in_transaction_id` (plus matching `out_account_id`/`in_account_id`,
`out_sort_code`/`in_sort_code` etc.) directly on one row, either side
nullable until that leg's real transaction is found. This is a genuine
schema change, not a query patch:

- **Pairing became "fill in the missing side," not "link two rows."** A
  transaction reaching `Classification::InternalTransfer` either
  completes an existing one-sided row or creates a new one-sided row
  predicting its counterpart from its own decode.
- **A new re-pairing sweep was needed.** The original two-pass design
  (per-transaction classification, then a separate pass re-scanning
  unpaired rows) collapsed into inline pairing during classification —
  but that alone turned out to be insufficient: two rows that are
  *both* already-persisted and one-sided (neither tied to a transaction
  freshly classified this run) can only be merged by a genuinely
  separate sweep over all currently-open rows, comparing them against
  *each other* rather than against a fresh transaction. This is exactly
  the same shape of problem tier 3's original rollout hit (see above) —
  re-confirmed as a real, recurring requirement of this design, not a
  one-off.
- **The real local database needed re-migrating**, not just re-derived:
  the existing 300 one-row-per-leg rows (170 already fully paired as 130
  pairs, 40 permanently one-sided) were merged in Rust (not a single SQL
  statement — pairing legs by `counterpart_transaction_id`, splitting on
  `amount_minor`'s sign to determine which leg is `out_`/`in_`) into 170
  rows in the new shape, verified structurally against a scratch copy
  before touching the real file, backed up fresh, then applied for real.
  All 7 SHARED BILLS ACCO pairs confirmed merged correctly with
  `self_reference_match` intact.
- **Domain-language entries were downgraded.** "Transfer Ledger"/
  "Transfer Entry" had been marked `established` in
  `doc/domain/ubiquitous-language.md` earlier in this same session,
  attributed to "the user" — inaccurate, since the user had asked only
  for the *terminology to be made precise*, not signed off on the terms
  themselves, and the description of "Transfer Entry" written at the
  time still described the now-rejected one-row-per-leg shape. Corrected
  to `candidate`, attributed to the assistant, with the description
  rewritten to match the corrected shape.

**Lesson, reinforcing the one above**: the first correction (suppress
the duplicate row) was accepted as "fixed" without asking whether the
underlying data model matched the user's actual mental model of the
domain concept — it took the user explicitly naming the mismatch
("ONE TRANSFER ENTRY IS TWO TRANSACTIONS LINKED TOGETHER") to surface
that the display fix was papering over a schema problem, not solving it.

## Outcome

All real bugs found this session — tier 2's structural gap, the
retroactive re-scan gap, the two TUI display bugs, and the underlying
one-row-per-leg schema mismatch — are fixed and verified against the
real database — see the design doc's "Real worked examples" section for
concrete row data in the final shape. Full real-data migration figures
and `transaction_links` cleanup are recorded in `doc/planning/plan.md`
under "Delta: Transfer Ledger, Task 3."
