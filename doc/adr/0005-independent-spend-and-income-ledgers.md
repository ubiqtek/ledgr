# 5. Independent spend and income ledgers, derived from raw transactions

Date: 2026-07-11

## Status

Accepted

## Context

Raw imported transactions mix real-world money events (payments to
merchants and people, income) with internal transfers between the
user's own accounts. The Rebel Finance method ledgr follows explicitly
excludes internal transfers, pension/investment contributions, and
credit-card repayments from "spending" — counting them double-counts
every purchase that is paid for by a matching transfer. Categorisation
and the "gap" (income − spending) must therefore operate on cleaned-up,
derived ledgers, not on raw statement lines.

Two shapes were considered for the derived layer:

1. **One ledger table** with `kind ∈ {spend, income}`, spend and income
   exposed as views. Attractive because the derivation pipeline,
   classification provenance (manual vs rule, confidence), and
   review/re-classification UX are built once.
2. **Two independent ledgers** — a spend ledger and an income ledger.

The single-table option was initially chosen, but it immediately
resisted naming: "spend ledger" was wrong because it held income,
"household ledger" smuggled in scope that was never agreed, and every
neutral alternative ("money events", "cash-flow ledger") was vague.
That naming difficulty was the domain talking: **spend and income are
different concepts, not two kinds of one concept.** The spend ledger is
about the whole act of classifying and categorising spending and
working to budgets. Income is a completely different domain involving
salary, tax, pensions — with essentially zero overlap in workflow,
taxonomy, or the questions asked of it. Forcing them into one table
optimised for machinery reuse at the cost of modelling two domains as
one.

## Decision

- **Raw transactions are immutable evidence.** They are never edited or
  categorised directly.
- **Two independent derived ledgers:**
  - The **spend ledger** (`spend_entries`) — one row per real-world
    outflow to a merchant or person. This is where categorisation,
    budgets, review queues, and Rebel Finance spending analysis live.
    Refunds and reimbursements are negative spend linked to the
    original entry, so they reduce spending rather than masquerading
    as income.
  - The **income ledger** (`income_entries`) — one row per real inflow
    (salary, interest, cashback). Its own, much simpler, lifecycle and
    its own taxonomy when one is needed.
- Each ledger entry links back to its source raw transaction(s) via its
  own provenance edge table. Internal transfers produce entries in
  neither ledger; they are recorded only as pairings between raw
  transactions (`transaction_links`, `relation = 'transfer'`).
- **Classification provenance is first-class** in both ledgers: every
  entry records whether it was classified by rule, matcher, or
  manually, with a confidence for automatic classifications. Manual
  classifications are never overwritten by re-derivation, which is
  otherwise re-runnable.
- The Rebel Finance gap is computed across the two ledgers:
  `SUM(income_entries) − SUM(spend_entries)` for a period.

Full design: `doc/implementation-notes/spend-ledger-design.md`.

## Consequences

- Each domain gets a schema shaped for itself: spend entries carry
  merchant/counterparty, category, notes, and budget-facing fields;
  income entries stay minimal and can grow tax/pension-facing fields
  without dragging the spend schema along.
- Queries, TUI screens, and analysis are naturally scoped — no
  `WHERE kind = 'spend'` foot-gun.
- The derivation pass writes to both ledgers; shared mechanics
  (provenance links, classified-by/confidence metadata, manual-wins
  rules) are a convention repeated across the two, accepted as the
  cost of keeping the domains independent. If the duplication grows
  painful, extract shared helpers in code — not a shared table.
- Reclassifying a credit from "income" to "refund of spend" is a
  delete-and-recreate across ledgers rather than a column flip; this
  is expected to be rare and always user-reviewed.
- The existing `transactions.category_id` column is redundant (only
  spend entries are categorised) and will be dropped.
- Transfer detection requires knowing the user's own accounts, so
  `accounts` gains sort code / account number columns plus a
  config-maintained list of own accounts not yet imported.
