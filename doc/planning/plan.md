# ledgr — Plan

## What's Next

Waiting on an ADR from another session covering tech choices/design for
bank statement processing. Once it's pushed, pull it, read it, and start
Delta 1, Task 1: implement a real bank CSV statement parser under
`ledgr-core::import`, following the existing `StatementParser` trait
and `generic_csv.rs` as a reference.

Rebel Finance research for Delta 4 is done — see
`doc/kb/rebel-finance/research.md`. Once the user has the actual
spreadsheet, confirm the 8 open questions listed there (exact default
category enum, transfer-flagging mechanics, income categorisation, etc.)
before finalising the Delta 4 taxonomy/schema.

## Summary

| Delta | Status |
|---|---|
| [Bank Statement Import](#delta-bank-statement-import) | Not started (blocked on incoming ADR) |
| [Credit Card Statement Import](#delta-credit-card-statement-import) | Not started |
| [Amazon Order Import](#delta-amazon-order-import) | Not started |
| [Spending Categorisation](#delta-spending-categorisation) | Not started (research in progress) |
| [Other Statement Import](#delta-other-statement-import) | Not started |
| [TUI Analysis Views](#delta-tui-analysis-views) | Not started |
| [Packaging & Distribution](#delta-packaging--distribution) | Not started |

Real-world goal driving the first four deltas: analyse monthly spending
across current account, credit card, and Amazon orders.

## Delta: Bank Statement Import

Parse a real download from the user's bank into the `ledgr-core` domain
model via the `StatementParser` trait. Design informed by an ADR another
session is writing (tech choices for statement processing) — pull and
read it before starting.

### Task 1: Bank CSV/OFX parser

TODO — implement `StatementParser` for the user's actual bank export
format (confirm format once a sample statement is available).

### Task 2: Import de-duplication

TODO — ensure re-importing an overlapping statement doesn't duplicate
transactions (edge tables for transfer/refund links already exist in
`schema.sql`; de-dup logic should respect these).

## Delta: Credit Card Statement Import

TODO — parser for the user's credit card statement export, needed
alongside the current account to see full monthly spending.

## Delta: Amazon Order Import

TODO — import Amazon order history (format TBD — Amazon "Request my
data" export vs order history CSV) so Amazon purchases show up as
proper line items rather than one lump "Amazon" transaction per card
charge.

## Delta: Spending Categorisation

Categorise transactions using the "Rebel Finance" method. Background
research written to `doc/kb/rebel-finance/research.md`; the user can also
obtain their actual spreadsheet to confirm/replace details we couldn't
verify publicly.

### Task 1: Confirm Rebel Finance taxonomy

TODO — cross-check `doc/kb/rebel-finance/research.md` against the user's
own spreadsheet once available; finalise the category list and rules.

### Task 2: Rule-based categorisation engine

TODO — design a rule format (merchant match, amount range, account) that
implements the confirmed taxonomy, applied during import or as a
post-processing pass.

### Task 3: Inference-assisted categorisation

TODO — explore once rule-based categorisation has enough real data to
evaluate against.

## Delta: Other Statement Import

Lower-priority formats, deferred behind the four deltas above.

### Task 1: Pension/investment statement parser

TODO — decide format (PDF vs OFX) and implement a parser.

## Delta: TUI Analysis Views

Build out `ledgr-tui` beyond the current scaffold.

### Task 1: Transaction list view

TODO — browsable, filterable transaction list in `ui.rs`/`app.rs`.

### Task 2: Net worth / spending trend views

TODO — charts or summary tables driven by `ledgr-core::analysis`.

## Delta: Packaging & Distribution

### Task 1: Publish `ledgr-core` to crates.io

TODO — see the `publish-crate` skill for the ubiq-architecture
publishing conventions.

### Task 2: Publish `ledgr-tui` to crates.io

TODO.

### Task 3: Web frontend

TODO — longer-term; reuses `ledgr-core` per the design split in
`README.md`.

## Implementation Notes

- Workspace: `ledgr-core` (domain model, SQLite schema/migrations,
  import, analysis) + `ledgr-tui` (thin binary on top, ratatui +
  crossterm).
- Storage: SQLite via bundled `rusqlite`. Non-tabular relationships
  (transfers, category hierarchies, refund/reversal links) are modelled
  as edge tables in `ledgr-core/src/db/schema.sql` rather than a graph
  database.
- New statement formats are added by implementing `StatementParser`
  in `ledgr-core::import` (see `generic_csv.rs` for the existing example).
- Project is an early scaffold — not yet functional end-to-end.
