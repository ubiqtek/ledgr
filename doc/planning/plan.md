# ledgr — Plan

## What's Next

**Next:** Delta: Credit Card Transaction Import, Task 5 — match credit card payments to bank-side transfers (date+amount matching and/or card-number-prefix detection) so bill payments stop leaking into spend. Delta: The Gap, Task 2 (monthly "total spend" shape) remains the task after that.
**Sub-doc:** (none)
**Blockers:** None currently.

**2026-07-12 — `ledgr status` reformatted as tables; Shared Shopping
Account corrected from reference to tracked:** `run_status`
(`src/main.rs`) now prints both the real-accounts section (renamed
**"Tracked Accounts"**, new domain term added to
`doc/domain/ubiquitous-language.md`) and Household Reference Accounts
as aligned tables via a new `print_table` helper, dropped the
Institution/Type/sort-code columns (not needed), shortened account
numbers to last-4-digits `(nnnn)` style, right-aligned the `Txns`
column, and balance now shows just the amount (dropped the "(as of
...)" annotation). While reviewing the reference table, the user
flagged that **Shared Shopping Account** was miscategorised as a
reference household account — it's one of the user's own accounts
(same sort code, `208794`, as his other Barclays accounts) that simply
hadn't had a statement imported yet, unlike Romina's accounts which
genuinely never will be. Imported a real OFX export for it
(`ACCTID` confirmed sort `208794`/account `...3394`, matching the
config entry exactly): 24 transactions, £266.33 balance, 0 new spend
entries (its activity was already correctly excluded as internal
transfers via the reference-account config, so becoming a tracked
account didn't change spend totals). Renamed it via `ledgr
name-account 3394 "Shared Shopping Account"` and removed its now-
redundant entry from `config.toml`'s `household_accounts` (household
membership is now structural, via its own `accounts` row, same as any
other tracked account). A stray zero-transaction "Barclays Current
Account" row (id 5) was created as a side effect of `ledgr import`
choking on the pending Barclaycard CSV's thousands-separator amounts
(same known issue as before, see Task 2 below) before reaching the
OFX file — cleaned up by hand (`DELETE FROM accounts WHERE id = 5`,
confirmed 0 transactions attached first); worked around for this
import by temporarily moving the CSV out of the inbox and back.
Real `ledgr.db` backed up first
(`ledgr.db.bak-20260712-preimport`). `doc/domain/ubiquitous-language.md`
updated: **Reference Household Account**'s definition tightened to
make explicit it's for accounts that will *never* be imported, not
just "not yet imported" ones.
**Follow-up resolved (same day):** the user found the correct export
for Joint Annual Expense — this one's `ACCTID` (sort `208794`/account
`03868915`) matched the registered number exactly, confirming the
first download had genuinely been the wrong account, not a bad
registration. Imported via the same CSV-out/CSV-back workaround (no
stray account row this time, since the CSV was already out of the
inbox before running `ledgr import`): 37 transactions, £4.66 balance,
0 new spend entries. Renamed via `ledgr name-account 8915 "Joint
Annual Expense"`, removed its now-redundant `household_accounts` entry
from `config.toml`. `~/Downloads/annual-expenses-2026-07-12-WRONG-
ACCOUNT.ofx` queued for manual deletion (`rm` blocked in this
environment, command copied to clipboard). `ledgr.db` backed up first
(`ledgr.db.bak-20260712-preimport2`). **Household Reference Accounts
now correctly contains only Romina's two accounts** — the only ones
that genuinely will never be imported; all four originally-registered
entries have now been reviewed, two were miscategorised and are fixed.
`spend_entries` still 690 throughout; 52 tests pass.
**Follow-up (same day):** decimal points in the Tracked Accounts
`Balance` column now line up — added `align_decimal_column` (`src/
main.rs`), which left-pads each balance string with spaces so every
`.` lands in the same screen column (e.g. `7.47 GBP` and `3106.58
GBP` align on their decimals), applied before handing rows to
`print_table`. 52 tests pass.

**2026-07-12 — transfer-detection docs reframed around payment type
(no logic change):** `doc/developer-docs/transfer-detection.md`
referred to the two `NAME` encodings as "Shape 1"/"Shape 2" (later
briefly "leading shape"/"trailing shape"). Corrected framing per the
user: the document should lead with the real-world payment **types** —
**manual funds transfer** (leading position in `NAME`) and
**automated transfer** (direct debit or standing order; trailing
position) — not the structural shape, since "shape" is just the
technical detection mechanism, not the concept that matters. Either
type can be internal (household account) or external (spend); the
matching logic (sort code/account number against known household
accounts, else spend if money is going out) was confirmed correct
as-is, no code change — only doc framing changed. "Leading"/"trailing"
retained only in the structural detail (the `NAME` position check
itself, `Rule ordering`, `Known limitations`), not as the primary
framing. Also trimmed narrative flourishes throughout per the user's
request. No change to `doc/user-guide/transfer-detection.md` (already
used plain "start"/"end" language) or
`doc/domain/ubiquitous-language.md` (no new domain term introduced).

**2026-07-12 — partner's account registered as a reference household
account:** added Romina (wife)'s current account (sort `206325`, account
`40531189`) to `~/.config/ledgr/config.toml`'s `household_accounts` as
`"Romina (wife) Current account"`. Re-derived the 6 real transactions
already referencing her account number (backed up DB first,
`ledgr.db.bak-20260712*`) — all 6 correctly reclassified from spend to
internal transfer, `spend_entries` dropped 705 → 699, no duplicate
`transaction_links` from the reprocessing. Noticed in passing: a
*different* account (sort `206325`, account truncated to `23308324`) also
shows up in some standing-order descriptions (£14.99/month, landing in the
Bills Account) — not one the user has given account details for yet, left
unclassified/out-of-scope, not acted on; possibly another account of hers.
Named and settled the concept as **Reference Household Account** — ADR
`doc/adr/0008-reference-household-accounts.md`, recorded in
`doc/domain/ubiquitous-language.md`. `ledgr status` now lists configured
reference household accounts in their own section, explicitly labelled as
carrying no balance/transaction data, so their absence from the main
account list isn't mistaken for a bug.
**Minor inefficiency noted, not urgent:** `pending_derivation_transactions()`
(`src/db/spend.rs`) only excludes transactions that produced a
`spend_entries` row — `InternalTransfer`/`OutOfScope` classifications never
get a `spend_entry_sources` row, so every `ledgr import` run reprocesses
every transfer and out-of-scope transaction from scratch, not just newly
imported ones. Harmless today (idempotent — `transaction_links` has a
`UNIQUE` constraint, confirmed no duplicates after two reprocessing runs
this session) but will get slower as transaction volume grows; worth a
"mark as considered" mechanism for non-spend classifications if it becomes
noticeable.

