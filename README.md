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
  a dedicated graph database — see `src/db/schema.sql`.
- **Single crate**: domain model, database access, statement import,
  analysis, and the TUI all live in one `ledgr` crate/binary for now. See
  ADR `doc/adr/0003-single-crate-package-ledgr.md` — split back into a
  library + binary later if/when a web frontend needs to reuse the domain
  logic without the TUI.
- **TUI**: [`ratatui`](https://ratatui.rs) + [`crossterm`](https://docs.rs/crossterm).
- **Import**: statement parsers implement the `StatementParser` trait in
  `src/import`, so adding a new bank's CSV/OFX format is a matter of
  writing one new parser.

## Development

```sh
cargo build
cargo run
cargo test
```

## Roadmap

- [ ] Parse common UK bank CSV export formats
- [ ] Parse pension/investment statement formats (PDF? OFX?)
- [ ] Transaction categorization (rule-based, then inference-assisted)
- [ ] Net worth / spending trend views in the TUI
- [ ] Web frontend for richer visualizations, extracting the domain logic
      back into its own crate
- [ ] Publish `ledgr` to crates.io

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
