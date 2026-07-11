# ledgr — Plan

## What's Next

**Next:** Task 2 — Import de-duplication (Delta: Bank Statement Import) — decide + implement a de-dup strategy for `GenericCsvParser`-imported institutions, which have no stable per-transaction ID
**Sub-doc:** (none)
**Blockers:** None

## Summary

| Delta | Task | Status |
|-------|------|--------|
| [Delta: Bank Statement Import](#delta-bank-statement-import) | [1. Barclays OFX parser](#task-1-barclays-ofx-parser) | ✓ DONE |
| | [2. Import de-duplication](#task-2-import-de-duplication) | IN PROGRESS |
| | [3. Account resolution and balance tracking](#task-3-account-resolution-and-balance-tracking) | ✓ DONE |
| [Delta: Automatic Inbox Import](#delta-automatic-inbox-import) | [1. Inbox change notification](#task-1-inbox-change-notification) | TODO |
| [Delta: Credit Card Statement Import](#delta-credit-card-statement-import) | [1. Credit card statement parser](#task-1-credit-card-statement-parser) | TODO |
| [Delta: Amazon Order Import](#delta-amazon-order-import) | [1. Amazon order import](#task-1-amazon-order-import) | TODO |
| [Delta: Spending Categorisation](#delta-spending-categorisation) | [1. Confirm Rebel Finance taxonomy](#task-1-confirm-rebel-finance-taxonomy) | IN PROGRESS |
| | [2. Rule-based categorisation engine](#task-2-rule-based-categorisation-engine) | TODO |
| | [3. Inference-assisted categorisation](#task-3-inference-assisted-categorisation) | TODO |
| [Delta: Other Statement Import](#delta-other-statement-import) | [1. Pension/investment statement parser](#task-1-pensioninvestment-statement-parser) | TODO |
| [Delta: TUI Analysis Views](#delta-tui-analysis-views) | [1. Transaction list view](#task-1-transaction-list-view) | ✓ DONE |
| | [2. Net worth / spending trend views](#task-2-net-worth--spending-trend-views) | TODO |
| [Delta: Packaging & Distribution](#delta-packaging--distribution) | [1. Publish `ledgr` to crates.io](#task-1-publish-ledgr-to-cratesio) | ✓ DONE |
| | [2. Web frontend](#task-2-web-frontend) | TODO |
| [Delta: Live Open Banking (Enable Banking)](#delta-live-open-banking-enable-banking) | [1. Evaluate feasibility & security model](#task-1-evaluate-feasibility--security-model) | IN PROGRESS |

Real-world goal driving the first four deltas: analyse monthly spending
across current account, credit card, and Amazon orders.

## Delta: Bank Statement Import

Parse a real download from the user's bank into the `ledgr` domain model
via the `StatementParser` trait. Format choice decided in
`doc/adr/0002-use-ofx-for-barclays-statement-import.md`.

A config + inbox mechanism now backs this delta (and future ones): the
user points `ledgr` at a synced folder (their Google Drive) as a drop
location for downloaded statements.
- `~/.config/ledgr/config.toml` (`src/config.rs`) holds `inbox_dir`;
  auto-created on first run — hand-edit it to point at the Google Drive
  folder instead. XDG location per
  `doc/adr/0004-xdg-conventions-for-local-files.md`.
- `src/inbox.rs` — `Inbox` ensures `<inbox_dir>/processed/` exists, lists
  pending files (dotfiles like `.DS_Store` are ignored), and moves a
  file into `processed/` once handled.
- `src/import/pipeline.rs` — `import_inbox()` ties it together: picks a
  parser by extension (`.ofx`/`.qfx` → `BarclaysOfxParser`, `.csv` →
  `GenericCsvParser`), resolves the account via the parser's own
  `account_identity()` when the format carries one (falls back to a
  single default account otherwise), inserts transactions and any
  balance snapshot, moves the file to `processed/`.
- Wired up as headless CLI commands: `ledgr import` and `ledgr status`
  (checked in `main.rs` before the TUI starts) — no `clap`, just a
  manual `env::args()` check to keep things minimal.
- Account resolution: `StatementParser::account_identity(path)` lets a
  format identify which account a file belongs to (e.g. OFX
  `BANKACCTFROM`/`ACCTID`) so multiple accounts at the same institution
  don't collapse into one shared account — `BarclaysOfxParser`
  implements this. Formats with no such info (e.g. generic CSV) fall
  back to a single default account per institution.

### Task 1: Barclays OFX parser
- ✓ DONE — `BarclaysOfxParser` (`src/import/barclays_ofx.rs`) implemented
  using `ofx-rs`, maps `FITID` to `Transaction::external_id`, converts
  `OfxAmount` (`rust_decimal::Decimal`) to signed minor units via
  `rescale(2)` + `mantissa()`. Passes unit tests against a synthetic OFX
  fixture.
- ✓ DONE — validated against 3 real Barclays OFX exports (939
  transactions, 2026-01-02 to 2026-07-10, 0 parse failures, every
  transaction got a non-null `external_id` via FITID). `ofx-rs` (v0.2)
  handled the real files without needing parser fixes, despite being
  newer/less battle-tested than the GPL-licensed alternative `ofxy`
  (rejected on licensing grounds).

### Task 2: Import de-duplication
- ✓ DONE — whole-file de-dup: `src/import/pipeline.rs` hashes each inbox
  file (`sha2`) and skips it if `statements.file_hash` already has that
  hash (the schema already had the `UNIQUE` column; it just needed
  wiring up).
- ✓ DONE — per-transaction de-dup on `external_id`: added a partial
  unique index `idx_transactions_account_external_id` on
  `transactions(account_id, external_id) WHERE external_id IS NOT NULL`
  (`src/db/schema.sql`), so it only applies to formats that carry a
  stable external ID (e.g. Barclays OFX's `FITID`) and leaves
  `GenericCsvParser`-imported rows (`external_id` always `NULL`)
  unaffected. `Db::insert_transaction` (`src/db/transactions.rs`) now
  uses `INSERT OR IGNORE` and returns `Option<Id>` (`None` when the
  external_id already exists for that account) instead of `Id`.
  `import_inbox` (`src/import/pipeline.rs`) counts these as
  `transactions_deduplicated` on a new `ImportSummary` field, separate
  from `transactions_imported`. Covered by a new unit test in each of
  `db/transactions.rs` and `import/pipeline.rs` (the latter simulates a
  re-saved file — same FITIDs, different file_hash — asserting zero
  duplicate rows land in the database). Not yet validated against a
  real re-saved Barclays file; the user is going to test this by
  importing a new real file.
- ✓ DONE — validated per-transaction de-dup against real data: imported
  a new real account (Barclays Savings, `...3693`, "Adventure Fund", 28
  transactions) alongside a 7-day-overlap re-download of the existing
  "Jims Premier Account" (`...1892`) file. All 20 overlapping
  transactions in the overlap file were correctly caught as duplicates
  (account's transaction count stayed at exactly 562, confirmed one
  FITID directly in the database), while the new account's 28
  transactions imported cleanly.
- ✓ DONE — added a per-file import log: `import_inbox`
  (`src/import/pipeline.rs`) now writes a `.log` file alongside each
  processed statement in `processed/` (same timestamp-prefixed base
  name, `.log` extension via `Path::with_extension`), with one
  tab-separated line per transaction: `<external_id or "->\t<imported|
  duplicate|error>\t<message or "->`. Per-transaction insert errors
  (`Db::insert_transaction`) are now caught individually instead of
  propagated with `?`, so one bad row logs as `error` with the DB error
  message instead of aborting the rest of the file's import. Also
  fixed `Inbox::mark_processed` (`src/inbox.rs`) to prefix moved files
  with a `YYYYMMDDHHMMSS%3f-` millisecond timestamp, since banks reuse
  the same filename for every download (e.g. `data.ofx`), which would
  otherwise silently overwrite the previous copy in `processed/` — the
  timestamp also makes the `.log`'s companion file's processing time
  obvious at a glance. Two new unit tests added (log content/naming,
  including the duplicate-status case); verified end-to-end against a
  real re-saved Barclays file (all 20 duplicate transactions correctly
  logged as `duplicate`). Test count now 37, all passing.
- TODO — de-dup strategy for `GenericCsvParser`-imported institutions,
  which have no stable per-transaction ID.

### Task 3: Account resolution and balance tracking
- ✓ DONE — fixed a real bug found via `ledgr status`: all 3 real
  Barclays OFX files had been collapsing into one hardcoded "Barclays
  Current Account" instead of being recognised as 3 separate real
  accounts (`ACCTID`s ending `...5086`, `...1892`, `...2608`). Added
  `StatementParser::account_identity()` (`src/import/mod.rs`,
  `src/import/barclays_ofx.rs`) so each OFX file resolves to its own
  account via `BANKACCTFROM`. Real DB reset and re-imported: now shows
  3 correctly separated accounts (562/162/215 transactions).
- ✓ DONE — fixed a second bug: displayed balance was a naive
  `SUM(transactions)`, which didn't match reality because a statement's
  transaction window often doesn't reach back to account opening (2 of
  3 real accounts were off; confirmed the parsing itself was correct by
  manually summing raw `TRNAMT` values). Added a `balance_snapshots`
  table (`schema.sql`) storing bank-reported balance anchors (e.g. OFX
  `LEDGERBAL`), `src/db/balances.rs` (`insert_balance_snapshot`,
  `latest_balance_snapshot`, and `balance_as_of(account_id, date)` which
  reconstructs balance at any date from the nearest anchor plus
  transactions between it and the target — built generally to support
  future balance-history/trend views, not just "current balance"). New
  `StatementParser::balance_snapshot()`; `BarclaysOfxParser` reads OFX
  `LEDGERBAL`. Real balances now match each file's `LEDGERBAL` exactly
  (946.26 / 7.47 / 3106.58 GBP).
- ✓ DONE — added `ledgr status` CLI command (`src/main.rs`) printing
  per-account balance (with as-of date), transaction count, date range,
  and last-imported-at — this is what surfaced both bugs above.

## Delta: Automatic Inbox Import

Currently `ledgr import` must be run manually. Explore having new files
in the inbox trigger an import automatically instead of the user
remembering to run the command.

### Task 1: Inbox change notification
- TODO — evaluate launchd's `WatchPaths` key (a LaunchAgent plist that
  runs `ledgr import` whenever the inbox directory changes) versus
  embedding the `notify` crate (wraps macOS FSEvents) so `ledgr` itself
  watches while running. Leaning towards `WatchPaths`: native, no
  polling/cron, and doesn't require a long-running `ledgr` process.
  Avoid a cron-based polling loop if a native change-notification
  mechanism (FSEvents-backed) covers it.

## Delta: Credit Card Statement Import

### Task 1: Credit card statement parser
- TODO — parser for the user's credit card statement export, needed
  alongside the current account to see full monthly spending.

## Delta: Amazon Order Import

### Task 1: Amazon order import
- TODO — import Amazon order history (format TBD — Amazon "Request my
  data" export vs order history CSV) so Amazon purchases show up as
  proper line items rather than one lump "Amazon" transaction per card
  charge.

## Delta: Spending Categorisation

Categorise transactions using the "Rebel Finance" method.

### Task 1: Confirm Rebel Finance taxonomy
- ✓ DONE — background research written to `doc/kb/rebel-finance/research.md`.
- TODO — cross-check the research against the user's own spreadsheet once
  available; finalise the category list and rules; confirm the 8 open
  questions listed in the research doc (exact default category enum,
  transfer-flagging mechanics, income categorisation, etc.).

### Task 2: Rule-based categorisation engine
- TODO — design a rule format (merchant match, amount range, account)
  that implements the confirmed taxonomy, applied during import or as a
  post-processing pass.

### Task 3: Inference-assisted categorisation
- TODO — explore once rule-based categorisation has enough real data to
  evaluate against.

## Delta: Other Statement Import

Lower-priority formats, deferred behind the four deltas above.

### Task 1: Pension/investment statement parser
- TODO — decide format (PDF vs OFX) and implement a parser.

## Delta: TUI Analysis Views

Build out the TUI beyond the current scaffold.

### Task 1: Transaction list view
- ✓ DONE — built out the Transactions and Accounts screens in
  `ui.rs`/`app.rs` into real, browsable, scrollable views over the 939
  imported transactions: switched both from plain `List` rendering to
  ratatui `Table` widgets with fixed-width columns (date/amount/currency/
  description for transactions; name/type/institution/balance/last-imported
  for accounts) so columns align regardless of content length.
- ✓ DONE — fixed three real rendering bugs found by actually driving the
  TUI in tmux: (1) no auto-scroll — `ListState`/`TableState` were being
  recreated fresh every frame instead of persisted on `App`, so the scroll
  `offset` never carried over and ratatui recentred the viewport on every
  keypress instead of scrolling by the minimal amount; fixed by adding
  `accounts_table_state`/`transactions_table_state` fields to `App` that
  persist across frames. (2) highlighting only coloured the text span, not
  the full row — fixed via `.highlight_style()` on the whole `Table`/`List`
  instead of styling each item's `Span` individually. (3) a stray
  cursor/glyph artifact — fixed via `terminal.hide_cursor()` on startup
  plus a `Clear` widget rendered at the top of every frame (screen
  transitions between different widget layouts could otherwise leave a
  previous frame's characters showing through a shorter cell).
- ✓ DONE — root-caused what looked like a column-alignment bug as literal
  tab characters embedded in Barclays' own OFX transaction descriptions
  (e.g. `ESSO NEWQUAY\tON 09 JUL CPM`), which made the terminal jump to
  tab stops. Fixed going forward via a `clean_description()` whitespace
  collapse in `barclays_ofx.rs`, and cleaned the 939 already-imported rows
  in the real local database directly (one-off `UPDATE`, not a code path).
- ✓ DONE — added keyboard navigation matching nvim conventions: `gg`
  (jump to first row) and `G` (jump to last row), plus `Ctrl-d`/`Ctrl-u`
  for full-page down/up (sized to the actual visible list height each
  frame via `terminal.size()`), and a `?` help screen listing all
  keybindings (`Screen::Help` in `app.rs`, toggles back to whichever
  screen was open before).
- ✓ DONE — renamed `AccountType::Checking` to `AccountType::Current`
  throughout the model, schema `CHECK` constraint, and all call sites,
  since the correct UK banking term is "current account" not "checking
  account" (British English, per this project's own convention). Migrated
  the schema change into the real local database by hand (SQLite doesn't
  support altering a `CHECK` constraint in place, so this required
  recreating the `accounts` table).
- ✓ DONE — added a `ledgr name-account <last-4-digits> "<name>"` CLI
  command (`src/main.rs`) backed by a new `account_names` map in
  `src/config.rs`'s `Config` (keyed by the last 4 digits embedded in the
  bank-generated account name, e.g. `(...5678)`), so the user can give
  their own display names to accounts instead of showing Barclays' own
  naming. Deliberately stored in the config file rather than the database,
  so renaming an account can never break `find_or_create_account`'s
  institution/name matching used to avoid duplicating accounts on
  re-import. Applied via `Config::apply_account_name_overrides()` wherever
  accounts are displayed (TUI accounts list, transactions screen title,
  `ledgr status`).

### Task 2: Net worth / spending trend views
- TODO — charts or summary tables driven by `analysis.rs`. Groundwork
  exists: `Db::balance_as_of(account_id, date)` (`src/db/balances.rs`)
  can already reconstruct balance at an arbitrary date from balance
  snapshots + transactions.

## Delta: Packaging & Distribution

### Task 1: Publish `ledgr` to crates.io
- ✓ DONE — published as v0.1.0: https://crates.io/crates/ledgr. Name
  claimed, `cargo install ledgr` works. Future releases: bump version in
  `Cargo.toml` and re-run `cargo publish`.

### Task 2: Web frontend
- TODO — longer-term. Would need extracting the domain logic (`db`,
  `import`, `model`, `analysis`) back out into its own crate, per
  `doc/adr/0003-single-crate-package-ledgr.md`.

## Delta: Live Open Banking (Enable Banking)

Explore OAuth-based live account access via [Enable Banking](https://enablebanking.com)
as an alternative/supplement to manual OFX export download. Purely
exploratory / doc-only so far — not blocking Bank Statement Import
(still waiting on a real Barclays OFX export). Lower priority than the
other deltas since it's a bigger architectural fork (dependency on a
hosted aggregator vs ledgr's "nothing leaves the machine" local-file
model) than adding a new `StatementParser`.

### Task 1: Evaluate feasibility & security model
- ✓ DONE — research written to `doc/kb/enable-banking-registration.md`,
  grounded against Enable Banking's real docs (Quick Start, Control
  Panel, API reference, FAQ) after discovering the first draft
  (Gemini-generated) contained inaccuracies: the "bypass KYB" framing
  and a fixed 90-day consent claim were both wrong/misleading.
  Corrected to the real "Restricted Mode (Account Linking)" tier (no
  KYB needed for linking your own accounts) and per-ASPSP
  `valid_until` consent validity (typically up to 180 days, not a
  fixed 90).
- TODO — confirm Barclays' actual `maximum_consent_validity` via
  `/aspsps`.
- TODO — decide where to store the long-lived private key/session
  locally, consistent with ledgr's local-only data model; likely
  warrants its own ADR given the architectural fork.
- TODO — confirm Restricted Mode account-linking caps and Enable
  Banking's pricing before relying on it.

## Checkpoint: Session 2026-07-11

**What was completed this session:**
- Built a config + inbox mechanism (new, not previously in the plan):
  `src/config.rs` (`Config.inbox_dir`, TOML at
  `~/Library/Application Support/dev.ledgr.ledgr/config.toml`,
  auto-created on first run) and `src/inbox.rs` (`Inbox` ensures
  `<inbox_dir>/processed/` exists, lists pending files, moves processed
  files).
- Implemented `BarclaysOfxParser` (`src/import/barclays_ofx.rs`) using
  `ofx-rs`, mapping `FITID` → `external_id`, converting amounts via
  `rust_decimal`.
- Implemented `src/import/pipeline.rs::import_inbox()` tying
  config/inbox/parser/db together: SHA-256 whole-file de-dup against
  `statements.file_hash`, parser selection by extension, account
  resolution via new `Db::find_or_create_account`, moves processed files.
- Wired up as a headless `ledgr import` CLI command in `main.rs` (no
  `clap`, manual `env::args()` check).
- Added `Db::insert_statement` / `Db::find_statement_by_hash`
  (`src/db/statements.rs`) and `Db::find_or_create_account`
  (`src/db/accounts.rs`).
- Ran a full round-trip smoke test with an isolated `HOME`: synthetic
  OFX file → imported → moved to `processed/` → re-import correctly
  skipped as a duplicate. All 18 unit tests pass.
- Kicked off background research (separate from the ledgr codebase) into
  "open banking as an API/MCP bridge" as a possible startup idea —
  report written to a scratchpad file, not part of this repo.

**State of the project:**
Barclays OFX import is code-complete and unit-tested but not yet
validated against a real Barclays file — that's the immediate blocker,
waiting on the user's download. The inbox/config/pipeline
infrastructure is generic enough to support the next statement formats
(credit card, pension) once their parsers exist. The TUI still only has
the original Accounts → Transactions scaffold; import is CLI-only for
now, not wired into the TUI.

**Immediate next priorities:**
1. Validate `BarclaysOfxParser` against a real Barclays OFX export once
   downloaded; fix any real-world parsing gaps.
2. Once validated, run a real `ledgr import` against the user's actual
   inbox/Google Drive folder and confirm transactions look correct.
3. Decide whether/how to surface import status in the TUI itself
   (currently CLI-only).
4. Move on to Credit Card Statement Import once Bank Statement Import is
   fully validated.

## Checkpoint: Session 2026-07-11b

**What was completed this session:**
- Reviewed a Gemini-generated doc about using Enable Banking for live
  Open Banking API access to Barclays, found it contained inaccuracies
  (misleading "bypass KYB" framing, wrong fixed 90-day consent claim).
- Verified against Enable Banking's real documentation (Quick Start,
  Control Panel, API reference, FAQ) and rewrote
  `doc/kb/enable-banking-registration.md` with corrected, sourced
  information: the real "Restricted Mode (Account Linking)" tier
  (personal accounts, no KYB required), correct JWT structure (exp up
  to 24h not 1h), the real AIS flow (`/aspsps` → `/auth` → `/sessions`
  → `/accounts/{id}/transactions`), and correct consent lifetime
  (per-ASPSP `valid_until`, typically up to 180 days).
- Added a new "Live Open Banking (Enable Banking)" Delta to the plan to
  track this as an explicit, lower-priority exploratory thread
  alongside the main OFX import work.

**State of the project:**
No code changes this session — this was documentation/research only.
Bank Statement Import (Barclays OFX) remains the active priority,
still blocked on the user downloading a real OFX export. The Enable
Banking exploration is parked as a documented option for live account
access, not yet decided whether to pursue.

**Immediate next priorities:**
1. Validate `BarclaysOfxParser` against a real Barclays OFX export once
   downloaded; fix any real-world parsing gaps.
2. Once validated, run a real `ledgr import` against the user's actual
   inbox/Google Drive folder and confirm transactions look correct.
3. Decide whether/how to surface import status in the TUI itself
   (currently CLI-only).
4. Move on to Credit Card Statement Import once Bank Statement Import is
   fully validated.
5. Separately, if pursuing live Open Banking: confirm Barclays'
   `maximum_consent_validity`, decide on local key/session storage, and
   write an ADR before writing any code against Enable Banking.

## Checkpoint: Session 2026-07-11c

**What was completed this session:**
- Moved `ledgr`'s config file to an XDG-style path,
  `~/.config/ledgr/config.toml` (was
  `~/Library/Application Support/dev.ledgr.ledgr/config.toml`), so it
  can be symlinked into a dotfiles repo. Recorded as
  `doc/adr/0004-xdg-config-location.md`. `src/config.rs` now uses
  `directories::BaseDirs::home_dir()` instead of `ProjectDirs`. The
  SQLite data dir (`ledgr.db`) is unaffected — only the config path
  moved.
- Set the real `inbox_dir` in `~/.config/ledgr/config.toml` to the
  user's Google Drive folder
  (`.../GoogleDrive-jim.barritt@gmail.com/My Drive/_ledgr_inbox`).
- Ran `ledgr import` against 3 real Barclays OFX exports the user
  dropped into that folder. Result: 939 transactions imported across
  2026-01-02 to 2026-07-10, 0 files skipped, every transaction has a
  non-null `external_id` (FITID). Spot-checked descriptions and
  pence-denominated amounts look correct. `BarclaysOfxParser` is now
  validated against real data — Task 1 of Bank Statement Import marked
  done.
- Also this session (documentation-only, before the above): reviewed
  and corrected `doc/kb/enable-banking-registration.md` (a
  Gemini-generated doc on using Enable Banking for live Open Banking
  API access) against Enable Banking's real docs, added a Security
  implications section and a note on the painful manual Barclays OFX
  download UX, and added a new "Live Open Banking (Enable Banking)"
  Delta to the plan to track it as a lower-priority exploratory thread.

**State of the project:**
Bank Statement Import is now functionally proven end-to-end against
real data: config → inbox → parser → de-duped import → SQLite, using
the user's actual Barclays exports. 939 real transactions are now in
the local database. Only per-transaction/generic-CSV de-dup refinement
remains before this delta is fully done. The TUI still hasn't caught up
— it only shows the original Accounts → Transactions scaffold and
doesn't yet browse the newly-imported real data.

**Immediate next priorities:**
1. Implement per-transaction de-dup on Barclays `FITID` (via
   `external_id`) so re-importing a re-saved/renamed file with
   overlapping dates doesn't duplicate transactions.
2. Decide + implement a de-dup strategy for `GenericCsvParser`-imported
   institutions, which have no stable per-transaction ID.
3. Build out the TUI transaction list view so the 939 imported
   transactions are actually browsable, not just sitting in SQLite.
4. Move on to Credit Card Statement Import once Bank Statement Import
   (Task 2) is fully done.

## Checkpoint: Session 2026-07-11d

**What was completed this session:**
- Added `ledgr status` CLI command (`src/main.rs`) — this surfaced two
  real bugs in the "validated" Barclays OFX import from the previous
  checkpoint.
- Bug 1: all 3 real OFX files had been collapsing into one hardcoded
  "Barclays Current Account" instead of 3 separate real accounts.
  Fixed by adding `StatementParser::account_identity()`
  (`src/import/mod.rs`), implemented in `BarclaysOfxParser`
  (`src/import/barclays_ofx.rs`) by reading OFX `BANKACCTFROM`/`ACCTID`.
  `pipeline.rs` now resolves the account per file instead of always
  using the hardcoded one. `CLAUDE.md`'s documented trait snippet
  updated to match. Real DB reset and re-imported: 3 correctly
  separated accounts (562/162/215 transactions).
- Bug 2: displayed balance was `SUM(transactions)`, which didn't match
  reality for 2 of 3 accounts because a statement's transaction window
  doesn't necessarily reach back to account opening (confirmed via
  manual cross-check that the parsing itself was correct — only the
  window was incomplete). Fixed by adding a `balance_snapshots` table
  (`schema.sql`) and `src/db/balances.rs`
  (`insert_balance_snapshot`/`latest_balance_snapshot`/`balance_as_of`)
  to treat bank-reported balances (OFX `LEDGERBAL`) as anchors,
  reconstructing balance at any date from the nearest anchor plus
  transactions — built generally to support future balance-history/
  trend views. New `StatementParser::balance_snapshot()`;
  `BarclaysOfxParser` reads OFX `LEDGERBAL`. Real balances now match
  each file's `LEDGERBAL` exactly.
- Small fix: `Inbox::pending_files()` (`src/inbox.rs`) now ignores
  dotfiles (e.g. `.DS_Store`, which Google Drive/Finder leaves in
  synced folders) instead of counting/reporting them as skipped.
- Consolidated config+data location into one broadened ADR,
  `doc/adr/0004-xdg-conventions-for-local-files.md` (renamed from the
  narrower `0004-xdg-config-location.md`). Database now also at
  `~/.local/share/ledgr/ledgr.db` (was
  `~/Library/Application Support/dev.ledgr.ledgr/ledgr.db`),
  `data_dir_db_path()` in `src/main.rs` updated. Both config and DB
  migrated by hand for the real local install.
- Added a "Destructive commands" section to the user's global
  `CLAUDE.md` (outside this repo): since `rm` is blocked in this
  environment, copy `rm` commands to the clipboard via `pbcopy` instead
  of working around the block.
- Test count grew from 18 to 32, all passing.

**State of the project:**
Bank Statement Import is now genuinely correct against real data, not
just "imports without crashing" — account separation and balances both
verified against the source OFX files' own reported values (`ACCTID`,
`LEDGERBAL`). The balance-anchor infrastructure (`balance_as_of`) is
generic enough to directly support the still-TODO net worth/trend view
work later. Only per-transaction/generic-CSV de-dup refinement remains
before Bank Statement Import is fully done. The TUI still hasn't caught
up — no browsing of the real transaction/balance data yet.

**Immediate next priorities:**
1. Implement per-transaction de-dup on Barclays `FITID` (via
   `external_id`) so re-importing a re-saved/renamed file with
   overlapping dates doesn't duplicate transactions.
2. Decide + implement a de-dup strategy for `GenericCsvParser`-imported
   institutions, which have no stable per-transaction ID.
3. Build out the TUI transaction list view so the real imported data is
   browsable, not just sitting in SQLite.
4. Consider wiring `ledgr status`/balance data into a TUI view — the
   `balance_as_of` groundwork is already there.
5. Move on to Credit Card Statement Import once Bank Statement Import
   (Task 2) is fully done.

## Checkpoint: Session 2026-07-11e

**What was completed this session:**
- Added a `ledgr name-account <last-4-digits> "<name>"` CLI command and a
  new `account_names` map in `Config` (`src/config.rs`) so the user can
  give accounts their own display names instead of Barclays' own naming,
  without risking `find_or_create_account`'s dedup matching (used the
  real accounts to set: 1892 → "Jims Premier Account", 2608 → "Online
  Spending", 5086 → "Bills Account").
- Renamed `AccountType::Checking` → `AccountType::Current` everywhere
  (model, schema `CHECK` constraint, all call sites) — UK banking calls
  these "current accounts", not "checking accounts". Migrated the real
  local database by hand (recreated the `accounts` table since SQLite
  can't alter a `CHECK` constraint in place).
- Rebuilt the Transactions and Accounts TUI screens from plain `List`s
  into ratatui `Table`s with fixed-width columns, and fixed three real
  bugs found by actually driving the TUI in tmux: no auto-scroll
  (`ListState`/`TableState` weren't persisted across frames — now
  `accounts_table_state`/`transactions_table_state` live on `App`),
  highlighting only covering the text span instead of the full row, and a
  stray cursor/glyph artifact (fixed via `terminal.hide_cursor()` plus a
  `Clear` widget each frame).
- Found and fixed the real root cause of what looked like a column
  misalignment bug: literal tab characters embedded in Barclays' OFX
  transaction descriptions. Added `clean_description()` to
  `barclays_ofx.rs` and cleaned the 939 already-imported rows in the real
  database directly.
- Added nvim-style navigation: `gg`/`G` (jump to top/bottom), `Ctrl-d`/
  `Ctrl-u` (full-page down/up, sized from the real terminal height), and
  a `?` help screen listing all keybindings.
- Added Balance and Last Imported columns to the Accounts screen (reusing
  `Db::account_statuses()`, the same data `ledgr status` uses), right-
  aligned so decimal points line up, with all columns fixed-width so
  leftover space trails right instead of stretching across the terminal.
- Renamed the `just run` recipe to `run-local` and gave it `*ARGS` so
  `just run-local status` works. Added a `ledgr()` shell function to
  `~/.zshrc_machine` (outside this repo) that runs `cargo run --quiet --`
  against this repo, so typing `ledgr <args>` anywhere always runs the
  latest code without a separate install/reinstall step.
- Accidentally ran `tmux kill-server` while debugging pane sizing during
  TUI testing, which killed all of the user's tmux sessions, not just the
  test one — flagged to the user at the time; no repo/data impact, but
  worth remembering not to reach for `kill-server` again (scope to
  `kill-session` on the specific test session instead).

**State of the project:**
The TUI now has a genuinely usable, browsable Accounts → Transactions
flow over the 939 real imported transactions: proper scrolling, full-row
highlighting, aligned columns, balance/last-imported visibility, nvim-
style navigation, and a help screen — Task 1 of TUI Analysis Views is
done. Account naming is now user-controlled via config rather than stuck
with the bank's own naming. `AccountType` terminology now matches UK
banking conventions throughout. Bank Statement Import itself is
unchanged this session — per-transaction/generic-CSV de-dup (Task 2)
remains the oldest open TODO in the plan.

**Immediate next priorities:**
1. Implement per-transaction de-dup on Barclays `FITID` (via
   `external_id`) so re-importing a re-saved/renamed file with
   overlapping dates doesn't duplicate transactions.
2. Decide + implement a de-dup strategy for `GenericCsvParser`-imported
   institutions, which have no stable per-transaction ID.
3. Net worth / spending trend views (Task 2 of TUI Analysis Views) — the
   `balance_as_of` groundwork already exists in `src/db/balances.rs`.
4. Move on to Credit Card Statement Import once Bank Statement Import
   (Task 2) is fully done.

## Checkpoint: Session 2026-07-11f

**What was completed this session:**
- Implemented per-transaction de-dup on `external_id` (e.g. Barclays
  `FITID`): a partial unique index `idx_transactions_account_external_id`
  on `transactions(account_id, external_id) WHERE external_id IS NOT NULL`
  (`src/db/schema.sql`), `Db::insert_transaction` changed to
  `INSERT OR IGNORE` returning `Option<Id>` (`src/db/transactions.rs`),
  and `import_inbox` (`src/import/pipeline.rs`) now tracks a new
  `transactions_deduplicated` count on `ImportSummary`, separate from
  `transactions_imported`. Two new unit tests added (one in each of
  `db/transactions.rs` and `import/pipeline.rs`); all 35 tests pass.
- Discussed adding a Delta for automatically triggering `ledgr import`
  when new files land in the inbox, instead of the user running the
  command manually. Leaning towards macOS launchd `WatchPaths` (native
  FSEvents-backed change notification via a LaunchAgent plist) over
  embedding the `notify` crate or a cron polling loop — added as a new
  "Automatic Inbox Import" Delta, not yet designed or implemented.

**State of the project:**
Bank Statement Import's de-dup story is now complete for formats that
carry a stable per-transaction ID (Barclays OFX via FITID) — both
whole-file (SHA-256 hash) and per-transaction (external_id) dedup are in
place and unit-tested, though the per-transaction path is not yet
validated against a real re-saved Barclays file (the user is going to
test this manually by importing a new real file). The one remaining gap
in Task 2 is a de-dup strategy for `GenericCsvParser`-imported
institutions, which have no stable per-transaction ID to key off. A new
"Automatic Inbox Import" Delta was scoped (not started) to remove the
manual `ledgr import` step.

**Immediate next priorities:**
1. Decide + implement a de-dup strategy for `GenericCsvParser`-imported
   institutions, which have no stable per-transaction ID.
2. Net worth / spending trend views (Task 2 of TUI Analysis Views) — the
   `balance_as_of` groundwork already exists in `src/db/balances.rs`.
3. Design the Automatic Inbox Import mechanism (launchd `WatchPaths` vs
   `notify` crate) once higher-priority import/TUI work settles.
4. Move on to Credit Card Statement Import once Bank Statement Import
   (Task 2) is fully done.

## Checkpoint: Session 2026-07-11g

**What was completed this session:**
- Validated per-transaction de-dup against real data: imported a new
  real Barclays Savings account (`...3693`, "Adventure Fund", 28
  transactions) alongside a 7-day-overlap re-download of the existing
  "Jims Premier Account" file. All 20 overlapping transactions were
  correctly caught as duplicates; the account's transaction count
  stayed at exactly 562 (confirmed by checking one FITID directly in
  the database).
- Added a per-file import log: `import_inbox` (`src/import/pipeline.rs`)
  now writes a `.log` file next to each processed statement (same
  timestamp-prefixed name, `.log` extension), one tab-separated line
  per transaction (external_id, status — imported/duplicate/error,
  message). Per-transaction insert errors are now caught individually
  instead of aborting the whole file's import.
- Fixed `Inbox::mark_processed` (`src/inbox.rs`) to prefix processed
  filenames with a millisecond timestamp, since banks reuse the same
  filename for every download and this was silently overwriting the
  previous copy in `processed/`.
- Two new unit tests added; verified end-to-end against a real
  re-saved Barclays file (20 transactions all correctly logged as
  `duplicate`). Test count now 37, all passing.
- Cleaned up a synthetic test statement/file created during real-data
  verification from both the real database and the real inbox
  `processed/` folder (no stray transactions were created — the DELETE
  only removed the harmless `statements` row).

**State of the project:**
Bank Statement Import's de-dup story (Task 2) is now fully validated
end-to-end against real data for formats with a stable external ID
(Barclays OFX via FITID) — whole-file and per-transaction dedup are
both proven, plus a new audit trail (per-file `.log`) showing exactly
what happened to every transaction in an import. The remaining gap is
a de-dup strategy for `GenericCsvParser`-imported institutions (no
stable per-transaction ID).

**Immediate next priorities:**
1. Decide + implement a de-dup strategy for `GenericCsvParser`-imported
   institutions, which have no stable per-transaction ID.
2. Net worth / spending trend views (Task 2 of TUI Analysis Views) — the
   `balance_as_of` groundwork already exists in `src/db/balances.rs`.
3. Design the Automatic Inbox Import mechanism (launchd `WatchPaths` vs
   `notify` crate) once higher-priority import/TUI work settles.
4. Move on to Credit Card Statement Import once Bank Statement Import
   (Task 2) is fully done.

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
