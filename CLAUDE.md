# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`ledgr` is a terminal UI app for personal finance: it imports downloaded
bank/pension statements, builds a local SQLite database, and helps analyse
money (categorisation, trends, net worth over time) with no data leaving the
machine. Early scaffold — not yet functional end-to-end.

## Commands

```sh
cargo build
cargo run
cargo test                        # whole crate
cargo test parses_generic_csv     # single test by name
```

## Architecture

Single crate/binary (`ledgr`) for now — domain model, SQLite schema,
statement import, and analysis all live alongside the TUI as modules
under `src/`. This used to be a two-crate workspace (`ledgr-core` +
`ledgr-tui`); it was merged so `cargo install ledgr` works via crates.io
without needing a second published crate. See
`doc/adr/0003-single-crate-package-ledgr.md`. Split it back into a
library + binary if/when a web frontend needs to reuse the domain logic
(`db`, `import`, `model`, `analysis`) without the TUI (`app.rs`, `ui.rs`).

### Storage

SQLite via `rusqlite` (bundled feature — no system `sqlite3` needed).
`Db::open`/`Db::open_in_memory` apply `src/db/schema.sql` on every open
using `CREATE TABLE/INDEX IF NOT EXISTS`, so schema application must stay
idempotent (see `db::tests::schema_is_idempotent`).

Relationships that don't fit a strict tabular shape — transfers between
accounts, category hierarchies, refund/reversal links — are modelled as
edge tables (`TransactionLink` / `LinkRelation` in `model.rs`) rather than
reaching for a graph database. See ADR `doc/adr/` for the reasoning trail
on decisions like this.

Amounts are always signed integers in minor currency units (e.g. pence),
never floats, to avoid drift — see `amount_minor` on `Transaction` and
`import::generic_csv::parse_amount_minor`.

### Statement import

Every supported bank/pension export format implements the
`StatementParser` trait (`src/import/mod.rs`):

```rust
trait StatementParser {
    fn name(&self) -> &'static str;
    fn parse(&self, path: &Path, account_id: Id) -> Result<Vec<NewTransaction>, ImportError>;
}
```

Parsers must not touch the database themselves, so they stay trivially
unit-testable in isolation — persistence happens afterwards via
`Db`. Adding a new institution's format means writing one new parser
module under `import/`, not touching the database or TUI layers.
`generic_csv.rs` is the reference implementation (`date,description,amount`
CSV).

### TUI

`app::Screen` is a simple state machine (`Accounts` → `Transactions`).
`main.rs` owns the terminal setup/teardown and the event loop; `app.rs`
mutates `App` state in response to key events; `ui.rs` renders from
`App` state only. Keep new screens following this same split rather than
mixing rendering and state mutation.

## Architecture decisions

Significant architectural decisions are recorded as ADRs in `doc/adr/`
(index at `doc/adr/decisions.md`). Check there before revisiting a past
decision, and add a new ADR when making one that future contributors would
otherwise have to reverse-engineer from the code.

## Conventions

- British English in code, comments, and docs (e.g. "categorise",
  "colour", "initialise").
