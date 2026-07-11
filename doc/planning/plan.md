# ledgr — Plan

## What's Next

The bank statement processing ADR has landed:
`doc/adr/0002-use-ofx-for-barclays-statement-import.md` — Barclays via
OFX (`ofx-rs` crate, mapping `FITID` to `Transaction::external_id`),
`GenericCsvParser` kept as fallback for institutions without OFX. Start
Delta 1, Task 1: implement the Barclays `StatementParser` per that ADR.

Rebel Finance research for Delta 4 is done — see
`doc/kb/rebel-finance/research.md`. Once the user has the actual
spreadsheet, confirm the 8 open questions listed there (exact default
category enum, transfer-flagging mechanics, income categorisation, etc.)
before finalising the Delta 4 taxonomy/schema.

The project was merged from a two-crate workspace (`ledgr-core` +
`ledgr-tui`) into a single crate/binary, package name `ledgr` — see
`doc/adr/0003-single-crate-package-ledgr.md`. All source now lives under
one `src/` tree (`db`, `import`, `model`, `analysis`, `app`, `ui`,
`main`); no more `-p <crate>` flags needed for `cargo run`/`cargo test`.

## Summary

| Delta | Status |
|---|---|
| [Bank Statement Import](#delta-bank-statement-import) | Not started (ADR landed, ready to implement) |
| [Credit Card Statement Import](#delta-credit-card-statement-import) | Not started |
| [Amazon Order Import](#delta-amazon-order-import) | Not started |
| [Spending Categorisation](#delta-spending-categorisation) | Not started (research in progress) |
| [Other Statement Import](#delta-other-statement-import) | Not started |
| [TUI Analysis Views](#delta-tui-analysis-views) | Not started |
| [Packaging & Distribution](#delta-packaging--distribution) | Not started |

Real-world goal driving the first four deltas: analyse monthly spending
across current account, credit card, and Amazon orders.

## Delta: Bank Statement Import

Parse a real download from the user's bank into the `ledgr` domain model
via the `StatementParser` trait. Format choice decided in
`doc/adr/0002-use-ofx-for-barclays-statement-import.md`.

### Task 1: Barclays OFX parser

TODO — implement `StatementParser` using the `ofx-rs` crate, mapping
`FITID` to `Transaction::external_id`. Needs a real sample OFX export
from Barclays online banking to confirm `ofx-rs` parses it cleanly
(it's newer/less battle-tested than the GPL-licensed alternative
`ofxy`, which was rejected on licensing grounds).

### Task 2: Import de-duplication

For Barclays, rely on `FITID` (via `external_id`) — no fragile content
hash needed. Other institutions imported via `GenericCsvParser` still
need a de-dup strategy since plain CSV has no equivalent stable ID.

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

Build out the TUI beyond the current scaffold.

### Task 1: Transaction list view

TODO — browsable, filterable transaction list in `ui.rs`/`app.rs`.

### Task 2: Net worth / spending trend views

TODO — charts or summary tables driven by `analysis.rs`.

## Delta: Packaging & Distribution

### Task 1: Publish `ledgr` to crates.io

Claiming the name now (see `publish-crate` skill for the
ubiq-architecture publishing conventions, and
`~/projects/dotfiles/doc/claiming-a-crate-name.md` for the general
steps). Since the merge into a single crate
(`doc/adr/0003-single-crate-package-ledgr.md`), this is one publish, not
two — no more dependency-ordering concern between `ledgr-core` and
`ledgr-tui`.
- Verify name landed as expected: `cargo search ledgr` or check
  https://crates.io/crates/ledgr directly (`cargo search` has been
  unreliable from this sandbox — API blocked, empty results).
- Confirm GitHub repo `https://github.com/ubiqtek/ledgr` was public at
  publish time.

### Task 2: Web frontend

TODO — longer-term. Would need extracting the domain logic (`db`,
`import`, `model`, `analysis`) back out into its own crate, per
`doc/adr/0003-single-crate-package-ledgr.md`.

## Implementation Notes

- Single crate `ledgr` (binary also named `ledgr`) — domain model,
  SQLite schema/migrations, statement import, and analysis sit alongside
  the TUI as modules under `src/` (`db`, `import`, `model`, `analysis`,
  `app`, `ui`, `main`). Previously a two-crate workspace; merged per
  `doc/adr/0003-single-crate-package-ledgr.md` so `cargo install ledgr`
  works via crates.io without a second published crate.
- Storage: SQLite via bundled `rusqlite`. Non-tabular relationships
  (transfers, category hierarchies, refund/reversal links) are modelled
  as edge tables in `src/db/schema.sql` rather than a graph database.
- New statement formats are added by implementing `StatementParser`
  in `src/import` (see `generic_csv.rs` for the existing example).
- Project is an early scaffold — not yet functional end-to-end.
