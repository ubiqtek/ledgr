# 2. Use OFX as the primary Barclays statement import format

Date: 2026-07-11

## Status

Accepted

## Context

`ledgr` needs to import Barclays statements as the first real bank format
(see `doc/planning/plan.md`, Delta: Statement Import). Barclays' Online
Banking (desktop only, not the mobile app) offers a "Download transactions"
feature supporting three formats: CSV, OFX, and QIF, typically limited to
the last 60-90 days (older data is only available as PDF statements).

Comparing the three:

- **CSV** has no standard schema — Barclays' own column layout, not
  self-describing — and no stable per-transaction identifier. Re-importing
  an overlapping date range (likely, given the 60-90 day window) risks
  duplicate transactions unless de-duplication falls back to hashing
  description/date/amount, which is fragile.
- **QIF** is a legacy line-based format with the same lack of a stable
  transaction ID as CSV, plus ambiguous field semantics that vary by
  exporter. No advantage over CSV for our purposes.
- **OFX** (Open Financial Exchange) is self-describing (SGML in v1.x, true
  XML in v2.x) and includes `FITID`, a bank-assigned stable transaction ID
  designed for exactly this de-duplication problem. It maps directly onto
  the `external_id` column already present on `transactions`
  (`ledgr-core/src/db/schema.sql`).

The Rust crate landscape for OFX has a naming trap: the crate literally
named `ofx` is unrelated (bindings for the OpenFX visual-effects plugin
standard, not Open Financial Exchange). Of the crates that actually parse
financial OFX:

- `ofxy` — OFX 1.x (SGML) only, and licensed GPL-3.0, which would conflict
  with `ledgr-core`'s planned MIT/Apache-2.0 dual license for crates.io
  publishing.
- `ofx-rs` (Govcraft) — supports both OFX 1.x (SGML) and 2.x (XML) through
  one entry point, exposes `FITID`/accounts/balances, uses
  `rust_decimal::Decimal` for amounts, and is dual-licensed MIT/Apache-2.0.
  Newer and lower-adoption (single-digit release count) than `ofxy`, so it
  needs validating against a real Barclays export before being trusted.

A live Open Banking API integration (Barclays' own Account & Transactions
API, or an aggregator like TrueLayer/Plaid/Tink) would remove the manual
download step entirely, but requires either regulated TPP registration or
a paid aggregator, plus OAuth consent renewal roughly every 90 days. That's
worthwhile eventually but too much upfront complexity for the first working
import path.

## Decision

Implement the Barclays `StatementParser` using OFX as the primary format,
via the `ofx-rs` crate, mapping `FITID` to `Transaction::external_id`.

Keep the existing `GenericCsvParser` as the fallback path for institutions
that don't offer OFX (expected to matter for pension/investment statements,
which are more likely to only offer PDF or a bespoke CSV).

Do not implement QIF support. Defer Open Banking API integration to a
later phase (see `doc/planning/plan.md`) once the file-based import path
is working end-to-end.

## Consequences

- `ledgr-core` gains a dependency on `ofx-rs`; its MIT/Apache-2.0 licensing
  keeps `ledgr-core`'s own dual license unencumbered, unlike `ofxy`.
- Before relying on `ofx-rs` in production, we need a real sample OFX
  export from Barclays online banking to confirm it parses cleanly —
  `ofx-rs` is newer and less battle-tested than `ofxy`, and real-world bank
  OFX files are often not fully spec-compliant.
- De-duplication on re-import can rely on `FITID` (via `external_id`)
  rather than a fragile content hash, for Barclays specifically. Other
  institutions imported via `GenericCsvParser` still need a de-dup strategy
  since plain CSV has no equivalent stable ID (tracked as Delta:
  Statement Import, Task 3 in `doc/planning/plan.md`).
