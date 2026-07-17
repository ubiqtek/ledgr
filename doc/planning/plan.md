# ledgr — Plan

## What's Next

- **Next:** Task 1 — Minimal income ledger (Delta: The Gap)
- **Sub-doc:** none
- **Blockers:** None
- **Context:** [Checkpoint: Session 2026-07-17b](#checkpoint-session-2026-07-17b)

## Summary

| Delta | Task | Status |
|-------|------|--------|
| [Delta: Decide on Switching to PDF for Transaction Import](#delta-decide-on-switching-to-pdf-for-transaction-import) | [1. Decide whether to build a BarclaysStatementPdfParser](#task-1-decide-whether-to-build-a-barclaysstatementpdfparser) | TODO |
| [Delta: Automatic Inbox Import](#delta-automatic-inbox-import) | [1. Inbox change notification](#task-1-inbox-change-notification) | TODO |
| [Delta: Credit Card Transaction Import](#delta-credit-card-transaction-import) | [1. Credit card statement parser](#task-1-credit-card-statement-parser) | ✓ DONE |
| | [2. Evaluate Barclaycard PDF export](#task-2-evaluate-barclaycard-pdf-export) | ✓ DONE |
| | [3. Import the user's partner's credit card](#task-3-import-the-users-partners-credit-card) | TODO |
| | [4. Manual spend entries via a proxy account](#task-4-manual-spend-entries-via-a-proxy-account) | TODO |
| | [5. Match credit card payments to bank-side transfers](#task-5-match-credit-card-payments-to-bank-side-transfers) | ✓ DONE |
| [Delta: Amazon Order Import](#delta-amazon-order-import) | [1. Evaluate automation route — email scanning vs manual export](#task-1-evaluate-automation-route--email-scanning-vs-manual-export) | TODO |
| | [2. Amazon order import](#task-2-amazon-order-import) | TODO |
| [Delta: Review and Re-classification TUI](#delta-review-and-re-classification-tui) | [1. Review queue screen](#task-1-review-queue-screen) | TODO — deprioritised below Delta: The Gap |
| [Delta: Reconciliation](#delta-reconciliation) | [1. Design account-level and household-level reconciliation checks](#task-1-design-account-level-and-household-level-reconciliation-checks) | TODO |
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
| | [3. Monthly Gap screen and spend drill-down](#task-3-monthly-gap-screen-and-spend-drill-down) | IN PROGRESS |
| | [4. Leader-key navigation](#task-4-leader-key-navigation) | ✓ DONE — uncommitted, pending review |
| [Delta: Packaging & Distribution](#delta-packaging--distribution) | [1. Publish `ledgr` to crates.io](#task-1-publish-ledgr-to-cratesio) | ✓ DONE |
| | [2. Web frontend](#task-2-web-frontend) | TODO |
| [Delta: Live Open Banking (Enable Banking)](#delta-live-open-banking-enable-banking) | [1. Evaluate feasibility & security model](#task-1-evaluate-feasibility--security-model) | IN PROGRESS |
| [Delta: Double-Entry Accounting](#delta-double-entry-accounting) | [1. Evaluate a double-entry model for ledgr](#task-1-evaluate-a-double-entry-model-for-ledgr) | TODO |
| [Delta: Payslip Import](#delta-payslip-import) | [1. Evaluate payslip format and scope](#task-1-evaluate-payslip-format-and-scope) | TODO |
| [Delta: Reclaimable Work Expenses](#delta-reclaimable-work-expenses) | [1. Design the reclaimable expenses ledger and marking flow](#task-1-design-the-reclaimable-expenses-ledger-and-marking-flow) | TODO |
| [Delta: Regular Payments](#delta-regular-payments) | [1. Design regular payment recognition and labelling](#task-1-design-regular-payment-recognition-and-labelling) | TODO |

Archived Deltas: see the [archive index](archive/index.md)

Real-world goal driving Delta: Credit Card Transaction Import, Delta:
Amazon Order Import, Delta: Spend Ledger, and Delta: The Gap: analyse
monthly spending across current account, credit card, and Amazon orders.

## Delta: Decide on Switching to PDF for Transaction Import

**Split out 2026-07-17** from Bank Transaction Import, Task 1, where the
underlying research already happened. Full findings, real-data evidence,
and de-dup analysis: `doc/implementation-notes/optimising-import-data.md`.

Headline: the Barclays current-account statement **PDF**
(`Transaction.pdf`, distinct from the already-built `BarclaycardPdfParser`)
keeps a transfer's sort code/account number on its own line, separate
from the truncated label — so account numbers never get cut the way
OFX's `NAME` field cuts them (the root cause of several transfer-pairing
workarounds in the archived Delta: Transfer Ledger, e.g. `SHARED BILLS
ACCO`). `data.csv` and `data.qbo` were both ruled out already (CSV
shares OFX's truncation failure mode and has no de-dup key at all; QBO
is the same OFX 1.02 payload under a different extension).

### Task 1: Decide whether to build a BarclaysStatementPdfParser
- TODO — open questions from the sub-doc: de-dup without `FITID`, using
  running balance as part of the de-dup key, primary-format-switch vs
  backfill-only.
- Sub-doc: `doc/implementation-notes/optimising-import-data.md`

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
- ✓ DONE — both ideas from the earlier discussion built together:
  1. **Pattern gate** (`looks_like_card_payment_reference` in
     `src/derive.rs`, new `Classification::CardPayment`): recognises a
     bank-side `NAME` shaped as `<cardholder name words> <digit run>` and
     validates the digit run against a real card-network IIN/BIN table
     (`known_card_network_prefix` — Visa `4`, Mastercard `51xx-55xx` and
     `2221-2720`), not just "looks numeric". Deliberately no lower
     digit-count bound (Barclays' truncation length varies with how much
     of the 32-char `NAME` field the preceding name text uses) but an
     upper bound of `MAX_PAN_DIGITS = 16` (a full untruncated PAN can't
     be longer than that, whatever its prefix looks like). This rule
     sits between the existing sort-code/account-number transfer rules
     and the CPM/FT suffix rules in `classify()`'s precedence order.
  2. **Date + exact amount matching** (`Db::find_card_payment_counterpart`,
     `src/db/spend.rs`): a transaction passing the pattern gate is only
     actually excluded from spend once matched to a same/opposite-amount
     transaction on a `CreditCard`-type account within a ±3 day window
     (mirrors `find_transfer_counterpart`'s existing window) — the
     pattern alone is corroborating evidence, not a reliable key on its
     own (truncated, and not reissue-stable), consistent with the KB
     article's recommendation. Unmatched candidates (e.g. the card
     statement for that period not yet imported) still become a spend
     entry, at reduced confidence (`rule_name = "card_payment_unmatched"`,
     0.5) rather than being silently dropped — same "stay visible for
     review" philosophy as the existing `"fallback"` rule.
  - New `DerivationSummary.card_payments_matched` field; 7 new unit
    tests covering the pattern gate (real card-shaped match, inbound
    money excluded, a short single-digit false positive, an unrelated
    long non-card reference number, a digit run longer than a full PAN)
    and the full derive path (matched pair excluded from spend; no
    credit card account yet → unmatched low-confidence spend). 68 tests
    total, all passing; `cargo clippy` clean.
  - **Validated against real data** (2026-07-12 session): found 32 real
    `"MR JAMES BARRITT <truncated PAN>"` transactions, all previously
    misclassified as `"fallback"` spend (confidence 0.4) — a real
    instance of the double-counting this task exists to fix. Also found
    and correctly excluded false positives during design: `"CORNWALL
    WILDLIFE 6060150000007"` (13-digit charity reference, wrong network
    prefix) and DVLA reference numbers (18 digits, exceeds
    `MAX_PAN_DIGITS`, and start with a non-card-network digit anyway).
    Real `ledgr.db` backed up first
    (`ledgr.db.bak-20260712221100-pre-card-payment-matching`); cleared
    the 32 misclassified `spend_entries`/`spend_entry_sources` rows and
    re-ran `ledgr import` — all 32 matched their exact-amount
    `"PAYMENT, THANK YOU"` counterpart on the Barclaycard account and
    were correctly excluded as internal transfers (`spend_entries`
    860 → 828, zero fell through to the unmatched low-confidence path).
    Confirmed idempotent — running `ledgr import` again created 0 new
    spend entries and 0 duplicate `transaction_links`.
  - **Not yet addressed:** the "only one registered Barclaycard" case —
    `find_card_payment_counterpart` matches against *any* `CreditCard`
    account, so once Task 3 (partner's card) adds a second one, a
    same-day same-amount coincidence across two cards could match the
    wrong counterpart. Not a problem today (only one card exists); flag
    for revisiting when Task 3 lands.

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

## Delta: Review and Re-classification TUI

**Split out 2026-07-17** from Spend Ledger, Task 3, once Spend Ledger's
design/schema/derivation work (Tasks 1-2) was fully done.

### Task 1: Review queue screen
- TODO — **deprioritised below Delta: The Gap** (the user wants total
  spend/income/gap working before investing in a categorisation UI).
  Review queue screen for low-confidence/uncategorised spend entries;
  single-key actions to mark internal transfer / not-spend, set
  category, edit note; manual actions stamp `classified_by='manual'`.

## Delta: Reconciliation

**Added 2026-07-13**, alongside Delta: Transfer Ledger — same underlying
motivation (trust, but verify, that the derivation layer is accounting
for everything correctly), different angle. The user's framing: now that
real balance anchors (`balance_snapshots`) and full transaction history
exist for every tracked account, `ledgr` should be able to *prove* the
books balance — take the opening balance at the start of a period, net
every transaction in that period, and arrive exactly at the closing
balance. If it doesn't, money has either appeared from nowhere (a
missing anchor, a mis-imported balance) or gone missing (a duplicate
filtered too aggressively, a file never actually imported, a gap in
date coverage). This is a general integrity check, independent of
spend/income/transfer classification — it validates the *raw
transaction* layer underneath all of it, and would likely have caught
issues faster than the ad hoc real-data debugging this project has
relied on so far (see the many "found a real bug via `ledgr status`/
manual SQL" entries throughout this plan's history).

### Task 1: Design account-level and household-level reconciliation checks
- TODO — not yet designed. Two levels worth distinguishing:
  1. **Per-account**: opening balance (from a `balance_snapshots` anchor,
     or the nearest one before the period) + net of every transaction in
     the period should equal the closing balance (the next anchor, or
     today's reported balance) — `Db::balance_as_of` (`db/balances.rs`)
     already does most of this arithmetic; reconciliation is really
     about *reporting a discrepancy* rather than new computation, and
     would primarily catch import gaps/duplicates/mis-mapped balance
     snapshots.
  2. **Household-level**: does spend + income + transfers-in/out net to
     the actual combined balance movement across all tracked accounts?
     This is a classification-*coverage* check, not a balance-arithmetic
     check — it would have caught the Delta: Transfer Ledger pairing gap
     faster than manual SQL did, and will matter more once Delta: The
     Gap's income ledger exists.
  Needs deciding: CLI command (`ledgr reconcile`?) vs a TUI screen vs
  both; whether "Reconciliation" needs its own entry in
  `doc/domain/ubiquitous-language.md` before building (check first, per
  the project's usual process). Not blocked on Delta: Transfer Ledger or
  Delta: The Gap landing first, but likely more useful once they have.

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

### Task 3: Monthly Gap screen and spend drill-down
- ✓ DONE (2026-07-12 session) — `Screen::MonthlyGap` (one row per month,
  newest first, Spend column only — Income/Gap columns wait on Delta: The
  Gap Task 1) and a per-month drill-down (`Screen::MonthSpend`, flat
  transaction list not merchant-grouped, per the user — auditing take
  priority over aggregation) with sticky column headers and an Account
  column (resolved through `app.accounts`, not a raw DB join, so it
  reuses the same user-overridden names as everywhere else). Reached via
  `m` from Accounts today — **will move to `<space>g` under Task 4's
  leader-key scheme**, see below.
- ✓ DONE — real transfer-detection bug found *while using this screen*:
  household members with no account digits in `NAME` (e.g. `"SCARAMAGLI R
  AMAZON OASIS FT"`) were misclassified as spend — see `derive.rs`'s rule
  1c and the **Reference Household Account** entry in
  `doc/domain/ubiquitous-language.md`.
- ✓ DONE — spend entry notes: `spend_entries.note` (always existed in the
  schema, never wired up) now settable via `ledgr note <id> "<text>"`
  (CLI) or `n` on the drill-down (TUI popup editor) — prompted by an
  unidentifiable real merchant (`"MARTS MEHAZEPE"`) the user wanted to
  record having checked and accepted as legitimate.
- Full session detail: see the dated entries under "What's Next" history
  above (2026-07-12).
- **Renamed 2026-07-17** — `Screen::MonthlyGap`/`Screen::MonthSpend` →
  `Screen::MonthlySpend`/`Screen::SpendMonth` (`src/app.rs`, `src/ui.rs`,
  `src/main.rs`), since "gap" (spend minus income) doesn't exist as a
  concept yet — only spend does, income being deferred to Delta: The Gap
  Task 1 — so the old name overclaimed what the screen actually shows.
  Naming pattern matches `Screen::MonthlyTransfers`/`Screen::TransferMonth`
  exactly (`Monthly<Noun>` top level, `<Noun>Month` drill-down) rather
  than introducing a third pattern. Leader-key binding moved `<space>g` →
  `<space>s` to match (`main.rs`'s leader-key match arm); the drill-down's
  own title text (`"Spend — {month}"`) was already distinct and needed no
  change. Verified live via `tmux` against the real database: `<space>s`
  opens "Monthly Spend", the drill-down shows "Spend — 2026-07", and the
  help screen shows the new binding. The "Gap" concept itself (spend vs
  income) is deferred to a future task, to be added back as a distinct
  screen/column once Delta: The Gap's income ledger exists, rather than
  bolted onto this one prematurely.

### Task 4: Leader-key navigation
- ✓ DONE (2026-07-12 session) — navigation-history stack + leader key:
  `app.rs`'s `previous_screen` field replaced with `nav_stack:
  Vec<Screen>` + `App::navigate_to(screen)` (pushes current screen,
  no-op if already on target) + `back()` (pops the stack, falls back to
  `Screen::Accounts` when empty). Every direct `self.screen = X`
  assignment converted to `navigate_to(X)`. `main.rs` gained a
  `pending_leader: bool` local (same pattern as `pending_g`): `<space>`
  then `a`/`g`/`t` jumps to Accounts/Monthly Gap/Monthly Transfers (see
  Delta: Transfer Ledger) from anywhere; the old `Accounts`-only `m`
  binding removed. `q`/`Esc` simplified to "quit if on Accounts, else
  `back()`". `cargo build`/`test`/`clippy` clean (71 tests passing).
  **Not yet committed to git** — sitting in the working tree for the
  user's review.
  - The Monthly Transfers screen itself (originally designed as part of
    this same task) moved to its own **Delta: Transfer Ledger** on
    2026-07-13, once it became clear its "no new persisted schema"
    design needed reopening — see that delta for the full history.

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

## Delta: Payslip Import

Future delta, raised 2026-07-12: the user wants to import his own full
payslip, not just the net salary `DIRECTDEP` that shows up on the bank
side — to see the full breakdown of where money comes from (gross pay,
tax, NI) and what's being paid into his pension straight from payroll
(never touches the bank account, so the current account/pension-account
approach can't see it at all). Feeds both the income ledger (Delta: The
Gap) and pension tracking (Other Transaction Import's still-TODO
pension/investment parser) — likely needs coordinating with both rather
than being fully independent. Also a hard dependency for **Delta:
Reclaimable Work Expenses** closing the loop on the user's own reclaimed
expenses, which are paid back via net pay rather than a separate bank
transaction.

### Task 1: Evaluate payslip format and scope
- TODO — decide the source format (PDF payslip export vs a payroll
  provider's own download/API, format TBD) and what fields matter
  (gross pay, tax, NI, pension contribution — employee and employer
  portions — net pay). Not yet started.

## Delta: Reclaimable Work Expenses

Future delta, raised 2026-07-13: some day-to-day spend is money a
household member (the user or Romina) pays out of pocket but can claim
back from their employer — currently invisible as a category, no
different from ordinary discretionary spend once it lands in the spend
ledger. Sketch, not yet designed:
- **Marking**: a keypress on the spend drill-down (`Screen::MonthSpend`,
  alongside the existing `n` note-editor binding) flags a spend entry as
  reclaimable — candidate key `w` (work) or `r` (reclaimable), TBD.
  Needs an explicit household-member assignment at mark-time (self vs
  Romina) since it isn't always inferable from the account alone (e.g.
  a shared card).
- **A new "reclaimable expenses ledger"**: a persisted table recording
  which spend entry, which household member, and whether/when it's been
  paid back — closer in spirit to `spend_entries.note`/
  `classified_by='manual'` (a manual annotation) than a derived ledger
  like Transfer Ledger, since the triggering event is a user keypress,
  not automatic classification.
- **Paid-back tracking is asymmetric between household members**:
  Romina's reclaims are likely paid back as an identifiable separate
  bank transaction; the user's own are folded into his net pay with no
  separately identifiable transaction — this is the direct reason
  **Delta: Payslip Import** matters here, since only a fully parsed
  payslip could surface a reclaimed-expenses line and let this close
  the loop automatically for his own claims.
- **A report**: some summary view (CLI or TUI, format TBD) of
  reclaimable expenses outstanding vs paid, by household member.

### Task 1: Design the reclaimable expenses ledger and marking flow
- TODO — not yet designed. Needs: (1) agreed domain term(s) — consult
  `doc/domain/ubiquitous-language.md` before naming anything (this
  delta's own title is a working name, not an agreed term); (2) schema
  for the new ledger (spend entry, household member, paid status, paid
  date/reference); (3) the TUI keypress + household-member picker on
  the spend drill-down; (4) the report view; (5) scoping what's
  buildable before Delta: Payslip Import lands vs blocked on it, given
  the asymmetric paid-back tracking above.

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

## Checkpoint: Session 2026-07-12h

**What was completed this session:**
- Confirmed the real Barclaycard PDF import (Delta: Credit Card Transaction Import, Task 1) has been run against the live `ledgr.db`, not just the scratch-inbox validation from earlier in the day — verified via `ledgr status`: Barclaycard account (`0002`), 205 transactions, £613.73 balance, matching the earlier scratch validation exactly.

**State of the project:**
Credit Card Transaction Import Task 1 (parser) is now fully done end-to-end against real data. Task 2 (PDF vs CSV evaluation) was already done. Tasks 3-5 (partner's card, proxy account for manual spend, matching card payments to bank-side transfers) remain TODO.

**Immediate next priorities:**
1. Delta: Credit Card Transaction Import, Task 5 — match credit card payments to bank-side transfers (date+amount matching and/or card-number-prefix detection) so bill payments stop leaking into spend.
2. Delta: The Gap, Task 2 — monthly "total spend" shape/command.

## Checkpoint: Session 2026-07-12i

**Completed:**
- Delta: Credit Card Transaction Import, Task 5 — built and validated card-payment-to-bank-transfer matching, combining both previously-discussed ideas rather than choosing one: a pattern gate (`looks_like_card_payment_reference` in `src/derive.rs`, new `Classification::CardPayment`) recognising a truncated-PAN `NAME` shape and validating it against a real Visa/Mastercard IIN/BIN table (`known_card_network_prefix`), then a date+exact-amount match (`Db::find_card_payment_counterpart`, `src/db/spend.rs`, ±3 day window, mirrors the existing transfer-pairing query) against any `CreditCard`-type account before actually excluding the transaction from spend. Unmatched candidates still become a low-confidence spend entry (`rule_name = "card_payment_unmatched"`) rather than being silently dropped, matching the existing `"fallback"` rule's philosophy.
- Iterated the pattern gate through several rounds of scrutiny before trusting it against real data: dropped an early exact-length-14 requirement (too rigid — truncation length varies with how much of Barclays' `NAME` field the preceding name text uses), dropped a follow-up minimum-length floor (arbitrary), and settled on relying on `known_card_network_prefix`'s own 4-digit floor for the short end plus a `MAX_PAN_DIGITS = 16` upper bound (a full untruncated PAN can't be longer than that, whatever its prefix looks like) — no lower bound needed. New unit tests cover the false-positive cases found along the way: a bare short digit, an unrelated long reference number with a non-card-network prefix (a real example — see below), and a digit run longer than a full PAN.
- **Validated against real data:** found 32 real `"MR JAMES BARRITT <truncated PAN>"` transactions, all previously misclassified as `"fallback"` spend (confidence 0.4) — confirming this was a genuine double-counting bug, not a hypothetical. Also found and correctly excluded two real false-positive shapes while designing the pattern gate: `"CORNWALL WILDLIFE 6060150000007"` (13-digit charity reference number, wrong network prefix) and DVLA vehicle-tax reference numbers (18 digits, exceeds `MAX_PAN_DIGITS`). Real `ledgr.db` backed up first (`ledgr.db.bak-20260712221100-pre-card-payment-matching`); cleared the 32 misclassified `spend_entries`/`spend_entry_sources` rows and re-ran `ledgr import` — all 32 matched their exact-amount `"PAYMENT, THANK YOU"` counterpart on the Barclaycard account and were correctly excluded as internal transfers (`spend_entries` 860 → 828). Confirmed idempotent: re-running `ledgr import` created 0 new spend entries and 0 duplicate `transaction_links`.
- 68 unit tests total (up from 65), all passing; `cargo clippy` clean (same pre-existing dead-code warnings as before, nothing new).

**State of the project:** Credit Card Transaction Import is now done except Tasks 3 (partner's card) and 4 (proxy account for manual spend) — both deferred, not blocking. Every real transaction that goes into computing total spend is now correctly classified: internal transfers (bank-to-bank and, as of this session, bank-to-credit-card) are excluded, and the previously-flagged blocker on Delta: The Gap, Task 2 is resolved. One known gap, not a problem today: `find_card_payment_counterpart` matches against *any* `CreditCard` account, so once Task 3 adds a second registered card, a same-day same-amount coincidence across two cards could match the wrong counterpart — flagged in the plan body for revisiting then.

**Immediate next priorities:**
1. Delta: The Gap, Task 2 — monthly "total spend" shape/command, now unblocked.
2. Delta: The Gap, Task 1 — minimal income ledger, needed before a real Gap (income − spend) figure is possible.

## Checkpoint: Session 2026-07-12j

**Completed:**
- Built all three parts of the previously-designed leader-key
  navigation + Monthly Transfers screen work: (1) `nav_stack`-based
  navigation history + `<space>` leader key (`a`/`g`/`t`), (2) the
  Monthly Transfers top-level screen with `derive::find_internal_transfers`
  (read-only preview pass, no new schema), (3) the per-month drill-down
  with counterpart-name resolution (`resolve_transferred_to`).
- Built via three sequential subagents (one per part, controller
  reviewing the diff after each before dispatching the next) to manage
  context on a task too large for one session.
- `cargo build`/`cargo test`/`cargo clippy` clean throughout (71 → 73 →
  76 tests passing across the three parts, no new clippy warnings).
  Deliberately not run against the real TUI/database — verified by
  tests and code reading only.

**State of the project:** All three TUI Analysis Views Task 4 items are
functionally complete and uncommitted in the working tree, awaiting the
user's review before committing. This closes out the last actively
in-progress TUI Analysis Views task.

**Immediate next priorities:**
1. Review and commit the uncommitted `app.rs`/`derive.rs`/`main.rs`/
   `model.rs`/`ui.rs` changes.
2. Manually sanity-check the new screens against the real TUI/database
   (deliberately not done yet — all three subagents were kept off real
   data).
3. Delta: The Gap, Task 1 (Minimal income ledger) — the next actionable
   undesigned TODO, since Task 2 depends on it.
4. Merchant-name normalisation — still deferred/undesigned.

## Checkpoint: Session 2026-07-13

**Completed:**
- Fixed a real bug in the new Monthly Transfers drill-down: counterpart
  name resolution (`resolve_counterparty`, `app.rs`) didn't handle
  Barclays' truncated account numbers the same way the classification
  logic already did, so a real transfer ("SHARED BILLS ACCO") showed
  raw digits instead of "Bills Account". Fixed to match consistently.
- Investigated the user's follow-up question ("is there a transfer
  ledger table with real SQL relations?") — confirmed no, by design
  (the Monthly Transfers screen deliberately re-derives on demand, no
  new schema). But found a genuine, separate gap while investigating:
  `transaction_links` (the existing edge table that *does* record
  transfer pairings during real import) has zero coverage for the real
  SHARED BILLS ACCO standing-order pairs, because
  `Db::find_transfer_counterpart`'s matching heuristic only handles
  manual transfers (both sides cross-reference each other's account
  number), not automated ones (STO/DD) like this one. Confirmed via
  direct read-only SQL against the real database.

**State of the project:** Spend ledger correctness is unaffected (these
transactions are still correctly excluded from spend either way) — this
is a missing-audit-trail gap, not a wrong-numbers bug. The new "show both
legs" `i` popup will under-report for automated transfers until this is
fixed.

**Immediate next priorities:**
1. Review and commit the still-uncommitted leader-key nav / Monthly
   Transfers working tree changes.
2. Decide whether to fix `find_transfer_counterpart`'s automated-transfer
   matching gap now or defer it.
3. Delta: The Gap, Task 1 (Minimal income ledger) — the next actionable
   undesigned TODO.

## Checkpoint: Session 2026-07-13b

**Completed:**
- The user pushed back on the "no persisted transfer schema" design
  decision from the checkpoint above — correctly pointed out it was
  never actually agreed, and no ADR recorded it (checked `doc/adr/`:
  confirmed nothing does). Rather than patch `find_transfer_counterpart`
  in place, decided to reopen the design properly.
- Restructured the plan: moved the Monthly Transfers screen work (and
  both 2026-07-13 bug/gap notes) out of TUI Analysis Views Task 4 into
  a new **Delta: Transfer Ledger**, with new Task 2 (design a real
  persisted ledger table + write the missing ADR) and Task 3 (persist +
  migrate the screen to query it) as the path forward. TUI Analysis
  Views Task 4 trimmed to just the leader-key/nav-stack work, which is
  generic and unaffected.
- Added a new **Delta: Reconciliation** — the user's idea: with real
  balance anchors (`balance_snapshots`) and full transaction history now
  in place, `ledgr` should be able to prove opening balance + net
  transactions in a period = closing balance, per account and
  household-wide, as a general integrity check independent of
  spend/transfer/income classification. Not designed yet — one TODO
  task recorded (Task 1), distinguishing per-account (balance
  arithmetic, catches import gaps/duplicates) from household-level
  (classification coverage, would have caught the transfer-pairing gap
  faster than manual SQL did).
- Established a explicit design principle for both new deltas and
  likely the future income ledger: **persist ledger relationships at
  import time, let the UI only query — don't re-derive relations live
  on every screen open.** To be written up formally as an ADR when
  Delta: Transfer Ledger Task 2 happens, not before.

**State of the project:** No code changed this checkpoint — plan-only
restructuring. The working tree still has the same uncommitted
leader-key nav / Monthly Transfers v1 changes as before.

**Immediate next priorities:**
1. Delta: Transfer Ledger, Task 2 — design the persisted schema (check
   `doc/domain/ubiquitous-language.md` for naming first) and write the
   ADR. Do this step by step, not in one large session.
2. Delta: Reconciliation, Task 1 — design account-level/household-level
   checks, likely easier once the transfer ledger exists.
3. Review and commit the still-uncommitted working tree changes.
4. Delta: The Gap, Task 1 (Minimal income ledger) — next after the two
   new deltas.

## Checkpoint: Session 2026-07-13c

**Completed:**
- Delta: Transfer Ledger, Task 2 (design + ADR) and Task 3 (build + real
  migration) — both done in full, closing out the delta reopened in the
  previous two checkpoints.
- Design written to `doc/implementation-notes/transfer-ledger-design.md`:
  new `transfer_entries` table (one row per internal-transfer-classified
  transaction, mirroring `spend_entries`' provenance idiom), pairing
  modelled as extra columns on the same row rather than a second edge
  table (a transfer has at most one counterpart), and a three-tier
  pairing algorithm (description match, mutual amount-date match,
  self-reference match — the third tier added mid-build once a real gap
  demanded it, see below).
- ADR written: `doc/adr/0009-persisted-ledgers-built-at-import.md`,
  formalising the principle the user asked for: every derived relation
  is built once at import into a persisted table, the UI only ever
  queries it. Income ledger (Delta: The Gap, Task 1) flagged as the next
  expected application; Delta: Reconciliation flagged as a direct
  beneficiary.
- Built: `transfer_entries` schema + indexes, three-tier pairing in
  `src/derive.rs`, new `Db` methods in `src/db/spend.rs`/`src/db/mod.rs`,
  `TransferPairMethod`/`NewTransferEntry` in `src/model.rs`, `src/app.rs`
  migrated to query the persisted table (`src/ui.rs` needed no changes
  at all — confirming the design doc's prediction).
- Two real bugs found and fixed along the way: (1)
  `parse_trailing_account_suffix` was rejecting a trailing marker word
  (`"STO"`) after the account number; (2) tier 2's mutual-match
  requirement structurally could never pair the SHARED BILLS ACCO ↔
  Bills Account legs, because the Bills Account's own `NAME` field
  self-references its own account number rather than the sender's. The
  user's decision on the second bug: add a third, explicitly-tracked
  `self_reference_match` pairing tier (confidence 0.6) rather than loosen
  tier 2's safety property, for future auditability. Also fixed a
  derivation bug where already-persisted-but-unpaired legs from an
  earlier run could never be reconsidered once a new tier was added
  later — pairing now iterates all currently-unpaired persisted rows,
  not just the current run's freshly-classified candidates.
- Tests: 80 total (up from 76), all passing; `cargo build`/`cargo clippy
  --all-targets` clean against the pre-existing warning baseline.
- Real `ledgr.db` migrated (backed up twice first): backfilled
  `transfer_entries` for all 300 already-imported internal transfers —
  218 description-match, 21 self-reference-match (including all 7 target
  SHARED BILLS ACCO pairs), 0 amount-date-match, 40 permanently unpaired
  (confirmed Reference Household Accounts, not a gap). Confirmed
  idempotent on a second `ledgr import` run. Cleaned up 110 now-redundant
  `transaction_links` transfer rows, deliberately kept the 32 rows still
  actively written by the separate card-payment-matching mechanism.

**State of the project:** Delta: Transfer Ledger is functionally
complete — the Monthly Transfers screen and its drill-down/popup now
read from a real persisted, provenance-tracked table instead of
re-deriving live, and the audit-trail gap that reopened this delta (zero
`transaction_links` coverage for automated transfers) is fully closed on
real data. Not yet done: a manual TUI click-through (only verified via
tests/build/clippy and real-DB SQL so far), and getting the user's
sign-off to formalise "Transfer Entry"/"Transfer Ledger" in
`doc/domain/ubiquitous-language.md`. Everything from this session is
uncommitted, sitting alongside the already-uncommitted leader-key nav /
Monthly Transfers v1 changes from prior sessions.

**Immediate next priorities:**
1. Review and commit the uncommitted working tree — now spanning the
   leader-key nav / Monthly Transfers v1 work plus this session's full
   Transfer Ledger Task 2/3 build — before starting anything new.
2. Get the user's sign-off on "Transfer Entry"/"Transfer Ledger" as
   agreed terms in `doc/domain/ubiquitous-language.md`.
3. Delta: Reconciliation, Task 1 — design account-level/household-level
   checks, now genuinely easier with a persisted transfer ledger to
   reconcile against.
4. Delta: The Gap, Task 1 (Minimal income ledger) — next delta expected
   to apply the same "persisted ledger, built at import" principle
   (ADR 0009).

## Checkpoint: Session 2026-07-13d

**Completed:**
- Two display bugs found by the user actually looking at the live TUI
  (neither caught by tests/build/clippy/SQL checks, since both were
  purely about rendering, not the persisted data): the counterparty
  column resolving to a self-referencing leg's own account instead of
  its real pair, and the drill-down showing two rows per paired transfer
  when the user expected one. Both initially patched at the
  query/display layer.
- **The real fix, once the user rejected the patch as papering over a
  wrong data model**: `transfer_entries` was redesigned from one row
  per leg (two rows per paired transfer, linked by
  `counterpart_transaction_id`) to **one row per real-world transfer**
  (`out_*`/`in_*` columns naming both legs directly, either side
  nullable until found). Full schema, model, `Db` layer, `derive.rs`
  pairing logic (now needs a genuine second-stage "re-pairing sweep"
  comparing open rows against each other, not just against fresh
  transactions), and `app.rs`/`ui.rs` all reworked accordingly. See
  "Delta: Transfer Ledger, Task 3" above for the full technical detail,
  and `doc/implementation-notes/transfer-ledger-history.md` for the
  complete reasoning trail and the lesson learned (a display-layer patch
  had been accepted as "fixed" without checking whether the underlying
  model actually matched the domain concept).
- Real database re-migrated a second time (170 merged rows from the
  prior 300 per-leg rows; 130 fully paired, 40 correctly one-sided; all
  7 SHARED BILLS ACCO pairs confirmed intact) — backed up fresh first,
  verified structurally against a scratch copy before touching the real
  file.
- Corrected an overclaim from earlier this session: "Transfer Ledger"/
  "Transfer Entry" had been marked `established` in
  `doc/domain/ubiquitous-language.md` and attributed to "the user" —
  downgraded to `candidate`, re-attributed to the assistant, description
  rewritten to match the corrected one-row-per-transfer shape.
- Design docs split and rewritten: `transfer-ledger-design.md` is now a
  clean current-state reference (schema, pairing algorithm, real worked
  examples, all matching what's actually built); the discovery narrative
  (original two-tier plan, the tier-3 gap, the retroactive re-scan bug,
  the TUI display bugs, and this session's schema correction) moved to
  the new `transfer-ledger-history.md`.
- Tests: 81 total, all passing throughout every step (rewritten to
  exercise the new schema, not just re-labelled). `cargo build`/`clippy
  --all-targets`: 0 errors, same pre-existing dead-code baseline.

**State of the project:** Delta: Transfer Ledger is functionally
complete and, this time, structurally sound — a transfer entry is now
genuinely the link between two transactions, matching the user's
domain model, not two independently-stored legs. Everything from this
session remains uncommitted, sitting alongside the already-uncommitted
leader-key nav / Monthly Transfers v1 changes from prior sessions.

**Immediate next priorities:** unchanged from the previous checkpoint —
see "What's Next" at the top of this file.

## Checkpoint: Session 2026-07-13e

**Completed:**
- Renamed `derive_spend_entries` to `run_derivation` throughout
  `src/derive.rs`, `src/analysis.rs`, `src/main.rs`, `src/db/spend.rs`,
  `src/model.rs` (function, call sites, test names, doc comments) —
  prompted by the user questioning why transfer pairing ran "as part of
  derive_spend_entries" when it also derives transfers/card payments.
  `cargo build`/`test`/`clippy` clean throughout, 81 tests passing.
- Fixed a genuinely self-contradictory paragraph in
  `doc/implementation-notes/transfer-ledger-design.md` explaining
  `transaction_links` vs `transfer_entries` — traced the root cause via
  a read-only fable-model agent review (see
  `doc/implementation-notes/transfer-ledger-critique.md`, newly written
  up and linked from the design doc) rather than just improving the
  prose.
- Trimmed the design doc per the user's request: dropped the full DDL
  block (now points at `src/db/schema.sql`), replaced it with a mermaid
  ER diagram, and removed `transaction_links` from that diagram since
  it isn't actually part of the transfer ledger.
- Added **Credit Card Payment** to `doc/domain/ubiquitous-language.md`
  (candidate) — the user asked for "card payment" to be made explicit
  and distinguished from a card *purchase* (spend), since the informal
  term was ambiguous between opposite ends of the household boundary.
- Fable review's key finding: `Classification::CardPayment` should
  already be an internal transfer by the project's own agreed
  definition, but still writes to the legacy `transaction_links` table
  instead of `transfer_entries` — logged as new Task 4 under Delta:
  Transfer Ledger, deliberately **not fixed this session** (user asked
  for docs/plan only, real fix deferred to a new session).
- User flagged a separate, unrelated observation for later: OFX's
  `NAME` field truncation is "super annoying" and worth checking whether
  Barclaycard's CSV export avoids it, possibly reopening whether OFX is
  worth keeping as the primary bank import format — logged as a TODO
  note under Delta: Bank Transaction Import, Task 1, not investigated
  yet.

**State of the project:** Delta: Transfer Ledger's schema/pairing work
(Tasks 1-3) is done and structurally sound; this session found and
documented (but deliberately did not fix) a real gap where credit card
payments don't yet participate in it. Everything remains uncommitted,
alongside all prior uncommitted sessions' work.

**Immediate next priorities:** see "What's Next" at the top of this
file.

## Checkpoint: Session 2026-07-13f

**Completed:**
- Investigated three real alternate Barclays export formats for the
  current account against OFX's `NAME`-field truncation problem
  (`~/Downloads/data.csv`, `Transaction.pdf`, `data.qbo` — all
  scratch-only, never imported or committed). Full findings in new doc
  `doc/implementation-notes/optimising-import-data.md`, linked from
  Delta: Bank Transaction Import, Task 1.
- **`data.csv`**: same truncation failure mode as OFX (label and account
  number share one fixed-width field), no `FITID` equivalent, no balance
  column — no advantage over OFX found.
- **`data.qbo`**: confirmed to be the identical OFX 1.02 SGML payload
  `BarclaysOfxParser` already parses (same header, same `FITID`, same
  `NAME` field, same truncation) — not a distinct format, just a
  different file extension. `.qbo` isn't in `src/import/pipeline.rs`'s
  extension map today; a one-line addition if ever needed, not
  currently required.
- **`Transaction.pdf`** (current-account statement PDF, distinct from
  the already-built `BarclaycardPdfParser`): a genuine lead. Its
  transfer-type rows (`Funds Transfer`/`Standing Order`/`Direct
  Debit`/`Bill Payment`) carry the counterpart sort code/account number
  on its own line, separate from the truncated free-text label — so the
  account number is never truncated, unlike OFX where label and account
  number share one 32-char `NAME` field. This directly targets the
  `SHARED BILLS ACCO`-style gap driving Delta: Transfer Ledger's
  self-reference-match tier. Also has a per-transaction running balance
  (neither OFX nor CSV provide this per-line) and a "Pending debit card
  transactions" section with untruncated merchant descriptions and full
  card numbers, not currently importable by anything. Has no `FITID`
  equivalent — real-data check found a hash of `(date, amount,
  description)` alone is unsafe as a de-dup key (two genuine same-day,
  same-amount, same-description collisions found in 573 real
  transactions), but the running balance disambiguates them and is
  proposed as a required part of a synthetic de-dup key for this format.
- Evaluated three further options surfaced via external search
  (third-party PDF-to-CSV converters, a browser extension against a live
  Barclays session, Open Banking) — first two rejected as inconsistent
  with `ledgr`'s no-data-leaves-the-machine design; Open Banking is
  already tracked separately under Delta: Live Open Banking (Enable
  Banking) and not duplicated here.
- Not decided: whether to actually build a `BarclaysStatementPdfParser`
  for the current-account PDF format. Deliberately left as a TODO/open
  question, not actioned this session.

**State of the project:** unchanged from Session 2026-07-13e otherwise —
the real fix (Delta: Transfer Ledger, Task 4: migrate credit card
payment matching into `transfer_entries`) is still the next priority for
a new session; this session was research-only, no code touched, no
plan.md task marked done. The PDF-format lead is a separate, independent
thread (Delta: Bank Transaction Import, Task 1) that can be picked up
whenever, not a blocker for Task 4.

**Immediate next priorities:** see "What's Next" at the top of this
file.

## Checkpoint: Session 2026-07-13g

**Completed (run autonomously, per the user's request):**
- Sign-off obtained: "Transfer Entry"/"Transfer Ledger"/"Credit Card
  Payment" promoted `candidate` → `established` in
  `doc/domain/ubiquitous-language.md`.
- Delta: Transfer Ledger, Task 4 done in full — see that task's entry
  above for the complete write-up. Headline: credit card payment
  matching migrated off `transaction_links` onto `transfer_entries`
  (new `TransferPairMethod::CreditCardPaymentMatch`), both real bugs the
  critique doc flagged (endless reprocessing of matched payments;
  permanent double-count of ever-unmatched ones) fixed as a natural
  consequence, `LinkRelation::Transfer` removed, real database migrated
  and validated (32/32 real credit card payments matched, idempotent on
  a second run, no balance/transaction-count regression). 81 tests
  total, all still passing (two existing card-payment tests expanded
  rather than new ones added); `cargo clippy --all-targets` clean (same
  pre-existing dead-code baseline).
- Updated `doc/implementation-notes/transfer-ledger-design.md` (new
  "Credit card payment matching" section) and
  `doc/implementation-notes/transfer-ledger-critique.md` (resolution
  note at the top) to match.

**Not done / left for later:** Delta: Transfer Ledger, Task 5 (retire
`transaction_links` entirely by absorbing refund links into
`spend_entries`) — not started, still needs the column-shape design
question answered (see that task's TODOs). No manual click-through of
the live TUI this session (verified via `cargo build`/`test`/`clippy`
and real-database SQL checks only, consistent with how Task 3 was
verified).

**Immediate next priorities:** see "What's Next" at the top of this
file.

## Checkpoint: Session 2026-07-17

**Completed:** none — reorientation only, no code/doc changes. The user
had stepped away and lost track of state; re-briefed from this file plus
`git status`/`git diff --stat` (Session 2026-07-13g's Task 4 work is
still fully done but **uncommitted** — 10 files changed, listed in that
checkpoint). Also touched up
`doc/implementation-notes/transfer-ledger-design.md` in the prior session
(2026-07-14, not separately checkpointed): fixed three spots left stale
by Task 4 (the `transaction_links`-tangle notes, the naming open
question) and refreshed the real-data numbers/added a
`credit_card_payment_match` worked example — folded into the same
uncommitted diff.

**Decision:** the user wants a fresh session for the next real work
(Task 5) rather than continuing here.

**Immediate next priorities:** see "What's Next" at the top of this
file — review and commit the Task 4 diff first, then start Task 5.

## Checkpoint: Session 2026-07-17b

**Completed:**
- Extended `ledgr status` (Bank Transaction Import, Task 3): Spend Ledger
  and Transfer Ledger summary sections, unpaired-transfer split into
  "reference accounts" (expected) vs "unresolved" (needs review), and a
  Balance-column right-alignment fix. Prompted by the user asking how to
  test the still-uncommitted Task 4 work, then asking where the credit
  card payment matched/unmatched counts (until now only ever printed
  transiently by `ledgr import`) could be seen afterwards.
- Delta: Transfer Ledger, **Task 5 done in full** — see that task's entry
  above. Headline: `transaction_links` dropped entirely;
  `spend_entries.refunds_spend_entry_id` (self-referencing column)
  replaces its one remaining live purpose (refund linking); real database
  migrated (backfilled 3 existing refund links, dropped the table,
  confirmed idempotent).
- TUI Analysis Views, Task 3 addendum: `Screen::MonthlyGap`/`MonthSpend`
  renamed to `Screen::MonthlySpend`/`SpendMonth` (matching the
  `MonthlyTransfers`/`TransferMonth` pattern), leader key `<space>g` →
  `<space>s` — "gap" doesn't exist as a concept yet (no income ledger),
  so the old name overclaimed. Verified live via `tmux` against the real
  database.
- All work this session verified against the real `~/.local/share/ledgr/ledgr.db`
  (backed up first: `ledgr.db.bak-20260717205948-pre-transaction-links-retirement`),
  not just `cargo test`. 84 tests passing throughout; `cargo clippy
  --all-targets` clean at the same pre-existing dead-code baseline
  (nothing new) after every change.
- Also fixed as a side effect: the `ledgr import` summary line's
  `entr(y/ies)` pluralisation-attempt wording, at the user's request
  ("just have entries") — simplified to plain "entries" everywhere it
  appeared (`src/main.rs`).

**Not done / left for later:** Delta: Reclaimable Work Expenses — the
user asked what happened to this mid-session; confirmed it's still just
the sketch already in this file (Task 1, TODO), nothing built, not
started this session.

**Decision:** the user is committing this session's work themselves; no
git operations performed by the assistant this session.

**Immediate next priorities:** see "What's Next" at the top of this
file — next delta not yet chosen (candidates: Reconciliation, or
resuming the Credit Card/Amazon/Gap chain).

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
