# 7. Model assets and liabilities as accounts with (manual or imported) balance snapshots

Date: 2026-07-12

## Status

Accepted

## Context

Delta: The Gap exists to compute the Rebel Finance "gap" (income −
spending) without waiting for spend categorisation. Scoping its
income-ledger and gap-calculation tasks surfaced a related but distinct
need: the user also wants to see net worth, which means capturing
assets and liabilities beyond day-to-day current/savings accounts. The
user's actual list, beyond banking, is short and closed: their house,
the mortgage against it, pension funds, and the monthly credit card
balance (already covered once Credit Card Statement Import lands — it's
just an account like any other).

Each of the three new items has a different data-availability shape:

- **Mortgage** — may be periodically downloadable as a statement, format
  TBD.
- **Pension** — the provider produces some reports, but the user also
  wants to be able to record a value manually, without waiting for or
  parsing a report every time.
- **House** — no feed exists or is expected; purely a manually-recorded
  value, updated occasionally (e.g. after a periodic valuation).

This raised the bigger question of whether ledgr should move towards
explicit double-entry bookkeeping now — journal entries as the source
of truth, with `spend_entries`/`income_entries` becoming projections
over them (see Delta: Double-Entry Accounting, and the design doc's
"Future: double-entry compatibility" section, which already maps the
current model onto that future cleanly without being blocked by it).

## Decision

Do not pivot to double-entry now. Extend the existing `accounts` +
`balance_snapshots` machinery — already built for reconstructing a bank
account's balance at an arbitrary date from anchor balances
(`Db::balance_as_of`) — to cover assets and liabilities generally:

- Add new `AccountType` variants for these asset/liability kinds (exact
  set, e.g. `Property` / `Mortgage`, to be settled when implemented —
  fine-grained categorisation of account types can happen later without
  affecting this decision).
- `balance_snapshots` already stores `(account_id, import_id,
  balance_minor, as_of)` with `import_id` optional — it was already
  general enough to support a snapshot with no import behind it at
  all. Add a way to insert one **manually** (a new CLI command, not a
  parser) for accounts with no automated feed (the house always; the
  pension whenever the user prefers a manual update over waiting for a
  report).
- When a periodic download or report does exist (mortgage statement,
  pension report), it becomes a new `ImportFileParser` implementation
  like any other institution, reusing `ImportFileParser::
  balance_snapshot()` exactly as `BarclaysOfxParser` already does for
  `LEDGERBAL` — no new mechanism required.
- Net worth becomes: latest balance per account (via
  `latest_balance_snapshot`/`balance_as_of`), summed with assets
  positive and liabilities negative by account type — a read-side
  calculation, not a new ledger.

## Consequences

- Cheap to build: no schema change to `balance_snapshots` itself (it
  already supports this shape), just new `AccountType` values and one
  new manual-entry code path (CLI command + `Db` method) alongside the
  existing parser-driven one.
- Sets no ceiling on precision: a house or pension tracked this way is
  "a value on a date," which is exactly what's asked for today. It
  does *not* give postings-level detail (e.g. splitting a mortgage
  payment into interest vs. principal, or a pension contribution into
  employer/employee/tax-relief legs) — if that level of detail is
  wanted later, that is the point at which the double-entry pivot
  (Delta: Double-Entry Accounting) earns its cost, not before.
- Recorded as a live consideration on Delta: Double-Entry Accounting:
  this ADR's lightweight model is the deliberate interim step: assets
  and liabilities as accounts with balance snapshots, revisited only if
  daily use shows the lack of postings-level detail actually matters.
- No change needed to the spend/income ledger split (ADR 0005) or the
  double-entry compatibility mapping already recorded in the design
  doc — accounts remain accounts, whichever future is chosen.