**Follow-up (same session):** the £14.99/month `23308324` account
flagged above was confirmed by the user as also Romina's — added as a
second reference household account. Relabelled both for clarity: sort
`40531189` → **"Romina Primary Account"** (was "Romina (wife) Current
account"), sort `23308324` → **"Romina Secondary Account"**. Re-derived
the second account's 6 previously out-of-scope transactions (no
spend_entries needed clearing — out-of-scope transactions are always
still "pending", per the reprocessing behaviour noted above) — all 6
now correctly excluded as internal transfers. `ledgr status`'s
household-accounts section renamed to **"Household Reference
Accounts"** and given a proper columnar layout (Label / Sort Code /
Account Number, column-width computed from the longest label).

## Summary

| Delta | Task | Status |
|-------|------|--------|
| [Delta: Bank Transaction Import](#delta-bank-transaction-import) | [1. Barclays OFX parser](#task-1-barclays-ofx-parser) | ✓ DONE |
| | [2. Import de-duplication](#task-2-import-de-duplication) | IN PROGRESS |
| | [3. Account resolution and balance tracking](#task-3-account-resolution-and-balance-tracking) | ✓ DONE |
| [Delta: Automatic Inbox Import](#delta-automatic-inbox-import) | [1. Inbox change notification](#task-1-inbox-change-notification) | TODO |
| [Delta: Credit Card Transaction Import](#delta-credit-card-transaction-import) | [1. Credit card statement parser](#task-1-credit-card-statement-parser) | IN PROGRESS |
| | [2. Evaluate Barclaycard PDF export](#task-2-evaluate-barclaycard-pdf-export) | ✓ DONE |
| | [3. Import the user's partner's credit card](#task-3-import-the-users-partners-credit-card) | TODO |
| | [4. Manual spend entries via a proxy account](#task-4-manual-spend-entries-via-a-proxy-account) | TODO |
| | [5. Match credit card payments to bank-side transfers](#task-5-match-credit-card-payments-to-bank-side-transfers) | TODO |
| [Delta: Amazon Order Import](#delta-amazon-order-import) | [1. Evaluate automation route — email scanning vs manual export](#task-1-evaluate-automation-route--email-scanning-vs-manual-export) | TODO |
| | [2. Amazon order import](#task-2-amazon-order-import) | TODO |
| [Delta: Spend Ledger](#delta-spend-ledger) | [1. Spend ledger design](#task-1-spend-ledger-design) | ✓ DONE |
| | [2. Spend ledger schema and derivation](#task-2-spend-ledger-schema-and-derivation) | ✓ DONE |
| | [3. Review and re-classification TUI](#task-3-review-and-re-classification-tui) | TODO — deprioritised below Delta: The Gap |
| [Delta: The Gap](#delta-the-gap) | [1. Minimal income ledger](#task-1-minimal-income-ledger) | TODO |
| | [2. Gap calculation](#task-2-gap-calculation) | TODO |
| | [3. Discovery about recording assets and liabilities](#task-3-discovery-about-recording-assets-and-liabilities) | ✓ DONE |
| | [4. Implement assets and liabilities as accounts](#task-4-implement-assets-and-liabilities-as-accounts) | TODO |
| [Delta: Mortgage Tracking](#delta-mortgage-tracking) | [1. Design the mortgage domain model](#task-1-design-the-mortgage-domain-model) | TODO |
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
| [Delta: Payslip Import](#delta-payslip-import) | [1. Evaluate payslip format and scope](#task-1-evaluate-payslip-format-and-scope) | TODO |
| [Delta: Regular Payments](#delta-regular-payments) | [1. Design regular payment recognition and labelling](#task-1-design-regular-payment-recognition-and-labelling) | TODO |

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
- ✓ DONE (2026-07-12 session) — built and validated `BarclaycardPdfParser`
  (`src/import/barclaycard_pdf.rs`), parsing the Barclaycard PDF
  "Transactions" export (chosen over the CSV — see Task 2 below) via
  `pdf_extract::extract_text` + regex over the normalized text.
  Order-tolerant transaction regex handles both `<date> <type>` and the
  reversed `<type> <date>` that `pdf-extract` occasionally produces (a
  real quirk, not hypothetical — confirmed against the real export).
  Also strips "Page N of M" footers that can land mid-transaction across
  a page break. Validated end-to-end against a real 205-transaction
  Barclaycard export in a scratch-only inbox (never touched the real
  `ledgr.db`, real PDF never committed): all 205 transactions imported
  with correct signs (`Purchase` = money out; `Payment received`/`Other`
  = money in), correct account identity, and a balance snapshot matching
  the export's stated current balance exactly (£613.73).
- ✓ DONE — imported for real: real `ledgr.db` now has **Barclaycard**
  as a tracked account (205 transactions, £613.73 balance, matching the
  scratch validation exactly). `ledgr status` confirms it alongside the
  other 6 tracked accounts.
- New schema: `account_card_numbers` table (`src/db/schema.sql`) records
  every last-4-digits card number ever seen per account, since a
  statement export carries no stable account identity — only a masked
  last4 that changes on reissue (see
  `doc/kb/barclaycard/pdf-export-structure.md`). `last4` is globally
  unique (not per-account), so `Db::link_card_number`
  (`src/db/cards.rs`) can reassign a last4 away from a wrongly
  auto-created duplicate account onto the correct one once a human
  confirms a reissue — nothing is ever inferred automatically.
  `Db::find_or_create_credit_card_account` resolves a `CardIdentity`
  (new `src/model.rs` struct) to an account via this history instead of
  `find_or_create_account`'s institution+name matching (which would
  spawn a new account on every reissue). `AccountType::CreditCard`
  already existed in the schema/model, unused until now — no new account
  type needed.
- New generic `notes` column added to `transactions`
  (`src/db/schema.sql`, `src/model.rs`) as a catch-all for import-format
  detail that doesn't fit existing fields. Not yet populated by any
  parser (including this one) — `description`/`raw_description`/
  `trn_type` turned out sufficient for the PDF's fields after all.
- Known gaps, left as-is for now: (1) no per-transaction de-duplication
  across an overlapping re-export — same open problem as
  `GenericCsvParser`, only whole-file hash dedup applies; (2) the sign
  assumed for the `Other` type tag (always money in) is based on a small
  sample (only ever seen as Barclaycard Cashback); (3) one real
  transaction's description was found truncated (`"British"` instead of
  "British Triathlon, Loughborough") due to a page break splitting the
  description *after* the amount marker rather than before it — a
  different case from the page-footer-stripping already handled; amount
  and sign were still correct, only the description text was
  incomplete.
- Format discovered 2026-07-11: Barclaycard exports CSV only
  (`Date, Account/Card No, Amount, Subcategory, Memo`; DD/MM/YYYY
  dates, UTF-8 BOM, thousands separators, embedded tabs/newlines in
  memos, sign convention inverted vs bank statements, masked card
  number usable as account identity, `Subcategory` distinguishes
  Purchase / Payment received / Other). Data-quality constraint:
  every amount is rounded to whole pounds — see the spend ledger
  design doc.

### Task 2: Evaluate Barclaycard PDF export
- ✓ DONE (2026-07-12 session) — the PDF export is penny-precise
  (unlike the CSV, which rounds to whole pounds) and reliably
  text-extractable via `pdf_extract`. Decided: **PDF is the primary
  credit card import format**, CSV deprioritised/dropped. Full
  structure write-up, including card-number (PAN) structure/reissue
  research prompted by the user sharing a real card number mid-session,
  in `doc/kb/barclaycard/pdf-export-structure.md`.

### Task 3: Import the user's partner's credit card
- TODO (2026-07-12) — the user will load his partner's credit card
  statement as a normal Transaction Import (same parser as his own once
  Task 1/2 land), but not her personal bank accounts — those aren't
  going to be imported. This account will need registering as a
  **household account** (like the user's own accounts) so payments
  *to* her card from her own bank (i.e. her paying off her own card) are
  recognised as internal, not spend — needs her card account's real
  identity (masked card number, per Task 1's discovery) captured the
  same way the user's own accounts are. Her spend on her own accounts
  (uncaptured) is what Task 4 below covers.

### Task 4: Manual spend entries via a proxy account
- TODO (2026-07-12) — the user's partner's own spend (on her personal
  bank accounts, which won't be imported) still needs to count towards
  household spend/the Gap, entered manually on a rough cadence (e.g.
  monthly: "spent £200 this month on food"). Design idea agreed with the
  user: back a manual spend entry with a normal `Transaction` row on a
  new **proxy account** (an `Account` with no real sort code/account
  number, so it can never collide with or be mistaken for a real
  account) rather than changing the spend-entry schema — keeps
  `classified_by = 'manual'` (already in the schema's `CHECK`
  constraint, unused so far) working through the existing
  derivation/provenance model. New domain terms **Proxy Account** and
  **Manual Spend Entry** recorded as candidates in
  `doc/domain/ubiquitous-language.md`. Not yet designed: the actual
  entry flow (CLI command vs TUI form), or whether proxy accounts need
  their own `AccountType` variant.

### Task 5: Match credit card payments to bank-side transfers
- TODO — with the credit card account now real and importable (Task 1),
  credit card payments still need excluding from spend the same way
  inter-bank transfers are (see `derive.rs`'s existing transfer
  pairing). Two ideas discussed, not yet built:
  1. **Date + exact amount matching** between a bank-side outgoing
     payment and the card's `"PAYMENT, THANK YOU"` row — verified
     working against real June 2026 data (two payments matched to the
     penny and the day). Doesn't depend on the card number at all, so
     immune to reissue.
  2. **Card-number-prefix detection** — bank-side transfer descriptions
     to a credit card carry a truncated form of the card's PAN (e.g.
     `MR JAMES BARRITT 49291328548900`, missing the last 2 digits — see
     `doc/kb/barclaycard/pdf-export-structure.md`). The user proposed a
     function to detect "looks like the first N digits of a card
     number" in a transaction description, and — since there is
     currently only one registered Barclaycard — assuming any such match
     refers to it without needing an exact/current number. Not yet
     designed in detail (how the "only one card" assumption degrades
     once Task 3 adds the partner's card too).

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
- **Gmail access explored 2026-07-12 (doc-only, nothing built):** this
  Claude Code environment has no Gmail/mail integration configured today —
  no MCP server, no IMAP. Two setup paths discussed for the email-scanning
  route: (a) a Gmail MCP server (OAuth-based, needs a Google Cloud project +
  Gmail API enabled + OAuth credentials), or (b) direct IMAP (app password or
  OAuth2 IMAP), which would mean writing a small ledgr-side mail-fetching
  module since no existing tool/skill covers it. **Recommendation, not yet
  actioned:** avoid granting broad whole-inbox read access for this — set up
  a Gmail filter that forwards/labels Amazon order-confirmation emails only,
  so any future scanning integration touches a narrow, purpose-specific
  slice of mail rather than the full inbox. Narrows the privacy/scope
  question flagged above considerably before it needs answering in full.

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
- Prototyping session (2026-07-12): before building a "total spend"
  command, ad-hoc-queried `spend_entries` grouped by month directly
  against the real database — surfaced and fixed a real transfer-detection
  bug in the process (see below) rather than building a command on top of
  wrong numbers. Monthly spend totals (post-fix, real data): Jan £10,426,
  Feb £9,215, Mar £7,598, Apr £6,117, May £6,562, Jun £6,078, Jul (partial)
  £1,567. Not yet wired into a CLI command — the user wants to keep
  ad-hoc-querying/prototyping the shape (including a possible
  "closing the books" monthly record) before committing to schema/code for
  this task.
- **Real bug found and fixed:** the previously-flagged gap ("existing
  accounts' `sort_code`/`account_number` won't backfill until a genuinely
  new file is imported") turned out to hide a second, real bug once
  investigated. `BarclaysOfxParser::account_identity()`
  (`src/import/barclays_ofx.rs`) was storing OFX `BANKID` as `sort_code` —
  but `BANKID` is a fixed Barclays OFX-server identifier (`492900` for
  every account), not the customer-facing sort code. The real sort code +
  account number are concatenated inside `ACCTID` itself (first 6 digits =
  sort code, last 8 = account number) — confirmed by matching real
  transaction `NAME` fields against real account numbers. Fixed the
  parser to split `ACCTID` correctly. Also found transfers use a second
  `NAME` shape ledgr's matcher didn't handle: `"<label> <sort> <account>"`
  (label first) as well as the already-handled `"<sort> <account> <rest>"`
  (label last) — sometimes with the account number truncated to 6 digits
  when a long label pushes the `NAME` field past Barclays' length limit.
  Added `parse_trailing_account_suffix()` + truncation-tolerant
  `household_contains()` to `src/derive.rs`. Full write-up with real
  (anonymised) examples: `doc/developer-docs/transfer-detection.md`;
  user-facing explanation: `doc/user-guide/transfer-detection.md` (part of
  a new `doc/user-guide/spend-analysis.md`). Real local `ledgr.db`
  re-backfilled (`accounts.sort_code`/`account_number` corrected for all 4
  real accounts) and the spend ledger fully re-derived from a clean slate
  (`spend_entries`/`spend_entry_sources`/`transaction_links` cleared and
  rebuilt via `ledgr import`) — backed up first
  (`ledgr.db.bak-20260712124143`). Real impact: 85 previously-misclassified
  internal transfers (led by a recurring `SHARED BILLS ACCO` transfer of
  ~£3,415/month that had been inflating "spend" every month) now correctly
  excluded — monthly spend dropped by roughly £3.4k-£5.5k/month across the
  board (see corrected totals above). All 52 unit tests pass, including a
  new one covering the `ACCTID` split's fallback-to-no-sort-code case for
  an unexpected shape.
- **Follow-up session (2026-07-12): two more reference household accounts
  identified and registered.** During the leading/trailing `NAME` shape
  verification work (matching every distinct sort-code/account-number pair
  appearing in leading-shape transactions across the entire transaction
  history against known household accounts), two additional accounts not
  previously registered were identified: sort `208794`, account `33403394`
  → **"Shared Shopping Account"** (confirmed by the user as a real shared
  account, previously showing as an unresolved recurring monthly transfer of
  £250, incorrectly counted as spend); sort `208794`, account `03868915` →
  **"Joint Annual Expense"** (confirmed as a joint savings account for
  shared costs, previously appearing 23 times across the transaction history
  as an unidentified account, 2 of which had been incorrectly counted as
  spend). Both added to `~/.config/ledgr/config.toml`'s `household_accounts`
  list. Real local database backed up
  (`~/.local/share/ledgr/ledgr.db.bak-20260712125314`), specific
  misclassified `spend_entries` rows cleared, and `ledgr import` re-run to
  re-derive them correctly as internal transfers — `spend_entries` count
  dropped from 699 (after the earlier Romina-accounts fix) to 690 over these
  two fixes. `ledgr status`'s "Household Reference Accounts" table now lists
  4 accounts total: Romina Primary Account, Romina Secondary Account, Shared
  Shopping Account, Joint Annual Expense. All 52 unit tests pass throughout;
  `cargo build`/`cargo test` clean.

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
- TODO — per ADR 0007: add new `AccountType` variants for the house and
  pension (exact categorisation/naming TBD at implementation time); add
  a manual balance-snapshot entry path (new CLI command + `Db` method)
  for accounts with no automated feed (the house always; the pension
  whenever a report isn't being parsed); a pension statement import, if/
  when a format exists, reuses `ImportFileParser::balance_snapshot()`
  like any other institution — no new mechanism. Net worth: latest
  balance per account, summed with assets positive / liabilities
  negative by account type. **Mortgage deliberately excluded from this
  Task** — split out into its own Delta (see Delta: Mortgage Tracking
  below) once the user recognised it as a small domain of its own
  (interest rates, split/tranched parts of the mortgage, terms changing
  over time), not just another balance-snapshot account like the house
  or pension.

## Delta: Mortgage Tracking

Split out from Delta: The Gap, Task 4 (2026-07-12) once the user
recognised a mortgage isn't just another asset/liability balance
snapshot: it has its own small domain — one or more parts/tranches, each
with its own interest rate (fixed or tracker), a rate period with an end
date, and a repayment schedule — that changes discontinuously over time
(a fix ending, a product switch, overpayments). A single `balance_minor`
snapshot per account (as planned for the house/pension in Delta: The
Gap, Task 4) can't represent any of that. Scope, schema, and whether
this needs its own tables (e.g. a `mortgage_parts` or `mortgage_terms`
edge concept) not yet designed.

### Task 1: Design the mortgage domain model
- TODO — work out what a mortgage actually needs to record: how many
  parts/tranches it can have, what changes about each part over time
  (rate, rate type, rate-end date, term end, balance), and how that
  differs from a simple balance-snapshot account. Check
  `doc/domain/ubiquitous-language.md` and agree any new domain terms
  (e.g. "mortgage part"/"tranche"/"rate period") with the user before
  introducing them, per this project's ubiquitous-language rule. Decide
  whether this fits the existing `accounts` + `balance_snapshots`
  machinery (ADR 0007) with extension, or needs a genuinely new
  mechanism — an ADR either way, given the architectural weight.
- TODO — once designed, decide the data-entry route: the user's mortgage
  provider may or may not offer a statement/export to parse; likely
  manual entry (similar to the house) is the starting point regardless.

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

## Delta: Payslip Import

Future delta, raised 2026-07-12: the user wants to import his own full
payslip, not just the net salary `DIRECTDEP` that shows up on the bank
side — to see the full breakdown of where money comes from (gross pay,
tax, NI) and what's being paid into his pension straight from payroll
(never touches the bank account, so the current account/pension-account
approach can't see it at all). Feeds both the income ledger (Delta: The
Gap) and pension tracking (Other Transaction Import's still-TODO
pension/investment parser) — likely needs coordinating with both rather
than being fully independent.

### Task 1: Evaluate payslip format and scope
- TODO — decide the source format (PDF payslip export vs a payroll
  provider's own download/API, format TBD) and what fields matter
  (gross pay, tax, NI, pension contribution — employee and employer
  portions — net pay). Not yet started.

## Delta: Regular Payments

Future delta, raised 2026-07-12: let the user label up individual
recurring spend entries with a human name (e.g. "Jim's mobile
network"), so that recognising and auto-categorising them becomes
possible without waiting on general rule-based categorisation (Delta:
Spending Categorisation, Task 2) to be fully designed — a regular
payment is a narrower, higher-confidence case (same merchant, same-ish
amount, recurring cadence) than general categorisation rules need to
solve for. Two frequencies matter: **monthly** (most direct debits) and
**yearly** (e.g. annual subscriptions/insurance) — cadence itself is
probably useful signal for recognising a regular payment, not just for
display. Regular payments aren't only direct debits from the bank —
some come through the credit card (once Credit Card Transaction Import
lands) as recurring card charges, so this can't be scoped to `TRNTYPE`
alone; needs to work off the merchant/description pattern across both
sources. New domain term **Regular Payment** — not yet formally
recorded in `doc/domain/ubiquitous-language.md`, pending a design
session (relationship to **Category** and `rule_name` needs deciding:
is a regular payment its own concept that then implies a category, or
just a categorisation rule with a friendlier UI?).

### Task 1: Design regular payment recognition and labelling
- TODO — not yet started. Open questions: how a regular payment is
  matched (merchant/description pattern, cross-checked against
  recurrence) vs a general categorisation rule; where the label is
  stored (own table vs a `spend_entries` field); whether recognising
  the *cadence* itself (monthly vs yearly) is worth surfacing to the
  user (e.g. flagging when an expected monthly payment didn't occur);
  how this interacts with Spending Categorisation's rule-based engine
  once that exists — likely regular payments feed it rather than
  duplicate it.

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
4. Credit card CSV parser (Credit Card Transaction Import Task 1) — still
   the oldest open TODO, also unlocks derivation rules 8-10.

## Checkpoint: Session 2026-07-12b

**What was completed this session:**
- Delta: Statement/Import Naming Cleanup, both tasks — done in full.
  Term agreed with the user: **"Import"** (rejected: *export*,
  *download*, *import file*), despite already naming the `ledgr import`
  command — one **import** is one file, `ledgr import` processes a
  batch of them in one run (`ImportSummary`), confirmed coherent.
- Renamed throughout: `statements` table → `imports`
  (`src/db/schema.sql`), `statement_id` columns → `import_id`
  (`transactions`, `balance_snapshots`), `Transaction`/
  `NewTransaction.statement_id` → `import_id` (`src/model.rs`),
  `Db::insert_statement`/`find_statement_by_hash` →
  `insert_import`/`find_import_by_hash` (`src/db/statements.rs` →
  `src/db/imports.rs`), delta names (Bank/Credit Card/Other "Statement
  Import" → "Transaction Import" — "Transaction" chosen over plain
  "Import" for delta names since ledgr may import non-transaction data,
  e.g. balance-only pension reports).
- Mid-session follow-up (prompted by the user after the first pass):
  the parser trait was renamed `StatementParser` → `ImportParser`,
  then refined a second time to **`ImportFileParser`** — more explicit
  that it parses a single file, distinct from the batch-level
  `ImportSummary`/`import_inbox()` run. Applied consistently across
  `src/import/mod.rs`, `barclays_ofx.rs`, `generic_csv.rs`,
  `pipeline.rs`, and doc references.
- Generic English uses of "statement" (OFX statement response, "bank
  statement", "PDF statement export") were deliberately left alone —
  only the retired domain term and its code/schema identifiers were
  renamed. Confirmed no `Statement`/`NewStatement` struct ever existed
  (it was only a field/table name), correcting an inaccurate note in
  the delta's original Task 2 description.
- Real local `ledgr.db` migrated: new idempotent
  `Db::migrate_statements_to_imports` step in `Db::init`
  (`src/db/mod.rs`) renames the table and both FK columns on first open
  after upgrade (`ALTER TABLE ... RENAME TO` / `RENAME COLUMN`,
  supported by the bundled SQLite via `rusqlite` 0.31), no-op on a
  fresh or already-migrated database — same pattern as the earlier
  `AccountType::Checking` → `Current` migration. Verified against a
  scratch copy before touching the real file (table renamed correctly,
  `ledgr status` output unchanged, migration idempotent on a second
  run), then backed up (`~/.local/share/ledgr/ledgr.db.bak-20260712122251`)
  and applied for real — confirmed via `ledgr status` (4 real accounts,
  correct balances) and `sqlite3 .tables` (`imports` present,
  `statements` gone).
- Updated `doc/domain/ubiquitous-language.md` (Statement → Import entry
  with full provenance), ADRs 0002/0003/0007, `CLAUDE.md`'s trait
  snippet, `doc/kb/enable-banking-registration.md`, and
  `doc/implementation-notes/spend-ledger-design.md` for the renamed
  code identifiers.
- 51 unit tests still pass; `cargo clippy` clean (same pre-existing
  dead-code warnings as before, nothing new).
- One cleanup item deliberately left for the user: `src/db/statements.rs`
  is now dead (superseded by `src/db/imports.rs`, not declared as a
  module anywhere) but `rm` is blocked in this environment — the delete
  command is queued on the clipboard rather than run automatically.

**State of the project:**
The "statement" vs "import" naming inconsistency flagged in the
previous session is fully resolved across code, schema, the real
database, and docs — no more mismatch between the domain-language doc
and what the code actually says. Bank Transaction Import, Credit Card
Transaction Import, and Other Transaction Import are now the delta
names throughout the plan. Nothing else changed functionally this
session; all prior functionality (import, dedup, derivation, TUI)
is unaffected — this was a pure rename.

**Immediate next priorities:**
1. Delta: The Gap, Task 1 — minimal income ledger (`income_entries` +
   `income_entry_sources`).
2. Delta: The Gap, Task 2 — gap calculation (`ledgr gap` CLI).
3. Delta: The Gap, Task 4 — implement assets/liabilities as accounts
   per ADR 0007 (new `AccountType` variants, manual balance-snapshot
   entry command).
4. Credit card CSV parser (Credit Card Transaction Import Task 1) — still
   the oldest open TODO, also unlocks derivation rules 8-10.
5. Run the queued `rm src/db/statements.rs` (clipboard) to remove the
   now-dead old module file.

## Checkpoint: Session 2026-07-12e

**What was completed this session:**
- Developer documentation written and revised: `doc/developer-docs/ofx-format.md` (new, lightweight sketch of OFX file format — envelope, `BANKTRANLIST`, `STMTTRN` block structure and five fields) and `doc/developer-docs/transfer-detection.md` (new, technical notes on how ledgr detects internal transfers from Barclays OFX `NAME` fields). Transfer-detection doc went through several rounds of revision based on user feedback: real account sort codes and transaction descriptions initially mistakenly used as "anonymised" examples were caught by the user, verified against the real database and config, and replaced with fully fabricated examples matching real data shape/length. All Rust code snippets removed from transfer-detection.md at the user's request (refocused on data/behaviour, not implementation). Barclays' own published abbreviation reference (`FT` = "Funds Transfer") verified and sourced; OFX `NAME` field's 32-character cap confirmed as an OFX standard (`GenericNameType`, `maxLength="32"`) via XSD schema reference. Full `STMTTRN` block examples (all 5 fields) added for leading-shape, trailing-shape, and truncation cases. Refined theory (proposed by user, verified against real data): leading `NAME` shape (`<sort> <account> <reference> FT`) is used for one-off manual Faster Payments, while trailing shape (`<label> <sort> <account>`) is used for recurring standing orders/direct debits — shape doesn't directly determine this, reference/label *length* does (short reference leaves room for an `FT`/`STO` marker; long label consumes the 32-char budget and pushes both account number and marker out). Verified by cross-checking every distinct sort-code/account-number pair in leading-shape transactions against known accounts: zero false positives, all resolved to genuine accounts. Process lesson: when writing "anonymised" examples from real data, cross-check the final doc text against real values before considering done, not just intend to anonymise.
- User-facing documentation written: `doc/user-guide/spend-analysis.md` and `doc/user-guide/transfer-detection.md` (new, non-technical explanations of spend ledger and transfer detection concepts for a general reader, cross-linked to each other and to the developer-docs technical versions).
- Two more reference household accounts identified (during the leading-shape verification work above) and registered: sort `208794`, account `33403394` → "Shared Shopping Account" (confirmed real shared account, previously an unresolved recurring £250/month transfer, incorrectly counted as spend); sort `208794`, account `03868915` → "Joint Annual Expense" (confirmed joint savings account, appeared 23 times in transaction history as unidentified, 2 misclassified as spend). Both added to `~/.config/ledgr/config.toml`. Real database re-derived after clearing misclassified rows — `spend_entries` count dropped from 699 to 690 over this session's fixes. `ledgr status`'s "Household Reference Accounts" table now shows 4 accounts total.

**State of the project:**
Transfer-detection gaps are now fully closed: all four reference household accounts are registered (Romina Primary/Secondary, Shared Shopping, Joint Annual Expense), the real local database is clean and correctly re-derived (690 spend entries, 163 out of scope), and developer/user documentation is complete. The `spend_entries` data is now fully trustworthy for analysis. Delta: The Gap Task 2 (gap calculation / prototyping the monthly total-spend shape) is unblocked and can proceed on real, clean data.

**Immediate next priorities:**
1. Continue Delta: The Gap, Task 2 — prototype the monthly "total spend" shape via ad-hoc SQL and decide on the "closing the books" monthly record structure before committing to a CLI command/schema.
2. Delta: The Gap, Task 1 — minimal income ledger (`income_entries` + `income_entry_sources`).
3. Delta: The Gap, Task 4 — implement assets/liabilities as accounts per ADR 0007 (new `AccountType` variants, manual balance-snapshot entry command).
4. Credit card CSV parser (Credit Card Transaction Import Task 1) — still the oldest open TODO, also unlocks derivation rules 8-10.

## Checkpoint: Session 2026-07-12f

**What was completed this session:**
- Investigated a real transaction in June 2026 spend data
  (`MR JAMES BARRITT 49291328548900`) and identified it as the user's
  own Barclaycard credit card payment, misclassified as spend since the
  credit card was never a tracked account.
- Wrote `doc/kb/barclaycard/pdf-export-structure.md`: full structure of
  the Barclaycard PDF "Transactions" export, card-number (PAN)
  structure research (IIN/BIN, Luhn digit, reissue behaviour) prompted
  by the user sharing a real card number, and the recommended
  date+amount matching strategy (no real card number recorded in the
  doc or anywhere else in the repo).
- Built and validated `BarclaycardPdfParser` end-to-end against a real
  205-transaction Barclaycard PDF export (scratch-only inbox, in-memory
  database — real `ledgr.db` never touched, real PDF never committed):
  all 205 transactions imported correctly, including a real
  `pdf-extract` ordering quirk (date/type-tag reversed on some rows)
  found and fixed during validation.
- New schema/model: `account_card_numbers` table for credit-card
  number history (globally-unique `last4`, reassignable via
  `Db::link_card_number` for human-confirmed reissues), `CardIdentity`
  struct, `ImportFileParser::card_identity()` trait method,
  `Db::find_or_create_credit_card_account`, generic `notes` column on
  `transactions` (unpopulated so far). `AccountType::CreditCard` reused
  as-is — no new account type needed.
- 61 unit tests total (up from 52), all passing; `cargo clippy` clean
  (same pre-existing dead-code warnings as before).

**State of the project:**
Credit card import now has a working, validated parser producing real
transactions and a correct account/balance — the missing piece before
this was the biggest gap in "analyse monthly spending across current
account, credit card, and Amazon orders" (the plan's stated real-world
goal). What's not yet done: excluding credit card bill payments from
spend (they still show up as ordinary spend on both the bank and card
sides until transfer detection is extended to cover this pairing).

**Immediate next priorities:**
1. Build credit card ↔ bank transfer matching (Delta: Credit Card
   Transaction Import, Task 5) — likely date+amount matching, possibly
   combined with the card-number-prefix heuristic.
2. Resume Delta: The Gap, Task 2 (monthly "total spend" shape /
   "closing the books" prototyping) once credit card transfers are
   correctly excluded.
3. Delta: Credit Card Transaction Import, Task 3 (partner's credit
   card) and Task 4 (manual spend entries via proxy account) remain
   TODO, lower priority than Task 5.

## Checkpoint: Session 2026-07-12g

**What was completed this session:**
- Real Barclaycard PDF import executed end-to-end against the live database: created a new credit card account (renamed to "Barclaycard", card ending 0002) from `~/Downloads/Transactions - 2026-07-12T14_06_35.985Z.pdf`, 205 transactions imported covering 2026-01-02 to 2026-07-10, balance snapshot £613.73 confirmed matching the PDF's stated balance. Spend ledger re-derived: 170 new spend entries, 275 internal transfers detected (110 paired), 98 out of scope.
- Renamed the account via `ledgr name-account 0002 "Barclaycard"` once the Account column started showing the card's last4 separately, removing the now-redundant `(...0002)` suffix from the account name itself.
- Fixed two real schema-drift/migration bugs surfaced by this import, both fixed in code rather than by hand-patching the database alone:
  - `account_card_numbers.last4`'s `UNIQUE` constraint had drifted to per-account uniqueness on the real DB instead of `schema.sql`'s intended global uniqueness; recreated to match (table was empty, safe).
  - `transactions.notes` (added to `schema.sql` in an earlier session for this delta) had no migration path for existing databases — `CREATE TABLE IF NOT EXISTS` never alters an existing table. Added `Db::migrate_add_transactions_notes()` (`src/db/mod.rs`), idempotent and guarded to no-op both on fresh databases (table doesn't exist yet) and already-migrated ones.
  - Cleaned up several orphaned foreign-key rows in the real DB left by earlier manual `sqlite3` CLI deletes that ran without `PRAGMA foreign_keys=ON` (in `account_card_numbers`, `balance_snapshots`, and a pre-existing orphaned `imports` row referencing account id 5, deleted in an earlier session).
- `ledgr status`'s Account column previously showed `-` for the credit card (it has no bank sort code/account number, only a card-number history). Added `AccountStatus.card_last4` (`src/db/status.rs`), resolved once from `account_card_numbers` for any account with no bank account number, consumed by both the CLI (`src/main.rs`) and the TUI accounts screen (`src/ui.rs`/`src/app.rs`) — single source of truth instead of duplicated per-surface logic.
- Credit card balances are now shown negative (a liability, not an asset). The sign flip is baked into `AccountStatus.balance_minor` itself at the source (`src/db/status.rs`), not just at display time, so future net-worth summing (Delta: The Gap, Task 4) gets assets-positive/liabilities-negative for free per ADR 0007's stated convention, without each consumer needing to remember to apply it.
- Verified both fixes in the CLI (`ledgr status`) and the live TUI (driven via tmux) — both correctly show `Barclaycard  (0002)  -613.73 GBP`.
- Real `ledgr.db` backed up before any of today's database changes (`ledgr.db.bak-20260712174914-pre-cc-import`).
- All 61 unit tests pass; `cargo build` clean.
- Explored Gmail access for the Amazon Order Import delta's email-scanning route (doc-only, nothing built or configured) — recorded directly under Delta: Amazon Order Import, Task 1 in the plan body already (no action needed from you there): no Gmail/mail integration exists in this Claude Code environment today; two setup paths identified (Gmail MCP server vs. direct IMAP); recommended narrowing scope via a Gmail filter that forwards/labels Amazon order-confirmation emails only, rather than granting broad whole-inbox read access.

**State of the project:**
The credit card is now a real, live, imported account, completing the last piece of the plan's stated real-world goal (analyse monthly spending across current account, credit card, and Amazon orders — Amazon import itself still TODO). Spend and transfer detection are running against it, but Delta: Credit Card Transaction Import, Task 5 (matching card payments to bank-side transfers) is still open, so bank-to-card payment transactions likely still double-count in spend until that lands. `ledgr status` and the TUI accounts screen are now visually consistent with each other and with correct liability accounting semantics. Amazon Order Import's email-scanning route now has a recommended narrow-scope approach (Gmail filter, not full inbox access) but remains undecided and unbuilt.

**Immediate next priorities:**
1. Delta: Credit Card Transaction Import, Task 5 — match credit card payments to bank-side transfers (date+amount matching and/or card-number-prefix detection) so bill payments stop leaking into spend.
2. Delta: The Gap, Task 2 — monthly "total spend" shape/command.
3. Delta: Amazon Order Import, Task 1 — decide email-scanning vs. manual export, now informed by the Gmail-filter narrow-scope recommendation.

## Implementation Notes

- Single crate `ledgr` (binary also named `ledgr`) — domain model,
  SQLite schema/migrations, transaction import, and analysis sit
  alongside the TUI as modules under `src/` (`db`, `import`, `model`,
  `analysis`, `app`, `ui`, `main`). Previously a two-crate workspace;
  merged per `doc/adr/0003-single-crate-package-ledgr.md` so
  `cargo install ledgr` works via crates.io without a second published
  crate.
- Storage: SQLite via bundled `rusqlite`. Non-tabular relationships
  (transfers, category hierarchies, refund/reversal links) are modelled
  as edge tables in `src/db/schema.sql` rather than a graph database.
- New import formats are added by implementing `ImportFileParser`
  in `src/import` (see `generic_csv.rs` for the existing example).
- Project is an early scaffold — not yet functional end-to-end.
