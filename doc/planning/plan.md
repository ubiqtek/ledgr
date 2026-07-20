# ledgr — Plan

## What's Next

- **Next:** Commit outstanding uncommitted work to git (several sessions' worth — see Checkpoint below), then **Delta: Double-Entry Accounting, Task 1 — Evaluate a double-entry model for ledgr**, now top priority ahead of Task 3 (partner's credit card import — deprioritised, not dropped)
- **Sub-doc:** none
- **Blockers:** None
- **Context:** Decided 2026-07-20 to prioritise the double-entry evaluation now rather than let it keep sitting as "future/exploratory" — see Delta: Double-Entry Accounting, Task 1's 2026-07-20 decision note for why

## Summary

| Delta | Task | Status |
|-------|------|--------|
| [Delta: Decide on Switching to PDF for Transaction Import](#delta-decide-on-switching-to-pdf-for-transaction-import) | [1. Decide whether to build a BarclaysStatementPdfParser](#task-1-decide-whether-to-build-a-barclaysstatementpdfparser) | TODO |
| [Delta: Automatic Inbox Import](#delta-automatic-inbox-import) | [1. Inbox change notification](#task-1-inbox-change-notification) | TODO |
| [Delta: Credit Card Transaction Import](#delta-credit-card-transaction-import) | [1. Credit card statement parser](#task-1-credit-card-statement-parser) | ✓ DONE |
| | [2. Evaluate Barclaycard PDF export](#task-2-evaluate-barclaycard-pdf-export) | ✓ DONE |
| | [3. Import the user's partner's credit card](#task-3-import-the-users-partners-credit-card) | TODO — deprioritised below Delta: Double-Entry Accounting |
| | [4. Manual spend entries via a proxy account](#task-4-manual-spend-entries-via-a-proxy-account) | ✓ DONE |
| | [5. Match credit card payments to bank-side transfers](#task-5-match-credit-card-payments-to-bank-side-transfers) | ✓ DONE |
| | [6. Spend-from-transfer follow-ups](#task-6-spend-from-transfer-follow-ups) | ✓ DONE |
| [Delta: Amazon Order Import](#delta-amazon-order-import) | [1. Evaluate automation route — email scanning vs manual export](#task-1-evaluate-automation-route--email-scanning-vs-manual-export) | TODO |
| | [2. Amazon order import](#task-2-amazon-order-import) | TODO |
| [Delta: Review and Re-classification TUI](#delta-review-and-re-classification-tui) | [1. Review queue screen](#task-1-review-queue-screen) | TODO — deprioritised below Delta: The Gap |
| [Delta: Reconciliation](#delta-reconciliation) | [1. Design account-level and household-level reconciliation checks](#task-1-design-account-level-and-household-level-reconciliation-checks) | TODO |
| [Delta: The Gap](#delta-the-gap) | [1. Minimal income ledger](#task-1-minimal-income-ledger) | ✓ DONE |
| | [2. Gap calculation](#task-2-gap-calculation) | ✓ DONE |
| | [3. Discovery about recording assets and liabilities](#task-3-discovery-about-recording-assets-and-liabilities) | ✓ DONE |
| | [4. Implement assets and liabilities as accounts](#task-4-implement-assets-and-liabilities-as-accounts) | TODO |
| | [5. Drive the Gap screen's "Untracked" figure to zero](#task-5-drive-the-gap-screens-untracked-figure-to-zero) | TODO |
| [Delta: Mortgage Tracking](#delta-mortgage-tracking) | [1. Design the mortgage domain model](#task-1-design-the-mortgage-domain-model) | TODO |
| [Delta: Spending Categorisation](#delta-spending-categorisation) | [1. Confirm Rebel Finance taxonomy](#task-1-confirm-rebel-finance-taxonomy) | IN PROGRESS |
| | [2. Rule-based categorisation engine](#task-2-rule-based-categorisation-engine) | TODO |
| | [3. Inference-assisted categorisation](#task-3-inference-assisted-categorisation) | TODO |
| [Delta: Other Transaction Import](#delta-other-transaction-import) | [1. Pension/investment statement parser](#task-1-pensioninvestment-statement-parser) | TODO |
| [Delta: TUI Analysis Views](#delta-tui-analysis-views) | [1. Transaction list view](#task-1-transaction-list-view) | ✓ DONE |
| | [2. Net worth / spending trend views](#task-2-net-worth--spending-trend-views) | TODO |
| | [3. Monthly Gap screen and spend drill-down](#task-3-monthly-gap-screen-and-spend-drill-down) | IN PROGRESS |
| | [4. Leader-key navigation](#task-4-leader-key-navigation) | ✓ DONE — uncommitted, pending review |
| | [5. Right-align numeric column headers on the Spend/Income month drill-down screens](#task-5-right-align-numeric-column-headers-on-the-spendinccome-month-drill-down-screens) | ✓ DONE |
| | [6. Right-align the Monthly Transfers header row](#task-6-right-align-the-monthly-transfers-header-row) | ✓ DONE |
| | [7. Split Monthly Transfers into In/Out/Household columns](#task-7-split-monthly-transfers-into-inouthousehold-columns) | ✓ DONE |
| | [8. Filterable Transfers drill-down](#task-8-filterable-transfers-drill-down) | ✓ DONE |
| [Delta: Packaging & Distribution](#delta-packaging--distribution) | [1. Publish `ledgr` to crates.io](#task-1-publish-ledgr-to-cratesio) | ✓ DONE |
| | [2. Web frontend](#task-2-web-frontend) | TODO |
| [Delta: Live Open Banking (Enable Banking)](#delta-live-open-banking-enable-banking) | [1. Evaluate feasibility & security model](#task-1-evaluate-feasibility--security-model) | IN PROGRESS |
| [Delta: Double-Entry Accounting](#delta-double-entry-accounting) | [1. Evaluate a double-entry model for ledgr](#task-1-evaluate-a-double-entry-model-for-ledgr) | IN PROGRESS — now top priority |
| [Delta: Payslip Import](#delta-payslip-import) | [1. Evaluate payslip format and scope](#task-1-evaluate-payslip-format-and-scope) | TODO |
| [Delta: Reclaimable Work Expenses](#delta-reclaimable-work-expenses) | [1. Design the reclaimable expenses ledger and marking flow](#task-1-design-the-reclaimable-expenses-ledger-and-marking-flow) | TODO |
| [Delta: Regular Payments](#delta-regular-payments) | [1. Design regular payment recognition and labelling](#task-1-design-regular-payment-recognition-and-labelling) | TODO |
| [Delta: Classification Rules Tidying](#delta-classification-rules-tidying) | [1. Bundle classify()'s growing parameter list](#task-1-bundle-classifys-growing-parameter-list) | TODO |
| | [2. Remove throwaway diagnostic/maintenance scripts](#task-2-remove-throwaway-diagnosticmaintenance-scripts) | TODO |
| | [3. Revisit Classification::Refund's hardcoded confidence](#task-3-revisit-classificationrefunds-hardcoded-confidence) | TODO |

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
- TODO (2026-07-12) — **deprioritised 2026-07-20** below Delta:
  Double-Entry Accounting, Task 1 (not dropped, just no longer next).
  The user will load his partner's credit card
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
- ✓ DONE (2026-07-19 session) — the user's partner's own spend (on her
  personal bank accounts, which won't be imported) still needs to count
  towards household spend/the Gap. Built as: `s` on `Screen::TransferMonth`
  (the Monthly Transfers drill-down), not the Spend drill-down originally
  envisioned — triggered directly from the transfer that sent her the
  money, since the transfer already carries the date/amount/which
  Reference Household Account it went to.
  - Only activates when one leg of the selected transfer is a Reference
    Household Account (e.g. Romina's accounts) — a no-op on transfers
    between the user's own tracked accounts.
  - Design questions resolved: entry flow is a TUI form (`SpendFromTransferForm`
    in `src/app.rs`, mirrors the existing "add reference" form's
    Tab/Enter/Esc three-field shape), pre-filled Date/Amount from the
    transfer, free-text Description. Proxy accounts reuse the existing
    `AccountType::Other` rather than a new `AccountType::Proxy` variant —
    no functional difference since cash calculations already whitelist
    only `Current`/`Savings`, and adding a new CHECK-constrained enum
    value would have needed a live-database migration for no benefit.
  - One proxy account per Reference Household Account, lazily created
    via the existing `Db::find_or_create_account`, named
    `"<household label> (Manual Spend)"` (e.g. "Romina Primary Account
    (Manual Spend)").
  - Submitting posts a `Transaction` to the proxy account and a
    `spend_entries` row (`classified_by = 'manual'`,
    `rule_name = 'manual_entry'`) via the existing
    `Db::insert_spend_entry_with_source` — the originating
    `transfer_entries` row is untouched (the transfer itself was real
    and correctly classified; this is a separate record of what the
    money was then spent on).
  - One spend per transfer only, by design — splitting one transfer into
    multiple spends is left for a later delta.
  - Verified live via `tmux` against the real database: selected the
    real £1,415.00 30 March transfer to Romina Primary Account
    (identified via the new transfer filter, see Delta: TUI Analysis
    Views Task 8), entered "Holiday", confirmed the spend entry, proxy
    account, and transaction were created correctly, and that the Gap
    screen's Untracked figure dropped by exactly £1,415.00 (March:
    -£2,018.26 → -£603.26; YTD: -£4,084.19 → -£2,669.19) — then removed
    the test entry afterwards (real `ledgr.db` backed up first as
    `ledgr.db.bak-20260719224447-remove-test-manual-spend`; transaction,
    spend entry, and proxy account all deleted, confirmed empty).
  - 91 unit tests still passing; `cargo clippy` clean (same pre-existing
    warnings). Not yet committed to git.

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

### Task 6: Spend-from-transfer follow-ups
- ✓ DONE (2026-07-20 session) — all four issues from real-usage feedback
  fixed together:
  1. **Gap screen now auto-refreshes on `back()`.** `App::back()`
     (`src/app.rs`) now returns `anyhow::Result<()>` and, when the screen
     it's returning to is `Screen::Gap`, re-runs `load_gap_data` (the same
     helper `open_gap` uses) before restoring the selection — so returning
     from `Screen::TransferMonth` after recording a spend (or from
     anywhere else) shows current figures without a full app restart.
     `toggle_help` and `main.rs`'s `q`/`Esc`/`?` handlers updated to
     propagate the `Result`.
  2. **Spend entries now link back to their originating transfer.** New
     `spend_entries.transfer_entry_id` column (nullable, `ON DELETE SET
     NULL`, migration `migrate_add_spend_entries_transfer_entry_id_column`
     in `src/db/mod.rs`) — `NULL` for every normal derived spend entry,
     set to the transfer's id only for entries created via `s` on
     `Screen::TransferMonth`. New `Db::spend_entry_for_transfer` resolves
     it back for display.
  3. **All three entry types now support a note.** `transfer_entries`
     gained a `note TEXT` column (migration
     `migrate_add_transfer_entries_note_column`, deliberately run *after*
     the existing leg-shape/pair-method rename migrations so it never
     touches a table mid-rename) plus `Db::set_transfer_entry_note`.
     `income_entries.note` already existed in the schema — added
     `Db::set_income_entry_note`. `App::start_editing_note`/`commit_note`
     generalised from SpendMonth-only to also cover `Screen::IncomeMonth`
     and `Screen::TransferMonth` (same `n` key, same popup, routes on
     `self.screen` to the right entry/setter). Transfer notes show inline
     in the description column (`📝 note`), matching the existing spend/
     income convention exactly.
  4. **New "Tracked Spend" column on the Transfers drill-down.** New
     `TransferEntry.has_tracked_spend` (an `EXISTS` subquery against
     `spend_entries.transfer_entry_id` in `transfer_entries_for_month`)
     shows `Y` once a spend has been recorded — `App::commit_spend_form`
     also updates the in-memory row immediately so the column reflects it
     without leaving the screen. The existing `i` "both legs" popup
     (rather than a separate one — decided at implementation time) now
     also shows the linked spend entry's date/amount/description when
     present (`TransferDetail.linked_spend`, populated in
     `show_transfer_detail` via `spend_entry_for_transfer`).
  - Help screen text updated for the generalised `n` and extended `i`
    bindings.
  - 91 unit tests still passing; `cargo build`/`cargo clippy --all-targets`
    clean (same pre-existing dead-code warnings, nothing new).
  - **Verified live via `tmux`**, against a scratch copy of the real
    database under a throwaway `$HOME` (never touched the real
    `~/.local/share/ledgr/ledgr.db`): recorded a spend from the real
    £1,415.00 30 March transfer to Romina Primary Account — "Tracked
    Spend" showed `Y` immediately, the `i` popup showed "Tracked spend:
    2026-03-30 -1415.00 GBP Holiday", a note typed via `n` on that same
    transfer row appeared inline immediately, a note typed via `n` on an
    Income Month row worked identically, and backing out to `Screen::Gap`
    showed Spend/Untracked updated by exactly £1,415.00 (Spend -£43,948.12
    → -£45,363.12; Untracked -£1,150.19 → £264.81) without restarting the
    app.
  - **Real backfill needed and done (2026-07-20, same-day follow-up):** the
    new `transfer_entry_id` link only applies going forward — 16 real
    `rule_name = 'manual_entry'` spend entries the user had already
    recorded via `s` earlier the same day (before this session's schema
    change existed) all had `transfer_entry_id` left `NULL`, so none
    showed as Tracked Spend. Root-caused via each entry's `classified_at`
    timestamp (2026-07-20T18:04–18:14Z, confirming they predated this
    session's fix, not the 19th as first assumed). Backfilled by matching
    each entry to the one `transfer_entries` row sharing its exact date +
    amount (unambiguous in every case — checked first). Real `ledgr.db`
    backed up first
    (`ledgr.db.bak-20260720201726-pre-transfer-entry-id-backfill`); all 16
    rows (ids 894–909, £110–£1,415 each, spanning Jan–Jun) now correctly
    link to their transfers and show `Y` in Tracked Spend.
  - Not yet committed to git.

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
- ✓ DONE (2026-07-17 session) — `income_entries` + `income_entry_sources`
  added to `src/db/schema.sql`, deliberately thinner than `spend_entries`
  per the design doc's original scope note: no `category_id`, no
  refund-style link, and `income_entry_sources` has no `role` column
  (every row is implicitly the source — income has no annotation concept
  yet). New `IncomeEntry`/`NewIncomeEntry`/`IncomeEntryWithAccount`/
  `MonthlyIncome` in `src/model.rs`; persistence in new `src/db/income.rs`
  (`insert_income_entry_with_source`, `monthly_income_totals`,
  `income_entries_for_month`), mirroring `src/db/spend.rs`'s shape.
  Domain term **Income Entry** recorded in
  `doc/domain/ubiquitous-language.md` (obvious derivative of the
  already-established **Income Ledger**/**Spend Entry** pattern).
- Derivation (`src/derive.rs`): new `Classification::Income` variant.
  `classify()` now routes `DIRECTDEP` (positive amount — salary/wages)
  and the Barclaycard PDF's own `"Other"` type tag (positive amount —
  cashback, Title-case, distinct from the generic OFX `"OTHER"` fallback
  used elsewhere so there's no collision) to income instead of
  `OutOfScope`. `"Payment received"` (credit card bill payments)
  deliberately left alone — those are transfers, matched by the existing
  card-payment pairing logic, not income. `run_derivation`'s
  `pending_derivation_transactions` query (`src/db/spend.rs`) extended to
  also exclude transactions already linked via `income_entry_sources`,
  so a `ledgr import` re-run doesn't duplicate income entries — same
  idempotency shape as spend/transfers. New `DerivationSummary.income_entries_created`
  field, surfaced in `ledgr import`'s own output line.
- **Real backfill** (2026-07-17): real `ledgr.db` backed up
  (`ledgr.db.bak-20260717230737-pre-income-ledger`) and `ledgr import`
  re-run — 3 real Barclaycard Cashback transactions (previously
  `OutOfScope`, silently invisible) became income entries (£25.12 Jan,
  £11.53 Mar, £2.69 Apr); 0 salary/`DIRECTDEP` income found yet in real
  data. Confirmed idempotent (re-running creates 0 new entries); all
  existing spend ledger (822 entries) and transfer ledger (202 entries,
  32 card payments matched) counts unchanged.
- **TUI screens added** (requested alongside this task, not split out):
  `Screen::MonthlyIncome`/`Screen::IncomeMonth` in `app.rs`/`ui.rs`,
  copying the Monthly Spend/Spend Month screens' design exactly — same
  table layout, same per-month drill-down showing date/amount/
  counterparty/description/rule/account. Reached via `<space>i`
  (`main.rs`'s leader-key match arm), help screen updated. Verified live
  via `tmux`: `<space>i` opens "Monthly Income" showing the 3 real
  months; `Enter` drills into "Income — 2026-04" showing the real
  cashback entry with its account resolved correctly.
- 86 unit tests total (2 new: `classifies_a_direct_deposit_as_income`
  now replaces the old `..._as_out_of_scope` test,
  `classifies_a_credit_card_cashback_as_income`,
  `run_derivation_creates_an_income_entry_for_a_direct_deposit` covering
  both creation and idempotency); `cargo clippy` clean (same pre-existing
  dead-code warnings as before, nothing new).
- **Real bug found and fixed (2026-07-18 session):** the user reported the income ledger was recording almost nothing (£2.69/£11.53/£25.12 across 3 months — only Barclaycard cashback). Root cause: `classify()` in `src/derive.rs` only routed money to Income via OFX `TRNTYPE == "DIRECTDEP"`, but real Barclays OFX exports carry no `<TRNTYPE>` element at all for Bank Giro Credit (BGC) transactions — confirmed by grepping the real archived OFX file (`data(1).ofx`) for the `AZIMO LTD Pleo Technologies BGC` salary lines: no TRNTYPE tag present. `trn_type` is genuinely NULL in the DB for 967 of ~1230 real transactions (not a storage bug — the parser was already correct, the source data just omits it), so every salary/BGC-suffixed credit silently fell through to `OutOfScope`.
- Fix: added a new suffix-based rule in `classify()` (`src/derive.rs`, same match block as the existing CPM/CRM/FT suffix rules) — NAME ending in `"BGC"` with positive amount → `Classification::Income { rule_name: "bank_giro_credit", confidence: 0.75 }`. Placed after rule 1c (household-name matching) so a household member's own inbound BGC (e.g. "ROMINA SCARAMAGLI pizza BGC") is still correctly caught as `InternalTransfer` first, not misclassified as income — verified with new test `a_household_members_bgc_credit_is_still_an_internal_transfer`. Also added `classifies_a_bank_giro_credit_as_income`. 90 tests total, all passing; `cargo clippy` clean.
- **Real backfill (2026-07-18):** real `ledgr.db` backed up (`ledgr.db.bak-20260718080143-pre-bgc-income-rule`) and `ledgr import` re-run — 20 new income entries created (rule_name `"bank_giro_credit"`, £37,151.74 total), covering real salary (AZIMO LTD Pleo Technologies BGC, ~£5.8k-6.4k/month), HMRC PAYE credits, and other genuine inbound BGC transfers (SIMPLYHEALTH claim payouts, World of Books, a lottery win, family gifts). Monthly income now realistic (£5,788-£6,574/month) instead of the £2.69-£25.12 cashback-only totals. Verified live via `tmux` against the real TUI.
- **New TUI feature added alongside the fix:** an `i` key on `Screen::IncomeMonth` (income drill-down) pops up the raw source transaction behind the selected income entry, for verification — mirrors the existing "both legs of transfer" popup pattern on `Screen::TransferMonth` exactly (`app.income_detail: Option<Transaction>`, `show_income_detail`/`close_income_detail` in `app.rs`, `draw_income_detail` in `ui.rs`, dismissed by any key). Required adding `transaction_id: Id` to `IncomeEntryWithAccount` (`src/model.rs`) and extending `income_entries_for_month`'s query (`src/db/income.rs`) to select `t.id`. Verified live via `tmux`. Help screen text updated.
- Files changed: `src/derive.rs`, `src/model.rs`, `src/db/income.rs`, `src/app.rs`, `src/main.rs`, `src/ui.rs`. Not yet committed to git — sitting in the working tree for the user's review.
- **Income vs Refund/Reimbursement redesign (2026-07-18, same-day follow-up session):** the user pushed back on treating all inbound BGC money as income — cashback and SimplyHealth claim payouts are money already spent coming back, not new income. Spawned a `fable`-model agent to think through a first-principles test and write a proposal to `doc/implementation-notes/income-vs-refund-classification-proposal.md`. Core test adopted: an inflow is a **Refund/Reimbursement** (spend ledger, sign-reversed) if it exists because of, and is bounded by, a prior household outflow that actually passed through the spend ledger; otherwise it's **Income**. This resolved an apparent contradiction (cashback reverses spend the ledger can see → Refund; a PAYE tax refund reverses a deduction that never appeared as spend → Income) without treating the two inconsistently.
- **Domain language split** (`doc/domain/ubiquitous-language.md`): the old single "Reimbursements and Refunds" entry split into **Refund** (linked/linkable to one specific original purchase, e.g. a card refund) and **Reimbursement** (not linked to one transaction — cashback, a claim payout, a person settling up) — same underlying mechanism (`Classification::Refund`, sign-reversed spend entry), different domain identity. Also added **Registered Person** (an external individual registered by name, e.g. family/friends, whose unexplained inbound payments default to Reimbursement not Income) and **Income Source** (a registered external payer — employer, tax authority — driving high-confidence Income). `Income Ledger`'s own entry refined to explain why the household-boundary test alone isn't sufficient. Two stale `transaction_links` references (a retired table) cleaned up in `spend-ledger-design.md`/`derive.rs` while editing nearby text.
- **New config-driven classification** (`src/config.rs`): three new registries — `income_sources` (typed `Salary`/`TaxAuthority`, each with its own `rule_name`/confidence — `employment_income` 0.95, `tax_refund` 0.8), `registered_people` (external individuals, default to `person_reimbursement` Refund), and `reimbursement_sources` (external institutions/schemes like a health cash plan, free-text `kind` for display, default to `claim_reimbursement` Refund) — the last added specifically for SimplyHealth. Every entry carries `name` (the literal string matched against a transaction's description), an optional `label` (short display nickname), and an optional `full_name` (the entity's true/full proper name, shown in `ledgr status`'s new "Name" column, distinct from `label` and from `name`/"Matches" which may be a truncated or payment-processor form).
- **Real truncation bug found and handled:** Aria's real transaction (`"ARIA SCARAMAGLI-RE CHASE BGC"`) turned out to be genuinely truncated by the 32-char `NAME` field cap (her real surname is "Scaramagli-Reeves"), confirmed by counting characters — registering her true full name would have broken matching, since the character immediately after the truncation point isn't a word boundary. Handled by keeping `name` as the truncated form that actually matches, with `full_name` carrying the true name for display only.
- **Person-name matching extended** (`derive::matches_person_name`, renamed from `matches_household_member_name`): a real transaction for "Fraser Crichton" appeared as `"F Crichton NORWAY CAR BGC"` — `"<initial> <Surname>"` order, the *opposite* of Faster Payments' documented `"<Surname> <initial>"` echo order. Added as a third matched variant, grounded in real data (Bank Giro Credit sender names are chosen by the originating bank, not Barclays, so a different order was plausible and turned out to be real).
- **`classify()`'s residual `bank_giro_credit` rule** confidence dropped 0.75 → 0.5 now that the specific rules above absorb the explicable cases — what's left genuinely needs human review. Barclaycard cashback moved from `Classification::Income` to `Classification::Refund` (`rule_name: "cashback"`).
- **Real config populated and reconciled:** registered AZIMO LTD (Salary → "Pleo Technologies"), HMRC PAYE (Tax Authority → "HM Revenue & Customs"), Wendy Barritt ("Ma"), Fraser Crichton, Aria Scaramagli-Re (label "Aria", full name "Aria Scaramagli-Reeves"), and SimplyHealth (Health Scheme) in the real `~/.config/ledgr/config.toml`. Real `ledgr.db` backed up twice (`ledgr.db.bak-20260718091241-pre-income-source-rules`, and again before the SimplyHealth rule), income/spend entries for the affected rules cleared and re-derived — confirmed idempotent both times (0 new entries on a second `ledgr import` run). Real result: SimplyHealth's 7 claim payouts (£365.00) and Barclaycard cashback (£39.34) moved out of income into spend-ledger reimbursements; Fraser's reimbursement rule also surfaced two Norway-trip transactions (£284.02, £68.71 accommodation) not found during earlier manual searching. Monthly income now £5,809–£6,428 (salary + occasional HMRC/direct-deposit), down from the earlier all-BGC-as-income total.
- **`ledgr status`** gained a combined "Named Entities" table (Label / Type / Name / Matches columns) listing every `income_sources`/`registered_people`/`reimbursement_sources` entry — verified live against the real config.
- **TUI polish:** the Spend/Income per-month drill-down screen titles now show the month's total, e.g. `"Spend — 2026-07 — -1458.90 GBP"` / `"Income — 2026-06 — 6400.27 GBP"` (`src/ui.rs`'s `draw_spend_month`/`draw_income_month`).
- **Real root-cause correction, mid-session:** the earlier same-day "BGC transactions have no TRNTYPE" diagnosis turned out to be wrong — a wider grep (not a narrow 3-line context window) showed the real Barclays OFX file DOES carry `TRNTYPE=DIRECTDEP` for every BGC credit; the actual bug was that these rows were imported into the database before `trn_type` capture worked, and `Db::insert_transaction`'s FITID-based dedup never revisits an already-imported row, so a normal `ledgr import` re-run could never backfill it. Fixed with a one-off script (`examples/backfill_trn_type.rs`, re-parses every processed OFX file and updates any matching still-NULL row) — real DB backfilled, all 967 previously-NULL rows corrected, confirmed against the original 27-`DIRECTDEP` baseline from `doc/kb/ofx/structure.md`'s 939-transaction analysis. The BGC-suffix fallback rule added earlier this session remains useful as defence-in-depth (a transaction with a genuinely absent TRNTYPE would still be caught) even though it wasn't the actual fix for this specific bug. Full writeup: `doc/implementation-notes/spend-ledger-design.md`'s "Stale-data footgun" note and updated derivation rules table (now also reflects rules 1b/1c/2c/6/6b/8-10 that had drifted out of sync with the code).
- 91 unit tests total; `cargo clippy --all-targets` clean except a new `#[allow(clippy::too_many_arguments)]` on `classify()` (now 8 parameters) — see the new Delta: Classification Rules Tidying below.
- Still not committed to git — everything sitting in the working tree for the user's review.
- **Three more real inbound-payment mis-classifications found and fixed (2026-07-18, same-day follow-up)**, same "register the sender" pattern as SimplyHealth: (1) Pleo Technologies paying back an out-of-pocket work expense directly (£49.83, description `"PLEO TECHNOLOGIES PLEO TECHNOLO"`, distinct from the salary-via-Azimo route already registered under `income_sources`) — added as a `reimbursement_sources` entry; (2) Great Western Railway Delay Repay (£21.38, description `"GREAT WESTERN TRAI GWR-2080-068"`) — added as a `reimbursement_sources` entry; (3) the user's brother "S Barritt" sending a BGC payment (£43.17, description `"S Barritt FARTER BGC"`) — added as a `registered_people` entry via the new TUI form (see below). Real `ledgr.db` backed up before each fix (`ledgr.db.bak-20260718151046-pre-pleo-reimbursement-rule` and further timestamped backups for GWR); all three re-derived idempotently into the spend ledger as reimbursements (`claim_reimbursement`/`person_reimbursement` rule names).
- **New TUI feature: `a` "add reference" form on `Screen::IncomeMonth`** — pops up a 3-field form (Name, pre-filled with a guess from the transaction description; Label; Full name), Tab/Down and Shift-Tab/Up move between fields, Enter advances to the next field or submits on the last field, Esc cancels. Submitting registers the entry's sender as a new `registered_people` entry in `config.toml`, deletes that one income entry (freeing its source transaction for re-derivation), and live re-runs `derive::run_derivation` so it moves to the spend ledger as a reimbursement — all without leaving the screen. New: `App::start_adding_person`/`person_form_next_field`/`person_form_previous_field`/`person_form_push_char`/`person_form_pop_char`/`person_form_enter`/`commit_person_form` and the `PersonForm`/`PersonFormField` types (`src/app.rs`); `Db::delete_income_entry` (`src/db/income.rs`); `Config::add_registered_person` (`src/config.rs`); `draw_person_form` (`src/ui.rs`). Help screen updated. Verified live via `tmux` end-to-end against the real database.
- **Known gap, not fixed:** `Config::save` re-serializes the whole `config.toml` via `toml::to_string_pretty`, which strips hand-written explanatory comments (e.g. the Aria-truncation note, the Pleo/GWR reasoning notes). Comments lost during this session's `a`-form use were manually restored by hand; this will recur on the next `a` use. Worth a future task (e.g. a comment-preserving TOML writer, or moving the "why" into a `note` field instead of file comments) if it becomes a real nuisance — not fixed now.
- **Ad-hoc SimplyHealth net-cost query (not a built feature):** computed via direct SQL against `spend_entries`/`transactions` — £276.00 paid in premiums vs £365.00 claimed back over Jan–Jun 2026 (6 standing-order payments vs 7 claims), net +£89.00 in the user's favour. The user wants a proper annual-cap-aware version (there's a yearly claim cap and the policy year isn't finished), but deferred supplying the cap amount and policy-year start month to a later session — do not invent these values or add a new Delta for it yet.

### Task 2: Gap calculation
- ✓ DONE (2026-07-18 session) — built as a TUI screen (`Screen::Gap`,
  reached via `<space>g`), not a CLI command — the user wanted a single
  report pane: a YTD (calendar-year-to-date) summary at the top and the
  full month-by-month history below, in one bordered block with no
  border between the two sections, not two separately-bordered widgets.
  Deliberately **not navigable** — no `TableState`/row selection/
  drill-down, unlike every other monthly screen — since there's nothing
  to jump into from a summary report.
  - New `MonthlyGap` model (`src/model.rs`): month, income_minor,
    salary_minor, spend_minor, gap_minor (`income_minor + spend_minor`,
    spend already signed negative — same convention throughout).
  - New `Db::monthly_gap_totals` (`src/db/gap.rs`) combines the spend and
    income ledgers per month via a `UNION` of both ledgers' distinct
    months `LEFT JOIN`ed back onto each side's aggregates, so a month
    present in only one ledger still gets a row (`COALESCE`d to 0 on the
    missing side) rather than being silently dropped by an inner join.
  - `App::open_gap` (`src/app.rs`) loads it and navigates to
    `Screen::Gap`; `draw_gap`/`draw_gap_summary`/`draw_gap_months`
    (`src/ui.rs`) render the two sections via a borderless inner
    `Layout` split inside one bordered outer `Block`. Month table columns:
    Month / Income / Spend / Gap / Salary / Other (Salary+Other break down
    Income, matching the existing Monthly Income screen). All money
    columns right-aligned, including the YTD summary's plain-text lines
    (fixed-width label + right-justified amount, not a table).
  - Verified live via `tmux` against the real database: `<space>g` opens
    "Gap" showing 2026 YTD (Income £36,452.36 / Spend -£42,473.02 / Gap
    -£6,020.66) and 7 months of history matching the existing Monthly
    Spend/Income screens' per-month totals exactly.
  - 91 unit tests still passing (no new tests — no new classification
    logic, just an aggregation query); `cargo build`/`clippy` clean (same
    pre-existing dead-code warnings, nothing new). Not yet committed to
    git.
  - **Not addressed, flagged by the user as a future direction, not a
    task yet:** the growing pile of Reference Household Accounts,
    Registered People, Income Sources, and Reimbursement Sources
    (`config.toml`) is starting to resemble a **chart of accounts** —
    worth revisiting once Delta: Double-Entry Accounting's evaluation
    happens, not before. Explicitly not scoped into that delta yet, just
    noted here so the connection isn't lost.
  - **Follow-up same session: added a cash-drawdown check to the same
    summary.** The user noticed the YTD Gap being negative (spending more
    than earning) was concerning without knowing where the shortfall was
    coming from — suspected savings. Added `Db::cash_balance_as_of`
    (`src/db/gap.rs`), summing `Db::balance_as_of` (previously-unused,
    already built for this exact reconstruction) across every `Current`/
    `Savings` account — **deliberately excludes `CreditCard` accounts**,
    since their balance is already reflected in the spend ledger via
    card-payment matching and double-counting it would misstate the
    drawdown. `App::open_gap` now also loads `cash_at_year_start`/
    `cash_now` (1 Jan of the current year vs today); `draw_gap_summary`
    shows both plus their difference under the existing Income/Spend/Gap
    lines, separated by a blank line, same pane.
  - **Follow-up, same session: excluded the in-progress current month
    from the YTD summary** (`draw_gap_summary`'s `ytd` filter now also
    requires `m.month < current_month`) — the user pointed out July was
    skewing the ratio: spend so far this month with no matching income
    yet (salary not paid), since the month isn't over. The month-by-month
    table below still shows the current month's partial data (useful for
    the shorter-term view), only the YTD roll-up excludes it. Cash
    now/1 Jan are unaffected by this — they're real point-in-time
    balances, not period sums, so a partial month doesn't skew them.
  - **Real finding, investigated and explained (2026-07-18, same-day
    follow-up)** — ad-hoc SQL against the real database (no code changes),
    prompted by the user suspecting untracked outbound transfers. First
    ruled out a reconciliation bug: direct-summed every real Current/
    Savings transaction Jan-Jun and it matched the `balance_as_of`-derived
    cash change exactly (-£8,645.95, once the summary was aligned to the
    last-complete-month cutoff below), so the balance reconstruction
    itself is sound. The residual between Gap (-£4,561.76) and cash change
    (-£8,645.95), roughly **-£4,084**, breaks down as: (1) **~£3,334 net**
    transferred to Romina's own registered household accounts (£4,244 out
    to her primary, £840 back in from her secondary, £70 back from
    primary) — correctly classified as internal transfers, not spend, but
    her own spending of that money is invisible to `ledgr` — this is
    exactly the gap **Task 4 below (Manual spend entries via a proxy
    account)** exists to close, not a bug; (2) **~£271** paid down on the
    Barclaycard beyond new charges this period (£6,298.86 paid vs
    £6,027.76 newly spent) — clearing card debt faster than it's being
    run up, draining cash with no matching new spend entry; (3) **~£479
    still unaccounted for**, small enough to likely be a date-boundary
    artefact (e.g. the 1 Jan balance anchor including/excluding same-day
    transactions) rather than anything real — not chased further.
  - **Follow-up, same session: laid the summary out as two side-by-side
    columns** rather than stacked lines — `draw_gap`'s top section now
    splits horizontally (`Layout::horizontal`, no border between the two)
    into `draw_gap_income_summary` (Income/Spend/Gap, left) and
    `draw_gap_cash_summary` (Cash 1 Jan/Now/Change, right), both above the
    still-borderless month table.
  - **Follow-up, same session: surfaced the residual in the UI itself** —
    a fourth line, renamed "Untracked" after "Unexplained" (`cash_change -
    gap`), added to the Cash column (`draw_gap_cash_summary`), so the
    £4,084 finding above is visible on the screen itself rather than only
    discoverable via ad-hoc SQL. `draw_gap` now computes YTD income/spend/
    gap once and passes `gap` into both summary panels rather than each
    recomputing it. Verified live: shows "Untracked -4084.19 GBP",
    matching the SQL investigation exactly.
  - **Follow-up, same session: Gap is now the screen the app opens on**,
    not Accounts — the user wants the household finance overview first,
    not an account list. `App::new` computes the Gap screen's data
    up front (shared with `open_gap` via a new `load_gap_data(&Db)` free
    function so both stay in sync) and sets `screen: Screen::Gap`
    directly. Since this bypassed the old hardcoded "`q` quits only from
    `Screen::Accounts`, otherwise `back()`" check in `main.rs`, replaced
    it with a general `App::can_go_back()` (`!nav_stack.is_empty()`) —
    `q`/`Esc` now quits whenever there's genuinely nowhere to go back to
    (the screen the app launched into, or any screen reached with an
    empty history), regardless of which screen that happens to be, rather
    than special-casing Accounts. Help screen's `Esc / q` line reworded to
    match. Verified live via `tmux`: launches straight into "Gap";
    `<space>a` → Accounts → `q` returns to Gap → `q` again quits.
  - **Follow-up, same session: fixed month ordering across every monthly
    screen, not just Gap.** The user pointed out the Gap screen listed
    months newest-first when it should read chronologically; turned out
    all four monthly queries (`monthly_spend_totals`, `monthly_income_totals`,
    `monthly_transfer_totals`, `monthly_gap_totals`) had the same `ORDER BY
    month DESC`, so this was a display convention fix across Monthly
    Spend/Income/Transfers/Gap, not just the new screen. Flipped all four
    to `ASC` (`src/db/spend.rs`, `src/db/income.rs`, `src/db/gap.rs`).
    Since row 0 now means January instead of the most recent month,
    `open_monthly_spend`/`open_monthly_income`/`open_monthly_transfers`
    (`src/app.rs`) updated to default the selection to the *last* row
    (most recent month) instead of index `0`, so opening those screens
    still lands on current data rather than January. Verified live via
    `tmux`: Gap and Monthly Spend both read 2026-01 → 2026-07 top to
    bottom.
  - **Follow-up, same session: fixed a real column-alignment bug in the
    two summary panels.** They were hand-padded strings
    (`format!("{label:<N}{amount:>15}")`), so a label that happened to
    fully fill its padding width (e.g. "End 2026-06", exactly 11 chars)
    left less room for the amount's own right-justification than a
    shorter label did, visibly shifting that row's digits by a column
    versus its neighbours — the amount's *right* edge (the "GBP" suffix)
    stayed aligned throughout, but the *start* of the digits drifted.
    Replaced both panels' rendering with a real `Table` widget (new
    shared `draw_summary_table` helper, `src/ui.rs`) — same convention
    already used everywhere else in the app (Month table, Spend/Income
    drill-downs) — so the amount column sits at a fixed position governed
    by the `Table`'s own `Constraint`, independent of label length.
    Verified live via `tmux`: every row's "GBP" now ends at the exact
    same column in both panels.
  - 91 unit tests still passing (no new tests — same reasoning as above,
    pure aggregation); `cargo build`/`clippy` clean. Not yet committed to
    git.
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
- **Follow-up session (2026-07-19): month-by-month Gap screen extended with
  a Cash Movement column and per-account drill-down.**
  - `draw_gap_months` (`src/ui.rs`) gained a **Cash Movement** column
    (`cash_end_minor - cash_start_minor`) between Cash End and Untracked,
    and the Salary/Other columns were dropped from the table (moved to an
    on-demand popup — `i` on a selected month row shows Salary/Other/Total,
    same popup pattern as the existing transfer/income detail popups).
  - The month table is now navigable (`j`/`k`/`gg`/`G`, `TableState`),
    unlike the original non-navigable report design — `Screen::Gap`'s
    `move_selection`/`select_first`/`select_last`/`selected_row_text` arms
    updated accordingly.
  - New `Screen::GapMonth` (`Enter` on a selected Gap month row) — a
    per-account cash breakdown table (Account / Start / End / Movement,
    with a bold Total row) for that month, answering "which real account
    is this cash figure actually sitting in?". New `Db::cash_balances_by_account_as_of`
    (`src/db/gap.rs`, per-account version of the existing
    `cash_balance_as_of`) and `App::combine_account_balances` (merges the
    start/end per-account lists by name). `month_bounds` (`src/db/gap.rs`)
    made `pub(crate)` so `App::open_selected_gap_month` can share it.
  - Column header dates fixed to read as first-day-to-last-day of the
    month (`Cash Start (2026-01-01)` → `Cash End (2026-01-31)`) rather
    than the initially-shown day-before/last-day convention
    (`2025-12-31` → `2026-01-31`) — purely a display fix, the underlying
    balance is unchanged (end of 31 Dec and start of 1 Jan are the same
    instant).
  - **Real investigation, not just a UI change:** the user doubted a
    reconstructed opening balance (£5,498.73 for "Jims Premier Account"
    on 1 Jan) at first. Verified via independent raw SQL (matched the
    app's own arithmetic exactly), checked for a date-coverage gap in
    that account's transaction history (none found, no gap over 5 days
    anywhere Jan–Jul), and checked for duplicate-imported transactions
    (found two same-day/same-amount pairs, but both carry genuinely
    different bank FITIDs — real distinct transactions, not an import
    bug). Separately found real duplicate-looking rows on the
    Barclaycard account (doesn't affect cash calculations, which exclude
    `CreditCard`) — flagged for a future look, not chased further.
  - **Real discovery: the user's salary lands at the end of each month
    (28th–31st), not the 1st as assumed** — confirmed from
    `income_entries` (`rule_name = 'employment_income'`): Jan 29, Feb 27,
    Mar 31, Apr 30, May 28, Jun 29. This explains why `Cash End` snapshots
    aren't depleted: they're taken 1-3 days *after* payday, not right
    before it. For January specifically, only ~£175 of the £6,424.16
    salary had been spent by the 31st (a £500 in/£500 out roofing
    transfer nets to zero) — most of the salary is still sitting
    untouched in the `Cash End` figure.
  - 91 unit tests still passing (no new tests — no new classification
    logic); `cargo build`/`clippy` clean. Not yet committed to git.

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

### Task 5: Drive the Gap screen's "Untracked" figure to zero
- TODO — added 2026-07-18, directly off the back of Task 2's real
  finding: the Gap screen currently shows **Untracked: -£4,084.19**
  (Jan-Jun 2026), i.e. real cash left the tracked `Current`/`Savings`
  accounts that neither the spend ledger nor the income ledger explains.
  Investigated (ad-hoc SQL, see Task 2) into three components, each with
  a different fix:
  1. **~£3,334 net to Romina's own registered accounts** — correctly
     classified as an internal transfer, not spend, but her spending of
     that money is invisible to `ledgr`. Fixed by **Delta: Credit Card
     Transaction Import, Task 4 (Manual spend entries via a proxy
     account)** — once her rough monthly spend is entered manually
     against a proxy account, this portion should net out.
  2. **~£271 extra Barclaycard paydown** beyond new charges that period —
     cash spent reducing a debt that isn't tracked as a liability. Once
     credit card accounts are included in a net-worth calculation
     (extending **Task 4 above, Implement assets and liabilities as
     accounts**, to also track the `CreditCard` account type's balance
     change rather than excluding it entirely from "cash"), the money
     paying down the card should show up as a transfer to a tracked
     liability, not vanish.
  3. **~£479 still unexplained** — suspected date-boundary artefact (the
     1 Jan balance anchor's day-inclusion), not yet root-caused.
  Not yet designed: whether "Untracked" hitting exactly £0 is a realistic
  target (item 3 needs its own investigation) or whether the aim should
  just be to shrink it to something clearly attributable, with any
  residual understood rather than eliminated.

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

### Task 5: Right-align numeric column headers on the Spend/Income month drill-down screens
- ✓ DONE — `draw_spend_month` (`Screen::SpendMonth`) and `draw_income_month` (`Screen::IncomeMonth`) in `src/ui.rs` both have right-aligned numeric "Amount" data cells but a plain left-aligned text header row, so the "Amount" column header doesn't sit above its numbers. Fix: apply the same `Cell::from(Line::from(text).alignment(Alignment::Right))` pattern used in `draw_monthly_income`'s header (fixed 2026-07-18) to the "Amount" header cell in both functions.
  - Applied the same `Cell::from(Line::from(text).alignment(Alignment::Right))` header pattern to the "Amount" header cell in both `draw_spend_month` and `draw_income_month` (`src/ui.rs`).
  - Also added running totals to the top-level `draw_monthly_spend`/`draw_monthly_income` title bars (e.g. `"Monthly Spend — 42473.02 GBP"`), summed live from `app.monthly_spend`/`app.monthly_income` — verified live via `tmux`.
  - `cargo build`/`cargo clippy --all-targets` clean (same pre-existing dead-code warnings, nothing new).

### Task 6: Right-align the Monthly Transfers header row
- ✓ DONE — folded into Task 7 below since the header rework only made sense once the column design changed shape.

### Task 7: Split Monthly Transfers into In/Out/Household columns
- ✓ DONE — redesigned mid-session after the originally-planned three-column "In / Out / Household" split turned out to be structurally impossible: a `transfer_entries` row only ever gets created because `classify()` already confirmed its counterpart is internal to the household (via `household_contains()`/`matches_person_name` against tracked accounts + `config.toml`'s `household_accounts`) — so a genuinely external "left the household" counterpart can never reach this table; it falls through to the spend ledger instead (confirmed live: a real payment to Fraser Crichton correctly appears as a `person_reimbursement` spend entry, not a transfer).
- Shipped design instead: **two columns — Own / Reference — plus a Total**, right-aligned headers throughout (this also delivers the original Task 6 header-alignment goal). "Own" = transfers where both legs are the user's own tracked accounts (`out_account_id`/`in_account_id` both known) — nets to zero across total tracked cash, just money relocating between the user's own accounts. "Reference" = transfers where only one leg is known and the other side is a registered Reference Household Account (e.g. Romina's Primary/Secondary Account) that `ledgr` doesn't track a balance for — this is the real, non-zero, meaningful split (and the same money identified in Delta: The Gap, Task 2's "Untracked" investigation as leaving tracked accounts to Romina).
- New `Db::monthly_transfer_totals` (`src/db/spend.rs`) query simplified accordingly — no longer needs `household_accounts` config passed in at all, since the two-way split only needs to know whether both `transfer_entries` legs are known, not who the counterpart is.
- `MonthlyTransfer` model (`src/model.rs`) fields renamed: `own_minor`/`reference_minor` (replacing the old `transferred_out_minor`/`transferred_in_minor`).
- Screen renamed from "Monthly Transfers" to **"Monthly Inter-Household Transfers"** (`src/ui.rs`'s `draw_monthly_transfers` title, and the help screen's `<space>t` line) — the user's own naming choice, to make clear this screen only ever shows internal household movement, never money genuinely crossing the household boundary (that's the spend/income ledgers' job).
- Verified live via `tmux` against the real database: headers align correctly over right-justified figures; e.g. 2026-01 shows Own £13,271.93 / Reference £208.99 / Total £13,480.92, matching an independent ad-hoc SQL check exactly across all 7 months of real data.
- 91 unit tests still passing (no new tests — pure aggregation query, same reasoning as the Gap screen's totals); `cargo build`/`cargo clippy --all-targets` clean (same pre-existing dead-code warnings only, nothing new).
- Files changed: `src/model.rs`, `src/db/spend.rs`, `src/app.rs`, `src/ui.rs`, `src/config.rs` (factored a small `household_accounts_contain` free function out of `Config::household_account_matches` during an earlier iteration of this work, still in use there). Not yet committed to git — sitting in the working tree alongside the rest of this session's uncommitted work (income ledger, Gap screen, this transfers redesign).
- **Same-session follow-up:** renamed the "Own" column header to **"Tracked"** and added a "Household accounts" grouping label above the Tracked/Reference columns (`draw_monthly_transfers` in `src/ui.rs`, using the same inner-`Layout`-split-plus-`Paragraph`-label pattern as `draw_summary_table`) — makes clear Tracked and Reference are the two components of household-internal movement, with Total as their sum sitting apart. Verified live via `tmux`; `cargo build`/`test`/`clippy --all-targets` clean (91 tests, same pre-existing warnings only).

### Task 8: Filterable Transfers drill-down
- ✓ DONE (2026-07-19 session) — `f` on `Screen::TransferMonth` opens a live
  filter box on the bottom line (`App::transfer_filter`/`transfer_filter_editing`,
  `App::visible_transfer_entries` in `src/app.rs`) — case-insensitive
  substring match against the transfer's description or either leg's
  resolved name, so typing e.g. "romina" matches whether she's sender or
  recipient. `Enter` stops editing but keeps the filter applied (so
  `j`/`k` navigate the filtered list); `Esc` discards the filter; `Ctrl-g`
  clears it at any time. Title shows `(N of M shown)` while filtered;
  filter resets automatically when a different month is opened.
- Real bug found and fixed during implementation: the header row (e.g.
  `Start (2026-02-01)`, 19 chars) was silently clipped from the left by a
  too-narrow fixed column width — widened the affected columns.
  Verified live via `tmux`: filtering "romina" on a real month correctly
  matched both description-based and resolved-account-name-based rows.
- 91 unit tests still passing; `cargo clippy` clean. Not yet committed to
  git.

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

**No longer purely future/exploratory as of 2026-07-20** — see Task 1's
decision note below. The spend ledger design
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
- Note (2026-07-18, flagged in passing during the Gap screen session, not
  investigated): the user observed that the growing pile of config-driven
  named entities — Reference Household Accounts, Registered People,
  Income Sources, Reimbursement Sources (`config.toml`) — is starting to
  resemble a **chart of accounts** in its own right, even without formal
  double-entry postings. Worth reconsidering as evidence for/against this
  delta when this evaluation actually happens, not before. See Delta: The
  Gap, Task 2 for the full context this observation came from.
- **Concrete evidence found 2026-07-20: the Gap screen's "Untracked"
  figure structurally cannot converge to zero under the current
  cash-only model, even with 100% correct classification.** Investigating
  a real June residual (user correctly rejected a "date-boundary
  artefact" explanation — verified `cash_balance_as_of`'s reconstruction
  against a direct sum of the month's real transactions and they matched
  exactly, so the balance walk itself isn't the bug) traced most of the
  gap to credit card overpayment: £749.58 paid onto the Barclaycard in
  June vs. only £462.54 of new charges posted that month — a **£287.04**
  difference with no offsetting Spend entry, since the Spend for that
  money was already recorded whenever the original purchase happened
  (a different month, or never, if it's older principal). Asked a
  `fable`-model agent for a second opinion on the user's hypothesis
  ("if everything were accounted for, Untracked should drop to zero") —
  verdict: **false, and provably so**, not a data-quality gap. With
  perfect classification, a card charge moves Spend but not Cash
  Movement, and a card payment moves Cash Movement but not Spend/Income;
  algebraically `Untracked = card charges − card payments for the
  month = that month's Δ(credit card balance)`, identically. It hits
  zero only in the coincidental month where payments exactly equal
  same-month charges, and can flip sign (looking like mystery income in
  a month where charges exceed payments). The fix that would make "zero
  when fully classified" a true invariant: replace "Cash Movement" with
  **Net Worth Movement** — fold the credit card's balance in (as a
  negative liability) alongside current/savings in the same start/end
  total, so a charge moves Spend and net worth together and a payment
  nets to zero (cash down, liability up by the same amount). This is
  the concrete case that pushes Delta: The Gap's cash-only model towards
  this delta's territory — the user's framing: this is exactly what
  double-entry bookkeeping is *for* — making it structurally provable
  that nothing has "escaped", rather than reasoned about ad hoc per
  discrepancy.
- **Decision (2026-07-20): this evaluation is now the top priority**,
  ahead of Delta: Credit Card Transaction Import, Task 3 (partner's card
  import — deprioritised, not dropped). The user's reasoning: the
  household's finances (accounts, credit card, Reference Household
  Accounts, Registered People, Income/Reimbursement Sources) have grown
  complex enough that single-entry, ad hoc cross-checking (like the June
  Untracked investigation above) is becoming the limiting factor — every
  new discrepancy needs its own manual SQL investigation to explain,
  rather than the books structurally proving nothing escaped. Not yet
  started — next session should begin Task 1 proper: study Firefly
  III/beancount/GnuCash models, decide whether/when to adopt, ADR if
  adopted (per Task 1's original TODO below).

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
  - Real payslips/P60s already exist locally: `/Users/jmdb/Library/CloudStorage/GoogleDrive-jim.barritt@gmail.com/My Drive/Barramali/05 - Work/02 - Pleo/Payslips` (2026-07-18). Design idea from the user: a config-driven **Employer** concept (e.g. registering "Pleo" as an employer) that carries a payslip-storage directory, so once "Employer" exists as a concept it can link back to the income ledger's Income Source entries (see Delta: The Gap, Task 1's Income Source config) rather than being a fully separate mechanism.

## Delta: Reclaimable Work Expenses

Future delta, raised 2026-07-13: some day-to-day spend is money a
household member (the user or Romina) pays out of pocket but can claim
back from their employer — currently invisible as a category, no
different from ordinary discretionary spend once it lands in the spend
ledger. Sketch, not yet designed:

**Related future delta, not yet created (flagged 2026-07-18):** a genuine loan (money lent to or borrowed from the household, as opposed to a reimbursement of an already-recorded purchase) needs its own liability-side treatment distinct from both Income and Reimbursement — surfaced by a real ambiguous transaction ("Wendy Barritt LOAN ETC BGC", which turned out to actually be a reimbursement, not a loan, but the genuine-loan case is real and still unhandled). Not designed yet.
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
  bank transaction; the user's own are typically folded into his net pay with no
  separately identifiable transaction — this is the direct reason
  **Delta: Payslip Import** matters here, since only a fully parsed
  payslip could surface a reclaimed-expenses line and let this close
  the loop automatically for his own claims. However, a real counterexample
  exists (see the Correction note below).
- **A report**: some summary view (CLI or TUI, format TBD) of
  reclaimable expenses outstanding vs paid, by household member.

**Correction found 2026-07-18:** a real transaction (`2026-04-07  £49.83  "PLEO TECHNOLOGIES PLEO TECHNOLO"`, `rule_name: "direct_deposit"`) shows this delta's own assumption above doesn't always hold — this reclaim *is* a separately identifiable bank transaction from the employer, not folded invisibly into net pay as assumed. Currently misclassified as generic Income (caught by the `TRNTYPE=DIRECTDEP` fallback rule, not the `employment_income` Income Source rule, since the description starts "PLEO TECHNOLOGIES" not "AZIMO LTD" — same payroll-adjacent company, two different transaction types). Should be reclassified as a spend-ledger Reimbursement, not Income — mirrors the SimplyHealth/cashback reimbursement treatment already built (see Delta: The Gap, Task 1). Also update `## Delta: Payslip Import`'s intro paragraph if its wording asserts reclaims are *never* separately identifiable — this real example shows that assumption is at least sometimes wrong.

### Task 1: Design the reclaimable expenses ledger and marking flow
- TODO — not yet designed. Needs: (1) agreed domain term(s) — consult
  `doc/domain/ubiquitous-language.md` before naming anything (this
  delta's own title is a working name, not an agreed term); (2) schema
  for the new ledger (spend entry, household member, paid status, paid
  date/reference); (3) the TUI keypress + household-member picker on
  the spend drill-down; (4) the report view; (5) scoping what's
  buildable before Delta: Payslip Import lands vs blocked on it, given
  the asymmetric paid-back tracking above.
  - TODO — add a `reimbursement_sources`-style config entry for "PLEO TECHNOLOGIES" (distinct from the existing `income_sources` "AZIMO LTD" salary entry) so this specific transaction type routes to the spend ledger as a Reimbursement via `classify()`, not Income. Not yet implemented.

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

## Delta: Classification Rules Tidying

Technical debt flagged during the 2026-07-18 income/reimbursement classification work (Delta: The Gap, Task 1) — deliberately deferred rather than gold-plating mid-session.

### Task 1: Bundle classify()'s growing parameter list
- TODO — `classify()` in `src/derive.rs` is now at 8 parameters (`description`, `trn_type`, `amount_minor`, `household`, `household_names`, `income_sources`, `registered_people`, `reimbursement_sources`), tripping clippy's `too_many_arguments` lint — currently silenced with `#[allow(clippy::too_many_arguments)]` rather than refactored, since the "named entity" shape (Income Source / Registered Person / Reimbursement Source, and a possible future "Merchant" kind) hadn't stabilised yet. Revisit once it has: likely bundle the three config-derived slices into a single struct parameter (e.g. `NamedEntities<'a>`) passed by reference, removing the `#[allow]`. Also touches `run_derivation`'s signature and every test call site (mechanical but wide).

### Task 2: Remove throwaway diagnostic/maintenance scripts
- TODO — `examples/debug_trntype.rs` and `examples/backfill_trn_type.rs` were one-off scripts used to diagnose the real Barclays OFX TRNTYPE-not-being-persisted bug and backfill the real database; no longer needed now the backfill's done and the finding is written up in `doc/implementation-notes/spend-ledger-design.md`'s "Stale-data footgun" note. `rm` command for both already copied to clipboard (pending manual run, since `rm` is blocked in this environment) — just needs actually running.

### Task 3: Revisit Classification::Refund's hardcoded confidence
- TODO — `Classification::Refund` has no `confidence` field; every Refund-producing rule (`card_refund`, `cashback`, `claim_reimbursement`, `person_reimbursement`) gets the same hardcoded 0.7 at the insert site in `run_derivation` regardless of how confident the match actually is (e.g. a registered `SIMPLYHEALTH`/known-person match is more certain than a generic unlinked card refund). Add a `confidence: f64` field to the `Refund` variant and thread real per-rule values through, mirroring how `Spend`/`Income` already do this.

## Checkpoint: Session 2026-07-17c

**Completed:** Delta: The Gap, Task 1 — minimal income ledger, done in
full, plus the Monthly Income/Income Month TUI screens the user asked
for alongside it. See that task's entry above for the complete
write-up. Headline: `income_entries`/`income_entry_sources` (deliberately
thinner than `spend_entries` — no categorisation), `DIRECTDEP` and
Barclaycard cashback now become income instead of vanishing as
`OutOfScope`, `<space>i` opens a Monthly Income screen mirroring Monthly
Spend's design exactly. Real database backfilled and verified (3 real
cashback entries surfaced, all counts elsewhere unchanged); verified
live in `tmux`, not just `cargo test`. 86 tests passing; `cargo clippy
--all-targets` clean at the same pre-existing baseline.

**Not done:** the CLI/TUI **gap calculation itself** (Task 2 —
`income - |spend|` for a period) — income and spend both now exist as
real ledgers, so this is unblocked and is the natural next step.

**Immediate next priorities:** Delta: The Gap, Task 2 — decide CLI
(`ledgr gap`) vs TUI surface for the gap number, now both ledgers exist
to compute it from.

## Checkpoint: Session 2026-07-18

**What was completed this session:**
- Found and fixed a real income-classification bug (Delta: The Gap, Task 1): real Barclays OFX salary/BGC credits carry no TRNTYPE at all, so they were silently invisible as income. Added a NAME-suffix-based `"BGC"` rule to `classify()` in `src/derive.rs`.
- Backfilled the real database: 20 new income entries, £37,151.74 total, monthly income now realistic (£5,788-£6,574/month vs the £2.69-£25.12 the user reported).
- Added an `i` popup on the Income drill-down TUI screen showing the raw source transaction for verification, mirroring the existing Transfer drill-down popup.

**State of the project:**
The income ledger (Delta: The Gap, Task 1) is now producing correct real-world totals after this bug fix — previously it silently missed almost all real income due to a source-data quirk (Barclays omits TRNTYPE for Bank Giro Credits) rather than a code defect in the original implementation. Both the Monthly Spend and Monthly Income screens are now trustworthy enough to support Task 2 (Gap calculation). Not yet committed to git.

**Immediate next priorities:**
1. Review and commit this session's changes (`src/derive.rs`, `src/model.rs`, `src/db/income.rs`, `src/app.rs`, `src/main.rs`, `src/ui.rs`).
2. Task 2 — Gap calculation (Delta: The Gap): decide CLI vs TUI surface for `income - |spend|`, now that both ledgers have correct real data.
3. Consider whether other transaction types besides "BGC" might also be missing TRNTYPE in real Barclays exports and silently falling through classify() undetected.

## Checkpoint: Session 2026-07-18b

**What was completed this session:**
- Corrected an earlier same-day misdiagnosis: real Barclays BGC credits DO carry TRNTYPE=DIRECTDEP; the actual bug was stale pre-existing rows never getting backfilled by a normal re-import. Fixed with a one-off backfill script; real database corrected.
- Redesigned income vs. reimbursement classification after the user pushed back on treating cashback/SimplyHealth as income — spawned a fable-model agent to propose a first-principles test, adopted it, and split the domain language accordingly (new Refund/Reimbursement/Registered Person/Income Source terms).
- Built config-driven classification rules for employer salary, HMRC tax refunds, registered friends/family (Wendy, Fraser, Aria), and SimplyHealth — all reconciled against the real database, confirmed idempotent.
- Added a "Named Entities" table to `ledgr status` and month-total titles to the Spend/Income drill-down TUI screens.
- Opened a new Delta: Classification Rules Tidying for technical debt deliberately deferred mid-session (a growing `classify()` parameter list, two throwaway scripts pending removal, `Refund`'s hardcoded confidence).

**State of the project:**
The income ledger (Delta: The Gap, Task 1) now reflects a carefully reasoned income-vs-reimbursement boundary rather than a blanket "all inbound BGC money is income" rule, and the real database has been reconciled against it twice this session. Both the Monthly Spend and Monthly Income screens are trustworthy enough to support Task 2 (Gap calculation). Nothing from this session is committed to git yet.

**Immediate next priorities:**
1. Review and commit this session's changes.
2. Task 2 — Gap calculation (Delta: The Gap): decide CLI vs TUI surface, now that both ledgers have reconciled real data.
3. Delta: Classification Rules Tidying — at least Task 2 (removing the two throwaway scripts) is a five-minute cleanup worth doing soon.
4. Consider the TUI person-registration popup and review-queue mechanism the user flagged mid-session (ties into the already-planned but deprioritised Delta: Review and Re-classification TUI).

## Checkpoint: Session 2026-07-18c

**What was completed this session:**
- Added a new `IncomeSourceKind::Prize` (rule_name `prize_win`, confidence 0.9) and registered the National Lottery (Allwyn) as a Prizes-typed Income Source in the real config.
- Reworked the Monthly Income TUI screen to show Month/Salary/Other/Total columns (`MonthlyIncome` gained a `salary_minor` field, split out via `rule_name = 'employment_income'` in `monthly_income_totals`), with right-aligned column headers.
- Real database reconciled again (lottery win reclassified from `bank_giro_credit` to `prize_win`), confirmed idempotent.
- Found a real Pleo out-of-pocket-expense-reimbursement transaction misclassified as income — see the new note under Delta: Reclaimable Work Expenses.
- Flagged that the Spend/Income month drill-down screens have the same un-aligned-header issue just fixed on Monthly Income — see new Delta: TUI Analysis Views, Task 5.

**State of the project:**
This session's classification and TUI work is functionally complete and reconciled against the real database, but two follow-ups were deliberately deferred rather than implemented immediately: the Pleo reimbursement reclassification (Delta: Reclaimable Work Expenses) and the remaining header-alignment fix (Delta: TUI Analysis Views, Task 5). Nothing from today is committed to git yet.

**Immediate next priorities:**
1. Task 5 — right-align the Spend/Income month drill-down "Amount" headers (quick, five-minute fix).
2. Delta: Reclaimable Work Expenses — add the Pleo reimbursement config rule.
3. Review and commit this session's accumulated changes.

## Checkpoint: Session 2026-07-18d

**What was completed this session:**
- Delta: TUI Analysis Views, Task 5 ✓ DONE — right-aligned the "Amount" header on the Spend/Income month drill-down screens, plus added running totals to the Monthly Spend/Monthly Income screen title bars.
- Delta: The Gap, Task 1 — three more real inbound payments found mis-classified as income and fixed (Pleo expense reimbursement, GWR Delay Repay, brother's BGC payment), each registered via `reimbursement_sources`/`registered_people` config entries and re-derived idempotently against the real database.
- New TUI feature: `a` "add reference" form on the Income month drill-down screen — registers a mis-classified sender as a Registered Person and live re-derives that entry into the spend ledger as a reimbursement, without leaving the screen or needing manual config/DB edits.
- Ad-hoc SQL query computed SimplyHealth's net cost-vs-claimed position (£276 paid, £365 claimed back, net +£89 over Jan–Jun 2026); a proper annual-cap-aware version is deferred pending the user supplying the actual cap and policy year.
- Noted (not fixed): `Config::save` strips hand-written comments from `config.toml` on every write; manually restored this session, will recur.

**State of the project:**
Both the spend and income ledgers are now populated with real, correctly-classified data across Jan–Jul 2026, with income vs. refund/reimbursement correctly separated for every distinct external payer encountered so far. The TUI now supports self-service correction of misclassified income entries via the new `a` form, closing a loop the user was previously doing by hand each time. Nothing from this session has been committed to git yet — the user will commit and push it themselves.

**Immediate next priorities:**
1. Delta: The Gap, Task 2 — Gap calculation: build the first real Gap statement (income − spend) now that both ledgers are populated with real data across a meaningful date range.
2. Decide the SimplyHealth annual-cap tracking approach once the user supplies the cap amount and policy year.
3. Continue registering any further mis-classified income entries surfaced during ordinary use via the new `a` form.

## Checkpoint: Session 2026-07-19

**What was completed this session:**
- Delta: TUI Analysis Views, Task 6 + Task 7 — Monthly Transfers screen reworked into "Monthly Inter-Household Transfers" with a redesigned Own/Reference/Total column split (not the originally-planned In/Out/Household — that design was found to be structurally impossible once tested against real data, see Task 7's body for the full reasoning)

**State of the project:**
Delta: TUI Analysis Views now has only Task 2 (net worth/spending trend views) left as TODO — every other task in that Delta is done. Still nothing from this session or recent prior sessions (income ledger, Gap screen, this transfers redesign) has been committed to git; the working tree carries several sessions' worth of verified-but-uncommitted work.

**Immediate next priorities:**
1. Review and commit the working tree (income ledger + Gap screen + this transfers redesign, all uncommitted)
2. Delta: The Gap, Task 5 — drive the Gap screen's "Untracked" figure (currently -£4,084.19 Jan-Jun) to zero
3. Delta: Credit Card Transaction Import, Task 3/4 — partner's credit card import + manual spend entries via a proxy account, which Task 5 above depends on for its first component

## Checkpoint: Session 2026-07-19b

**What was completed this session:**
- Gap screen month table: added a Cash Movement column, moved
  Salary/Other into an `i`-key popup, made the table navigable, and added
  a new `Screen::GapMonth` per-account cash breakdown drill-down
  (`Enter` on a month row) with Start/End/Movement columns and a Total row
- Fixed the Gap month drill-down's date labels to read first-to-last day
  of month, and fixed a header-clipping rendering bug
- Investigated (and ruled out) data-integrity explanations for a
  seemingly-high reconstructed opening balance; discovered the user's
  real salary payday is the last working day of the month, not the 1st
- Added a live filter (`f`/`Ctrl-g`) to the Transfers drill-down screen,
  matching on description or either leg's resolved account name
- Built Delta: Credit Card Transaction Import Task 4 (manual spend
  entries via a proxy account) — `s` on a Transfers-drill-down row opens
  a form to record what money sent to a Reference Household Account was
  actually spent on; verified end-to-end against real data, then cleanly
  removed the test entry

**State of the project:**
The Gap screen is now a genuine reconciliation tool: month-by-month cash
movement, per-account drill-down, and a working mechanism (manual spend
entries) to close the gap between what the ledgers explain and what
actually happened to the household's cash. Task 5 (driving Untracked to
zero) has real data-backed next steps rather than open questions — the
tool to close the largest component (~£3,334 net transferred to Romina)
now exists, it just needs the user to work through the remaining months.
91 unit tests passing throughout; nothing from this session committed to
git yet.

**Immediate next priorities:**
1. Use the new `s` (record spend from transfer) feature to log Romina's
   real holiday/other spending across the months with a net transfer to
   her, closing out that component of Untracked
2. Investigate the remaining ~£479 unexplained residual (suspected
   date-boundary artefact, not yet root-caused)
3. Delta: The Gap, Task 4 (assets/liabilities as accounts) — once built,
   extend to track `CreditCard` balance change so Barclaycard paydown
   beyond new spend stops showing as unexplained Untracked
4. Commit this session's and the prior sessions' uncommitted work to git

## Checkpoint: Session 2026-07-20

**What was completed this session:**
- No code changes — the user tried the newly-built "record spend from
  transfer" feature (`s` on the Transfers drill-down) against real data
  and reported four issues/requests, captured as Delta: Credit Card
  Transaction Import, Task 6

**State of the project:**
The spend-from-transfer feature works but has rough edges surfaced by
real use: the Gap screen doesn't refresh after using it (misleading
until a manual reload), there's no way to trace a manual spend entry back
to its originating transfer for correction, notes are only supported on
spend entries (not income or transfer entries), and the Transfers
drill-down gives no visual indication of which transfers already have a
recorded spend against them. None of these are implemented yet.

**Immediate next priorities:**
1. Fix the Gap screen's stale-data-on-back() bug (Task 6, item 1) —
   likely the smallest, most isolated fix
2. Design and build the spend-entry-to-transfer-entry link (Task 6, item
   2) — a prerequisite for item 4
3. Add the "Tracked Spend" column + linked-spend popup to the Transfers
   drill-down (Task 6, item 4)
4. Extend note support to income and transfer entries (Task 6, item 3)

## Checkpoint: Session 2026-07-20b

**What was completed this session:**
- Delta: Credit Card Transaction Import, Task 6 (Spend-from-transfer
  follow-ups) — all four issues fixed: Gap screen now refreshes on
  `back()`; spend entries link back to their originating transfer
  (`spend_entries.transfer_entry_id`); all three entry types (spend,
  income, transfer) now support an editable note via the same `n` key;
  a new "Tracked Spend" column plus a linked-spend line in the transfer
  detail popup on the Transfers drill-down. Two new schema migrations
  (`migrate_add_spend_entries_transfer_entry_id_column`,
  `migrate_add_transfer_entries_note_column`). Full detail under Task 6
  above.

**State of the project:**
The spend-from-transfer feature (Task 4/5's follow-on) is now complete
end-to-end: recording a manual spend from a transfer is immediately
visible everywhere it should be (Gap totals, the Transfers drill-down's
Tracked Spend column, the transfer detail popup) without restarting the
app or leaving the screen. Notes are now a consistent feature across all
three ledgers rather than spend-only. A meaningful backlog of committed-
but-not-yet-`git commit`ed work has built up across several recent
sessions (see repeated "not yet committed to git" notes throughout this
plan) — worth committing soon to avoid losing the thread of what's
changed.

**Immediate next priorities:**
1. Commit this session's and the prior sessions' uncommitted work to git
2. Delta: Credit Card Transaction Import, Task 3 — import the user's
   partner's credit card (needs her card registered as a household
   account, same identity-tracking mechanism as Task 1)
3. Delta: The Gap, Task 4/5 — assets/liabilities as accounts, and driving
   Untracked to zero (would resolve the remaining Barclaycard-paydown and
   ~£479 residual noted in Task 2's real-data investigation)
4. Delta: Spending Categorisation, Task 1 — confirm the Rebel Finance
   taxonomy (currently IN PROGRESS, blocking Tasks 2/3 of that Delta)

## Checkpoint: Session 2026-07-20c

**What was completed this session:**
- Real data-quality fix, no new code — the user reported none of their
  real spend-from-transfer entries showed as Tracked Spend after Task 6
  landed. Root cause: those 16 entries were created via `s` earlier the
  same day, before Task 6's `transfer_entry_id` column existed, so there
  was nothing to link at creation time (confirmed via each entry's
  `classified_at` timestamp — 2026-07-20T18:04–18:14Z). Backfilled all 16
  by matching each to its transfer on exact date + amount (unambiguous in
  every case); real `ledgr.db` backed up first
  (`ledgr.db.bak-20260720201726-pre-transfer-entry-id-backfill`). Full
  detail appended to Task 6 above.

**State of the project:**
Task 6 (Spend-from-transfer follow-ups) is now fully complete, including
this backfill — every existing manual spend entry and every future one
correctly shows as Tracked Spend. Nothing else changed this session; the
next-priorities list from the prior checkpoint still stands.

**Immediate next priorities:**
1. Commit this session's and the prior sessions' uncommitted work to git
2. Delta: Credit Card Transaction Import, Task 3 — import the user's
   partner's credit card
3. Delta: The Gap, Task 4/5 — assets/liabilities as accounts, and driving
   Untracked to zero
4. Delta: Spending Categorisation, Task 1 — confirm the Rebel Finance
   taxonomy (currently IN PROGRESS)

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
