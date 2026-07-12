# ledgr — Plan

## What's Next

**Next:** Delta: The Gap, Task 1 — minimal income ledger (`income_entries` + `income_entry_sources`), now that Task 3's assets/liabilities recording model is decided (ADR 0007). Task 4 (implementing that model) can run alongside or after Task 1/2.
**Sub-doc:** (none)
**Blockers:** None currently blocking. One follow-up note (not a hard blocker): existing accounts' `sort_code`/`account_number` won't backfill until a genuinely new file is imported per account, so transfer detection has no real household data yet.

## Summary

| Delta | Task | Status |
|-------|------|--------|
| [Delta: Bank Transaction Import](#delta-bank-transaction-import) | [1. Barclays OFX parser](#task-1-barclays-ofx-parser) | ✓ DONE |
| | [2. Import de-duplication](#task-2-import-de-duplication) | IN PROGRESS |
| | [3. Account resolution and balance tracking](#task-3-account-resolution-and-balance-tracking) | ✓ DONE |
| [Delta: Automatic Inbox Import](#delta-automatic-inbox-import) | [1. Inbox change notification](#task-1-inbox-change-notification) | TODO |
| [Delta: Credit Card Transaction Import](#delta-credit-card-transaction-import) | [1. Credit card statement parser](#task-1-credit-card-statement-parser) | TODO |
| | [2. Evaluate Barclaycard PDF export](#task-2-evaluate-barclaycard-pdf-export) | TODO |
| [Delta: Amazon Order Import](#delta-amazon-order-import) | [1. Evaluate automation route — email scanning vs manual export](#task-1-evaluate-automation-route--email-scanning-vs-manual-export) | TODO |
| | [2. Amazon order import](#task-2-amazon-order-import) | TODO |
| [Delta: Spend Ledger](#delta-spend-ledger) | [1. Spend ledger design](#task-1-spend-ledger-design) | ✓ DONE |
| | [2. Spend ledger schema and derivation](#task-2-spend-ledger-schema-and-derivation) | ✓ DONE |
| | [3. Review and re-classification TUI](#task-3-review-and-re-classification-tui) | TODO — deprioritised below Delta: The Gap |
| [Delta: The Gap](#delta-the-gap) | [1. Minimal income ledger](#task-1-minimal-income-ledger) | TODO |
| | [2. Gap calculation](#task-2-gap-calculation) | TODO |
| | [3. Discovery about recording assets and liabilities](#task-3-discovery-about-recording-assets-and-liabilities) | ✓ DONE |
| | [4. Implement assets and liabilities as accounts](#task-4-implement-assets-and-liabilities-as-accounts) | TODO |
| [Delta: Spending Categorisation](#delta-spending-categorisation) | [1. Confirm Rebel Finance taxonomy](#task-1-confirm-rebel-finance-taxonomy) | IN PROGRESS |
| | [2. Rule-based categorisation engine](#task-2-rule-based-categorisation-engine) | TODO |
| | [3. Inference-assisted categorisation](#task-3-inference-assisted-categorisation) | TODO |
| [Delta: Other Transaction Import](#delta-other-transaction-import) | [1. Pension/investment statement parser](#task-1-pensioninvestment-statement-parser) | TODO |
| [Delta: TUI Analysis Views](#delta-tui-analysis-views) | [1. Transaction list view](#task-1-transaction-list-view) | ✓ DONE |
| | [2. Net worth / spending trend views](#task-2-net-worth--spending-trend-views) | TODO |
| [Delta: Packaging & Distribution](#delta-packaging--distribution) | [1. Publish `ledgr` to crates.io](#task-1-publish-ledgr-to-cratesio) | ✓ DONE |
| | [2. Web frontend](#task-2-web-frontend) | TODO |
| [Delta: Live Open Banking (Enable Banking)](#delta-live-open-banking-enable-banking) | [1. Evaluate feasibility & security model](#task-1-evaluate-feasibility--security-model) | IN PROGRESS |
| [Delta: Double-Entry Accounting](#delta-double-entry-accounting) | [1. Evaluate a double-entry model for ledgr](#task-1-evaluate-a-double-entry-model-for-ledgr) | TODO |
| [Delta: Statement/Import Naming Cleanup](#delta-statementimport-naming-cleanup) | [1. Agree the replacement term](#task-1-agree-the-replacement-term) | ✓ DONE |
| | [2. Refactor to the agreed term](#task-2-refactor-to-the-agreed-term) | ✓ DONE |

Real-world goal driving the first four deltas: analyse monthly spending
across current account, credit card, and Amazon orders.

## Delta: Bank Transaction Import

Parse a real download from the user's bank into the `ledgr` domain model
via the `ImportFileParser` trait. Format choice decided in
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
- Account resolution: `ImportFileParser::account_identity(path)` lets a
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
  `ImportFileParser::account_identity()` (`src/import/mod.rs`,
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
  `ImportFileParser::balance_snapshot()`; `BarclaysOfxParser` reads OFX
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

## Delta: Credit Card Transaction Import

### Task 1: Credit card statement parser
- TODO — parser for the user's credit card statement export, needed
  alongside the current account to see full monthly spending.
- Format discovered 2026-07-11: Barclaycard exports CSV only
  (`Date, Account/Card No, Amount, Subcategory, Memo`; DD/MM/YYYY
  dates, UTF-8 BOM, thousands separators, embedded tabs/newlines in
  memos, sign convention inverted vs bank statements, masked card
  number usable as account identity, `Subcategory` distinguishes
  Purchase / Payment received / Other). Data-quality constraint:
  every amount is rounded to whole pounds — see the spend ledger
  design doc.

### Task 2: Evaluate Barclaycard PDF export
- TODO — the CSV export rounds every amount to whole pounds, which is
  not good enough for the spend ledger. The user will download a PDF
  statement export instead; evaluate whether it carries penny-precise
  amounts and is parseable, then decide the primary CC import format
  (PDF vs rounded CSV).

## Delta: Amazon Order Import

Amazon order data is a form of spend enrichment (see "Spend Enrichment"
in `doc/domain/ubiquitous-language.md`): a lump "AMAZON" card charge becomes
proper line items with real product descriptions, the same "more
informative source enriches the spend entry" pattern as transfer notes,
just sourced from Amazon instead of a bank transfer. The user considers
this a core enrichment worth automating, not a one-off manual chore.

### Task 1: Evaluate automation route — email scanning vs manual export
- TODO — two candidate routes, not yet compared:
  1. **Email scanning** — parse Amazon order-confirmation emails
     automatically (requires mail access — IMAP or a specific mail
     provider's API; scope and privacy implications not yet assessed).
     Preferred if feasible, since it needs no recurring manual step.
  2. **Manual export** — Amazon "Request my data" export or the order
     history CSV/page (format TBD), imported like any other statement.
     Simpler, requires the user to periodically request/download it.
- TODO — decide which to build first (or both, e.g. manual export as a
  fallback for orders predating email-scanning setup).

### Task 2: Amazon order import
- TODO — parse the chosen format so Amazon purchases show up as proper
  line items rather than one lump "Amazon" transaction per card charge.

## Delta: Spend Ledger

Derive a spend ledger — real-world spending to merchants and people —
from the raw imported transactions, excluding internal transfers
between household accounts (which would double-count purchases paid
for by matching transfers). This is the layer that gets categorised
and analysed; raw transactions stay immutable evidence. Design:
`doc/implementation-notes/spend-ledger-design.md`. Decision trail:
`doc/adr/0005-independent-spend-and-income-ledgers.md` (independent
spend and income ledgers; income deferred). Supporting research:
`doc/kb/ofx/structure.md` (OFX spec + observed Barclays NAME
encodings that make transfer detection deterministic) and
`doc/domain/household.md` ("Household" = the accounting entity).

### Task 1: Spend ledger design
- ✓ DONE — design session 2026-07-11: full design written to
  `doc/implementation-notes/spend-ledger-design.md` (derived
  `spend_entries` + `spend_entry_sources` provenance edge table,
  classification metadata rule/matcher/manual + confidence with
  manual-always-wins, transfer detection via household account
  registry, spend enrichment from transfers onto spend entries,
  double-entry compatibility mapping). ADR 0005 accepted. Open
  questions 1 (household membership → optional config) and 4
  (derivation runs as part of `ledgr import`, provisionally) decided;
  2, 3, 5 explained and awaiting the user's confirmation.

### Task 2: Spend ledger schema and derivation
- ✓ DONE — schema: `spend_entries` + `spend_entry_sources`
  (`src/db/schema.sql`), `sort_code`/`account_number` on `accounts`,
  `trn_type` added to `transactions` (needed to disambiguate
  `DIRECTDEP`/`CASH`/`DIRECTDEBIT`/`PAYMENT`/`REPEATPMT` — not in the
  original design doc schema sketch, added during implementation),
  `transactions.category_id` dropped. `BarclaysOfxParser` now
  populates `sort_code`/`account_number`/`trn_type`; existing accounts
  get backfilled on next import via `find_or_create_account`.
- ✓ DONE — household config: `Config.household_accounts` (hand-edit
  the config file — no CLI setter yet, see `HouseholdAccountRef` in
  `src/config.rs`); imported accounts are household members
  automatically.
- ✓ DONE — derivation pass: `src/derive.rs`, wired into `ledgr import`
  after `import_inbox`. Implements design doc rules 1-7 (own-account
  transfer, external-account payment, person FT payment/reimbursement,
  card payment, card refund, DIRECTDEP/CASH out-of-scope), rule
  precedence (account-prefix match beats TRNTYPE, so e.g. a standing
  order into a household savings account is never misclassified as
  spend), transfer pairing (`transaction_links relation='transfer'`,
  ±3 day window, idempotent), refund linking
  (`relation='refund'`, best-effort merchant-prefix match), and
  gates spend/refund classification on `AccountType::is_spending_account`
  (only current accounts + credit cards produce spend — savings
  accounts only see transfers, confirmed in real data). Idempotent by
  construction: only transactions with no `spend_entry_sources(role='source')`
  row are re-considered, so re-running `ledgr import` never
  double-derives.
- Deliberately out of scope for this task (see design doc's phasing
  note): rules 8-10 (Barclaycard CSV `Subcategory`) have no code path
  yet since no parser produces that field (Credit Card Statement
  Import Task 1 still TODO); spend enrichment (copying a transfer's
  note onto a later spend entry) is deferred to a follow-up.
- 51 unit tests total (up from 37), all passing; `cargo clippy` clean
  (only pre-existing-style dead-code warnings for forward-looking API
  surface not yet wired to a CLI/TUI consumer, same pattern as
  `get_account`/`balance_as_of` already had).
- **Follow-up session (2026-07-12) resolved the three open questions
  from the implementation session:**
  1. Real local `ledgr.db` migration — still TODO, deliberately not
     done autonomously (touches real data). Needs: create
     `spend_entries`/`spend_entry_sources`, add `accounts.sort_code`/
     `account_number`, add `transactions.trn_type`, drop
     `transactions.category_id` (SQLite can't drop a column in place
     on old versions — check `ALTER TABLE ... DROP COLUMN` support, or
     recreate the table as done for the `AccountType::Checking` →
     `Current` rename). Do this, then run `ledgr import` for real and
     sanity-check the derived spend entries against the actual
     imported transactions.
  2. **The `"fallback"` rule explained:** in `classify_inner`
     (`src/derive.rs`), after rules 1-7 all fail to match, any
     remaining transaction with a negative amount (money out) becomes
     a low-confidence (0.4) spend entry (`rule_name = "fallback"`)
     rather than being left unclassified. Reasoning: the design doc's
     rules table doesn't cover every possible `NAME`/`TRNTYPE`
     Barclays could produce (e.g. the foreign-currency card pattern
     noted in the OFX KB article, `"AMOUNT IN NOK"`, doesn't match any
     `CPM`/`CRM`/`FT` suffix). Without a fallback, an unrecognised
     outbound transaction would just sit unclassified forever —
     silently understating spend with no visible signal anything was
     missed. With it, the transaction still shows up in the ledger at
     low confidence, ready to surface in the future review queue
     (Task 3) for a manual fix. Inbound (positive-amount) unmatched
     transactions do **not** get this treatment — they're left out of
     scope, since guessing "not spend" for unknown inbound money is
     safe, but guessing "not spend" for unknown outbound money would
     make the ledger wrong with no indication. Kept as-is; no change
     requested.
  3. **`is_spending_account` gate — removed.** The user's real
     spending accounts are Jims Premier Account (main current),
     Online Spending, the Bills Account, and the credit card (once
     Credit Card Transaction Import lands) — i.e. most of what's
     imported. Rather than maintain a configurable list of which
     accounts count, the decision is to **scan every account
     uniformly** and rely on transfer pairing/reconciliation (rules
     1-2, `transaction_links`) to keep internal movement out of the
     ledger — not a pre-filter by account type. **Done** (2026-07-12 session): removed the `is_spending_account` check and its call site in `derive_spend_entries` (`src/derive.rs`) — `classify()` no longer takes an `is_spending_account` bool and now scans every account uniformly, relying solely on transfer pairing/reconciliation (rules 1-2) to keep internal movement out of the ledger. Deleted `AccountType::is_spending_account` (`src/model.rs`) entirely. Updated the design doc's Account registry section to match. Recorded as ADR `doc/adr/0006-no-account-type-gate-on-spend-derivation.md`. All 51 unit tests still pass; `cargo clippy` clean (same pre-existing dead-code warnings as before, nothing new).
- ✓ DONE — migrated the real local `ledgr.db` (2026-07-12 session): added `accounts.sort_code`/`account_number` columns, added `transactions.trn_type` column, dropped `transactions.category_id` (had to `DROP INDEX idx_transactions_category` first since SQLite's `ALTER TABLE ... DROP COLUMN` refuses if an index still references the column), then applied `schema.sql` to create `spend_entries`/`spend_entry_sources` and their indexes. Took a full file backup first (`~/.local/share/ledgr/ledgr.db.bak-20260712113230`) and validated the whole migration end-to-end on a scratch copy before touching the real file. Ran `ledgr status` and `ledgr import` against the real migrated database: 967 transactions across 4 real accounts, derivation created 804 spend entries and correctly left 163 out of scope. One known gap surfaced: the 4 existing accounts still have `NULL` `sort_code`/`account_number` since those columns are only backfilled by `find_or_create_account` when a file is actually (re-)parsed, and all 4 accounts' OFX files are already fully imported (deduped by file hash) — so household transfer detection currently finds 0 candidates for this real data until a genuinely new file per account is imported. Not fixed this session (backfilling by hand would require trusting an OFX field-mapping assumption — `BANKID`/`ACCTID` vs the sort-code+account-number split the `NAME` field actually uses — that wasn't verified); flagged as a follow-up, not blocking today's work. A stray test account/statement row (id 5 / statement 6) was created as a side effect of test-running `ledgr import` against the real inbox's pending Barclaycard CSV (which the current `GenericCsvParser` can't parse — thousands-separator amounts, unrelated to this migration) — cleaned up immediately, no transactions were ever attached to it.

### Task 3: Review and re-classification TUI
- TODO — **deprioritised below Delta: The Gap** (the user wants total
  spend/income/gap working before investing in a categorisation UI).
  Review queue screen for low-confidence/uncategorised spend entries;
  single-key actions to mark internal transfer / not-spend, set
  category, edit note; manual actions stamp `classified_by='manual'`.

## Delta: The Gap

Builds directly on the spend ledger rather than extending it further:
compute **The Gap** (income − spending for a period — the central
Rebel Finance metric, see the ubiquitous language doc) without waiting
for spending categorisation, which can come later. Requires activating
the previously-deferred income side (ADR 0005 deferred the income
ledger deliberately; this delta is what un-defers it) at minimal
scope — just enough to sum income, no categorisation.

### Task 1: Minimal income ledger
- TODO — `income_entries` (+ `income_entry_sources`, same provenance
  shape as `spend_entries`) per ADR 0005 and the spend ledger design
  doc's "Scope: spend ledger first" section, but deliberately thin: no
  categorisation, no taxonomy — just enough fields to sum income for a
  period (occurred_on, amount_minor, currency, counterparty,
  description, classification provenance). Derivation: at minimum,
  `DIRECTDEP` (and CC statement `Other`, once the CC parser exists)
  become income entries; reuse `derive.rs`'s existing account-scanning
  and household/transfer-detection machinery rather than duplicating
  it — internal transfers must stay excluded from income exactly as
  they are from spend.

### Task 2: Gap calculation
- TODO — for a given period: `SUM(income_entries) − |SUM(spend_entries)|`.
  Decide surface: a CLI command (`ledgr gap`, consistent with the
  existing `ledgr status` pattern) versus a TUI view — lean CLI first
  since it's the fastest path to a usable number, TUI view can follow
  under TUI Analysis Views.

### Task 3: Discovery about recording assets and liabilities
- ✓ DONE — decided: do **not** pivot to double-entry now. Extend the
  existing `accounts` + `balance_snapshots` machinery (already built
  for reconstructing a bank balance at an arbitrary date via
  `Db::balance_as_of`) to cover assets/liabilities generally, rather
  than introducing journal entries. Recorded as ADR
  `doc/adr/0007-assets-and-liabilities-as-accounts-with-balance-snapshots.md`.
  Full reasoning and consequences in the ADR; see Task 4 for the
  concrete build.

### Task 4: Implement assets and liabilities as accounts
- TODO — per ADR 0007: add new `AccountType` variants for the house,
  mortgage, and pension (exact categorisation/naming TBD at
  implementation time); add a manual balance-snapshot entry path (new
  CLI command + `Db` method) for accounts with no automated feed (the
  house always; the pension whenever a report isn't being parsed);
  mortgage/pension statement imports, if/when a format exists, reuse
  `ImportFileParser::balance_snapshot()` like any other institution —
  no new mechanism. Net worth: latest balance per account, summed with
  assets positive / liabilities negative by account type.

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

## Delta: Other Transaction Import

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
exploratory / doc-only so far — not blocking Bank Transaction Import
(still waiting on a real Barclays OFX export). Lower priority than the
other deltas since it's a bigger architectural fork (dependency on a
hosted aggregator vs ledgr's "nothing leaves the machine" local-file
model) than adding a new `ImportFileParser`.

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

## Delta: Double-Entry Accounting

Future/exploratory. The user is considering introducing double-entry
accounting at some point. The spend ledger design
(`doc/implementation-notes/spend-ledger-design.md`, "Future:
double-entry compatibility") records how the current derived-ledger
model maps onto it (spend entries → expense-account postings,
categories → chart of accounts, internal transfers → asset↔asset
transactions, household accounts → asset/liability typing). Nothing
should be built in a way that blocks this.

### Task 1: Evaluate a double-entry model for ledgr
- TODO — study Firefly III / beancount / GnuCash models; decide
  whether and when to adopt; ADR if adopted.
- Note (2026-07-12): ADR 0007 (see Delta: The Gap, Task 3) chose a
  lightweight interim model for assets/liabilities — accounts with
  (manual or imported) balance snapshots, not journal entries — rather
  than pulling this delta forward. Revisit this evaluation if daily use
  shows a real need for postings-level detail (e.g. splitting a
  mortgage payment into interest vs. principal) that the interim model
  can't give.

## Delta: Statement/Import Naming Cleanup

The domain term **"statement"** (the `statements` table, `statement_id`
columns, "Bank Statement Import"/"Credit Card Statement Import" delta
names) didn't fit what's actually being recorded once assets/
liabilities and non-bank sources are in scope — a manually-recorded
house valuation or a downloaded pension report isn't a "statement" in
the banking sense. Flagged 2026-07-12 as needing a rename across code,
schema, and docs. Per this project's ubiquitous-language rule
(`doc/domain/ubiquitous-language.md`, `CLAUDE.md`), the new term was
agreed with the user before renaming anything.

### Task 1: Agree the replacement term
- ✓ DONE (2026-07-12) — agreed **"Import"**: `statements` table →
  `imports`, `statement_id` columns → `import_id`, `StatementParser`
  trait → `ImportFileParser`. Considered and rejected: *export*,
  *download*, *import file*. "Import" was chosen despite already
  naming the `ledgr import` command/run — one **import** is one file;
  running `ledgr import` processes a batch of zero or more imports in
  one run (`ImportSummary`), which the user confirmed is coherent, not
  a collision. Recorded in `doc/domain/ubiquitous-language.md`. The
  delta names themselves were also agreed to change, since they used
  the same retired word: "Bank Statement Import" → **"Bank Transaction
  Import"**, "Credit Card Statement Import" → **"Credit Card
  Transaction Import"**, "Other Statement Import" → **"Other
  Transaction Import"** — "Transaction Import" was chosen over
  "Import" for the delta names specifically since ledgr may import
  other things (e.g. balance-only pension reports) beyond
  transactions.

### Task 2: Refactor to the agreed term
- ✓ DONE (2026-07-12) — renamed throughout: `statements` table →
  `imports` (`src/db/schema.sql`), `transactions.statement_id` /
  `balance_snapshots.statement_id` → `import_id`, `Transaction`/
  `NewTransaction.statement_id` → `import_id` (`src/model.rs`),
  `StatementParser` trait → `ImportFileParser` (`src/import/mod.rs` and all
  implementations), `Db::insert_statement`/`find_statement_by_hash` →
  `insert_import`/`find_import_by_hash`, `src/db/statements.rs` →
  `src/db/imports.rs`, delta names (this plan's Summary table and
  section headers), and code-identifier references in ADRs 0002/0003/
  0007, `doc/kb/enable-banking-registration.md`,
  `doc/implementation-notes/spend-ledger-design.md`, and `CLAUDE.md`'s
  trait snippet. Generic English uses of "statement" (e.g. "OFX
  statement response", "bank statement", "PDF statement export") were
  deliberately left alone — only the retired domain term and its code/
  schema identifiers were renamed. Real local `ledgr.db` migrated via a
  new idempotent step in `Db::init` (`src/db/mod.rs`,
  `migrate_statements_to_imports`): renames the `statements` table and
  both `statement_id` columns on first open after upgrade, no-op on a
  fresh or already-migrated database (mirrors the pattern used for the
  `AccountType::Checking` → `Current` rename).

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
- Added `Db::insert_import` / `Db::find_import_by_hash`
  (`src/db/imports.rs`) and `Db::find_or_create_account`
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
4. Move on to Credit Card Transaction Import once Bank Transaction Import is
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
Bank Transaction Import (Barclays OFX) remains the active priority,
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
4. Move on to Credit Card Transaction Import once Bank Transaction Import is
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
  validated against real data — Task 1 of Bank Transaction Import marked
  done.
- Also this session (documentation-only, before the above): reviewed
  and corrected `doc/kb/enable-banking-registration.md` (a
  Gemini-generated doc on using Enable Banking for live Open Banking
  API access) against Enable Banking's real docs, added a Security
  implications section and a note on the painful manual Barclays OFX
  download UX, and added a new "Live Open Banking (Enable Banking)"
  Delta to the plan to track it as a lower-priority exploratory thread.

**State of the project:**
Bank Transaction Import is now functionally proven end-to-end against
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
4. Move on to Credit Card Transaction Import once Bank Transaction Import
   (Task 2) is fully done.

## Checkpoint: Session 2026-07-11d

**What was completed this session:**
- Added `ledgr status` CLI command (`src/main.rs`) — this surfaced two
  real bugs in the "validated" Barclays OFX import from the previous
  checkpoint.
- Bug 1: all 3 real OFX files had been collapsing into one hardcoded
  "Barclays Current Account" instead of 3 separate real accounts.
  Fixed by adding `ImportFileParser::account_identity()`
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
  trend views. New `ImportFileParser::balance_snapshot()`;
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
Bank Transaction Import is now genuinely correct against real data, not
just "imports without crashing" — account separation and balances both
verified against the source OFX files' own reported values (`ACCTID`,
`LEDGERBAL`). The balance-anchor infrastructure (`balance_as_of`) is
generic enough to directly support the still-TODO net worth/trend view
work later. Only per-transaction/generic-CSV de-dup refinement remains
before Bank Transaction Import is fully done. The TUI still hasn't caught
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
5. Move on to Credit Card Transaction Import once Bank Transaction Import
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
banking conventions throughout. Bank Transaction Import itself is
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
4. Move on to Credit Card Transaction Import once Bank Transaction Import
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
Bank Transaction Import's de-dup story is now complete for formats that
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
4. Move on to Credit Card Transaction Import once Bank Transaction Import
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
Bank Transaction Import's de-dup story (Task 2) is now fully validated
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
4. Move on to Credit Card Transaction Import once Bank Transaction Import
   (Task 2) is fully done.

## Checkpoint: Session 2026-07-11h

**What was completed this session:**
- Design session for the spend ledger: wrote
  `doc/implementation-notes/spend-ledger-design.md` — raw transactions
  stay immutable evidence; a derived `spend_entries` table (with
  `spend_entry_sources` provenance links) holds real-world spending;
  internal transfers between household accounts produce no entries;
  classification carries rule/matcher/manual provenance + confidence,
  manual always wins; derivation runs as part of `ledgr import`.
- ADR 0005 `doc/adr/0005-independent-spend-and-income-ledgers.md`:
  independent spend and income ledgers (income deferred), reversing an
  initial single-table-with-kind decision after the naming difficulty
  exposed that spend and income are different domains.
- Researched the OFX spec properly and wrote `doc/kb/ofx/structure.md`:
  full STMTTRN/TRNTYPE reference plus observed Barclays behaviour —
  Barclays never emits XFER or BANKACCTTO; transfers are identified by
  sort code + account number packed into the 32-char NAME field, and
  both sides of a transfer carry the same user reference, making
  internal-transfer detection and pairing deterministic.
- Examined the real Barclaycard CSV export in the inbox: usable format
  but every amount is rounded to whole pounds (all 205 rows) — recorded
  as a data-quality constraint; parser details added to the Credit Card
  Transaction Import delta.
- Started the domain docs: `doc/domain/ubiquitous-language.md` (terms
  with provenance and status) and `doc/domain/household.md` (the
  "Household" accounting-entity concept — alternatives considered,
  Rebel Finance/economics evidence, adopted). Added a CLAUDE.md rule:
  no new domain terms without consulting the ubiquitous language doc
  and the user.
- Scrubbed real account numbers, names, and amounts from all new docs
  (repo may become public). NOTE: this plan file still contains real
  account last-4 digits, account nicknames, balances, and a Google
  Drive path with an email address — scrub before publishing.
- Added the Double-Entry Accounting exploratory Delta and the Spend
  Ledger Delta (design ✓ DONE).

**State of the project:**
The spend ledger is fully designed and decision-trailed (ADR 0005,
ubiquitous language, OFX research) but not yet implemented — schema
and derivation are the next build step. Import infrastructure is
unchanged and solid. Three design open questions await the user:
reimbursements as refund-of-spend, the sinking-fund convention, and
whether a precise (non-rounded) Barclaycard export exists.

**Immediate next priorities:**
1. Confirm spend-ledger open questions 2, 3, 5 with the user.
2. Implement the spend ledger schema + derivation pass (Spend Ledger
   Task 2).
3. Credit card CSV parser (Credit Card Transaction Import Task 1) — the
   spend ledger wants CC data; format is now known.
4. Review/re-classification TUI (Spend Ledger Task 3).
5. Generic-CSV de-dup strategy (Bank Transaction Import Task 2) remains
   the oldest open TODO.

## Checkpoint: Session 2026-07-12

**What was completed this session:**
- Reviewed the spend ledger design doc end-to-end and added a Summary
  section flagging its one piece of real scope creep: spend
  enrichment (renamed from "note propagation" — a fuzzy amount/date
  match that copies a transfer's reference onto a later spend entry's
  note) is UX polish layered on deterministic transfer detection, and
  was agreed to defer past the first implementation.
- Renamed "note propagation" → **Spend Enrichment** throughout (design
  doc, ubiquitous language doc with provenance, this plan) at the
  user's request.
- Expanded the Amazon Order Import delta: reframed as a form of spend
  enrichment (a lump "AMAZON" card charge → real line items), split
  into Task 1 (decide email-scanning vs manual-export automation
  route — the user wants this automated, not a recurring manual
  chore) and Task 2 (the parser itself).
- Cross-checked the spend ledger design's derivation rules table
  against the OFX KB article and found two real gaps: no stated
  precedence between the account-number transfer check and
  TRNTYPE-based rules (a standing order into a household savings
  account could otherwise be misclassified as spend), and
  `TRNTYPE=CASH` (cash withdrawals) had no rule at all. Fixed both in
  the design doc (explicit rule ordering + numbering, a `CASH` row,
  and a note that `NAME`'s 32-char cap can truncate reference text so
  matching must tolerate that).
- Implemented Spend Ledger Task 2 (schema + derivation), phased per
  the Summary: schema, rules 1-7, transfer pairing, and refund linking
  now; rules 8-10 (Barclaycard CSV) and spend enrichment deferred (see
  Task 2 above for full detail). New files: `src/derive.rs` (rules +
  orchestration), `src/db/spend.rs` (persistence). Touched: schema,
  model, config, both parsers, `db/accounts.rs`/`transactions.rs`/
  `status.rs`, `analysis.rs` (`category_totals` now reads
  `spend_entries`, since raw transactions no longer carry a
  category), `main.rs` (derivation now runs as part of `ledgr
  import`). 51 unit tests passing (up from 37), `cargo clippy` clean.

**State of the project:**
The spend ledger has moved from fully-designed to code-complete for
its first phase: raw transactions stay immutable, `ledgr import` now
also derives `spend_entries` and pairs internal transfers, gated so
only current accounts/credit cards produce spend (savings accounts
are transfer-only, confirmed in real data). None of this has run
against the user's real local database yet — the schema changes
(new columns/tables, a dropped column) need a manual migration first,
consistent with how past schema changes to the real `ledgr.db` were
handled. TUI still hasn't caught up to any of the spend ledger work;
transaction/account browsing remains the pre-existing scaffold.

**Immediate next priorities:**
1. Migrate the real local `ledgr.db` by hand (new `spend_entries`/
   `spend_entry_sources` tables, `sort_code`/`account_number`/
   `trn_type` columns, drop `transactions.category_id`), then run
   `ledgr import` for real and sanity-check the derived spend entries
   against the actual 939+ imported transactions.
2. Credit card CSV parser (Credit Card Transaction Import Task 1) — once
   in, wire its `Subcategory` field into `derive.rs` for rules 8-10
   (currently unreachable, no parser produces that field yet).
3. Review/re-classification TUI (Spend Ledger Task 3) — `list_spend_entries`
   already exists as a read path to build it on.
4. Spend enrichment as a follow-up pass once the core ledger is in
   daily use (deferred from Task 2 by design).
5. Amazon Order Import Task 1 — decide email-scanning vs
   manual-export automation route.

**Open questions raised at the end of this session — resolved in the
2026-07-12 follow-up session, see the checkpoint below:**
- Real `ledgr.db` migration → still TODO, explicitly deferred rather
  than done autonomously (see Spend Ledger Task 2).
- The `"fallback"` rule → explained, kept as-is (see Task 2).
- `is_spending_account` → decision reversed: remove it, scan all
  accounts, rely on transfer pairing/reconciliation instead (see
  Task 2). Code change not yet made — carried into the next session.

## Checkpoint: Session 2026-07-12 (follow-up)

**What was completed this session:**
- Resolved the three open questions from the previous session (full
  detail recorded under Spend Ledger Task 2 above): (1) real-database
  migration confirmed still pending, explicitly not run without the
  user's go-ahead; (2) the `"fallback"` classification rule explained
  and kept as-is; (3) decided to **remove** `is_spending_account`
  entirely rather than make it configurable — scan every account
  uniformly for spend, and trust transfer pairing (reconciliation) to
  keep internal movement out of the ledger, rather than pre-filtering
  by account type. This code change is not yet made.
- Spend Ledger delta declared **closed at its current basic scope** —
  Task 3 (review/re-classification TUI) deprioritised, not cancelled.
- Scoped and added a new delta, **Delta: The Gap**, building directly
  on the spend ledger: compute income − spending for a period without
  waiting for spend categorisation. Un-defers the income ledger from
  ADR 0005 at deliberately minimal scope (no categorisation/taxonomy,
  just enough fields to sum income) — Task 1 (minimal income ledger)
  and Task 2 (gap calculation, `ledgr gap` CLI leaning ahead of a TUI
  view).
- This was a planning-only session (context-constrained) — no code
  changes made; everything above is captured in the plan for the next
  session to act on.

**State of the project:**
Spend Ledger Task 2 code is unchanged from the previous session
(schema + derivation, 51 tests passing) except for one known-pending
fix: the `is_spending_account` gate needs removing before this is
truly "closed at the basics." The real local database still hasn't
been migrated, so none of this has run against real data yet. The
project's near-term direction has shifted from "deepen the spend
ledger" to "close it out at the basics and get to a working Gap
number" — categorisation (Spending Categorisation delta) and the
review TUI (Spend Ledger Task 3) both now sit behind Delta: The Gap.

**Immediate next priorities:**
1. Remove `is_spending_account` gating from `src/derive.rs` (and the
   method itself from `src/model.rs` if unused elsewhere); update the
   design doc's Account registry section to match.
2. Migrate the real local `ledgr.db` by hand, then run `ledgr import`
   for real and sanity-check the derived spend entries.
3. Delta: The Gap, Task 1 — minimal `income_entries` schema +
   derivation (DIRECTDEP → income, reusing `derive.rs`'s
   household/transfer-detection machinery).
4. Delta: The Gap, Task 2 — gap calculation, likely `ledgr gap`.
5. Credit card CSV parser (Credit Card Transaction Import Task 1) — CC
   data needed for a complete spend picture; also unlocks derivation
   rules 8-10.

## Checkpoint: Session 2026-07-12c

**What was completed this session:**
- Removed the `is_spending_account` account-type gate from spend ledger derivation (`src/derive.rs`, `src/model.rs`) — derivation now scans every account uniformly and relies solely on transfer pairing/reconciliation to exclude internal movement. Recorded as ADR 0006 (`doc/adr/0006-no-account-type-gate-on-spend-derivation.md`), added to `doc/adr/decisions.md`, and the spend ledger design doc's Account registry section updated to match. All 51 tests pass, clippy clean.
- Migrated the real local `ledgr.db` to the current schema (new `sort_code`/`account_number`/`trn_type` columns, dropped `transactions.category_id`, added `spend_entries`/`spend_entry_sources`) after validating the migration end-to-end on a scratch copy first. Confirmed `ledgr status` and `ledgr import` both work against the real migrated database: 967 real transactions across 4 accounts, 804 spend entries derived, 163 correctly out of scope. Surfaced (but did not fix) a follow-up gap: existing accounts' `sort_code`/`account_number` won't backfill until a genuinely new file is imported per account, so transfer detection currently has no real household data to work with yet.
- Scoped a new Task 3 under Delta: The Gap — "Discovery about recording assets and liabilities" — capturing that the user's only assets/liabilities beyond day-to-day banking are the house, its mortgage, pension funds, and the monthly credit card balance, and that properly capturing these raises the question of whether ledgr should move towards an explicit assets/liabilities model or a full double-entry pivot (with `spend_entries`/`income_entries` becoming projections over journal entries). Not designed yet — flagged for discussion before Task 1/2 implementation.

**State of the project:**
Spend Ledger delta is now fully closed at its basic scope (Task 2 has no more carried-over loose ends) and validated against the real, now-migrated local database. Delta: The Gap has grown from two tasks to three — the income ledger and gap calculation are still TODO, and a new discovery task now sits ahead of (or alongside) them to settle how assets/liabilities should be recorded before committing to an implementation, given it may reshape the ledger architecture towards double-entry.

**Immediate next priorities:**
1. Discuss and settle the assets/liabilities recording model (Delta: The Gap, Task 3) — lightweight assets/liabilities ledger vs. a fuller double-entry pivot — before starting Task 1.
2. Delta: The Gap, Task 1 — minimal income ledger (`income_entries` + `income_entry_sources`).
3. Delta: The Gap, Task 2 — gap calculation (`ledgr gap` CLI).
4. Follow-up: backfill `accounts.sort_code`/`account_number` for the 4 existing real accounts (or wait for a genuinely new file per account) so household transfer detection has real data to work with.
5. Credit card CSV parser (Credit Card Transaction Import Task 1) — still the oldest open TODO, also unlocks derivation rules 8-10.

## Checkpoint: Session 2026-07-12d

**What was completed this session:**
- Settled Delta: The Gap, Task 3 (assets/liabilities recording model):
  decided against pivoting to double-entry now. Accounts + balance
  snapshots (the existing bank-balance-reconstruction machinery) are
  extended to cover the house, mortgage, and pension instead — new
  `AccountType` variants, a manual balance-snapshot entry path for
  accounts with no automated feed, and parser-driven snapshots reused
  as-is when a mortgage/pension statement format exists. Recorded as
  ADR `doc/adr/0007-assets-and-liabilities-as-accounts-with-balance-snapshots.md`,
  added to `doc/adr/decisions.md`. Added Task 4 ("Implement assets and
  liabilities as accounts") to carry the build. Noted on Delta:
  Double-Entry Accounting as the live interim decision, to be revisited
  only if postings-level detail (e.g. mortgage interest/principal
  split) is actually needed later.
- Scoped a new delta, **Delta: Statement/Import Naming Cleanup**: the
  domain term "statement" (the `statements` table, `Statement` model,
  delta names like "Bank Transaction Import") no longer fits once
  non-bank/manually-recorded sources are in scope. Candidates discussed
  informally: "import", "export" — not agreed yet. Per the project's
  ubiquitous-language rule, split into Task 1 (agree the term with the
  user, consulting `doc/domain/ubiquitous-language.md`) and Task 2 (the
  refactor itself — schema, code, delta names, docs). Deliberately
  deferred to its own session rather than done now.

**State of the project:**
Both open threads from the previous session are now resolved at the
decision level: the account-type gate removal + real DB migration are
code-complete (previous checkpoint), and the assets/liabilities
question now has an accepted ADR. Delta: The Gap is unblocked to
proceed with Task 1 (income ledger); Task 4 (assets/liabilities build)
can run alongside or after it. The statement/import naming rename is
scoped but explicitly parked for a dedicated future session, since it
touches schema, code, and a wide set of docs.

**Immediate next priorities:**
1. Delta: The Gap, Task 1 — minimal income ledger (`income_entries` +
   `income_entry_sources`).
2. Delta: The Gap, Task 2 — gap calculation (`ledgr gap` CLI).
3. Delta: The Gap, Task 4 — implement assets/liabilities as accounts
   per ADR 0007 (new `AccountType` variants, manual balance-snapshot
   entry command).
4. Delta: Statement/Import Naming Cleanup, Task 1 — agree the
   replacement term with the user in a dedicated session.
5. Credit card CSV parser (Credit Card Transaction Import Task 1) — still
   the oldest open TODO, also unlocks derivation rules 8-10.

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
- New statement formats are added by implementing `ImportFileParser`
  in `src/import` (see `generic_csv.rs` for the existing example).
- Project is an early scaffold — not yet functional end-to-end.
