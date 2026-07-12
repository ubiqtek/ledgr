# 3. Single crate, package `ledgr`

Date: 2026-07-11

## Status

Accepted

## Context

The project started as a two-crate Cargo workspace: `ledgr-core` (domain
model, SQLite schema, statement import, analysis) and `ledgr-tui` (a thin
`ratatui`/`crossterm` binary depending on it), with `ledgr-core` intended
to be reused by a future web frontend without pulling in the TUI.

We want `cargo install ledgr` to work via crates.io. crates.io refuses to
publish a crate whose dependency graph contains a path-only dependency —
every dependency must resolve to a published version on the registry.
Since `ledgr-tui` depended on `ledgr-core` by workspace path, publishing
`ledgr-tui` alone would have required also publishing `ledgr-core`
separately and keeping the two in version lockstep indefinitely, purely
to satisfy the registry's reproducibility requirement — not because the
project currently needs two independently reusable crates. No web
frontend exists yet to justify that cost today.

## Decision

Merge `ledgr-core` and `ledgr-tui` into a single crate, package name
`ledgr`, producing a single binary also named `ledgr`
(`cargo install ledgr` installs a binary called `ledgr`). The former
`ledgr-core` modules (`db`, `import`, `model`, `analysis`) and former
`ledgr-tui` modules (`app`, `ui`, `main`) now live side by side under one
`src/` tree, with the same internal module boundaries preserved.

## Consequences

- One crate to publish and version; `cargo install ledgr` works directly
  from crates.io with no dependency-ordering concerns.
- The module boundary between domain logic and TUI is preserved in
  structure (`db`/`import`/`model`/`analysis` vs `app`/`ui`), so splitting
  back into a library + binary workspace later — if a web frontend needs
  to reuse the domain logic without the TUI — should mostly be a
  mechanical extraction rather than a redesign.
- Items that were previously part of a library's public API (e.g.
  `ImportFileParser`, `NewAccount`) may trigger `dead_code` warnings until
  something outside tests uses them, since they're no longer exported
  from a separate lib crate. This is expected to resolve naturally as
  Delta 1 (bank statement import) starts exercising them.
