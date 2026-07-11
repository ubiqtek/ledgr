# ledgr — Plan

## What's Next

Start Delta 1, Task 1: implement a UK bank CSV statement parser under
`ledgr-core::import`, following the existing `StatementParser` trait
and `generic_csv.rs` as a reference.

## Summary

| Delta | Status |
|---|---|
| [Statement Import](#delta-statement-import) | Not started |
| [Categorisation](#delta-categorisation) | Not started |
| [TUI Analysis Views](#delta-tui-analysis-views) | Not started |
| [Packaging & Distribution](#delta-packaging--distribution) | Not started |

## Delta: Statement Import

Parse real-world bank and pension/investment statement formats into the
`ledgr-core` domain model via the `StatementParser` trait.

### Task 1: UK bank CSV parsers

TODO — pick 2-3 common UK bank CSV export formats (e.g. Monzo, Starling,
a high-street bank) and implement `StatementParser` for each.

### Task 2: Pension/investment statement parser

TODO — decide format (PDF vs OFX) and implement a parser.

### Task 3: Import de-duplication

TODO — ensure re-importing an overlapping statement doesn't duplicate
transactions (edge tables for transfer/refund links already exist in
`schema.sql`; de-dup logic should respect these).

## Delta: Categorisation

Assign categories to transactions, first via rules, later via inference.

### Task 1: Rule-based categorisation

TODO — design a simple rule format (merchant match, amount range, account)
and apply it during import or as a post-processing pass.

### Task 2: Inference-assisted categorisation

TODO — explore once rule-based categorisation has enough real data to
evaluate against.

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
