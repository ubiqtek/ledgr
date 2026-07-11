# ledgr

A terminal UI app for personal finance. `ledgr` reads downloaded bank
statements, pension statements, and similar files, builds a local database,
and helps you analyze your money — categorization, trends, net worth over
time — without sending your data anywhere.

## Status

Early scaffold. Not yet functional.

## Design

- **Storage**: SQLite (via [`rusqlite`](https://docs.rs/rusqlite), bundled —
  no external `sqlite3` dependency needed). Relationships that don't fit a
  strict tabular shape (transfers between accounts, category hierarchies,
  refund/reversal links) are modeled as edge tables rather than reaching for
  a dedicated graph database — see `ledgr-core/src/db/schema.sql`.
- **Workspace layout**: a `ledgr-core` library crate holds the domain model,
  database access, statement import, and analysis logic. `ledgr-tui` is a
  thin binary crate on top of it. This split exists so a future web frontend
  can reuse `ledgr-core` without dragging in the TUI.
- **TUI**: [`ratatui`](https://ratatui.rs) + [`crossterm`](https://docs.rs/crossterm).
- **Import**: statement parsers implement the `StatementParser` trait in
  `ledgr-core::import`, so adding a new bank's CSV/OFX format is a matter of
  writing one new parser.

## Crates

- `ledgr-core` — domain model, SQLite schema/migrations, statement import,
  analysis.
- `ledgr-tui` — the terminal application.

## Development

```sh
cargo build
cargo run -p ledgr-tui
cargo test
```

## Roadmap

- [ ] Parse common UK bank CSV export formats
- [ ] Parse pension/investment statement formats (PDF? OFX?)
- [ ] Transaction categorization (rule-based, then inference-assisted)
- [ ] Net worth / spending trend views in the TUI
- [ ] Web frontend for richer visualizations, sharing `ledgr-core`
- [ ] Publish `ledgr-core`/`ledgr-tui` to crates.io

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
