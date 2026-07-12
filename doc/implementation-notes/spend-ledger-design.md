# Spend ledger — design

Design session notes, 2026-07-11. Companion ADR:
[0005 — Independent spend and income ledgers](../adr/0005-independent-spend-and-income-ledgers.md).
Grounding research: [Rebel Finance categorisation](../kb/rebel-finance/research.md)
and [OFX statement structure](../kb/ofx/structure.md).

## Summary

- **The problem is real**: a single purchase paid by card produces
  three raw transactions (CC merchant line, current-account debit,
  CC repayment credit). Counting all three as spend triples it, so
  spend needs to be a *derived* view, not the raw transaction table.
- **Core mechanism**: raw `transactions` never change. A derivation
  pass reads them and writes `spend_entries` (the categorised,
  human-facing ledger) plus `spend_entry_sources` (which raw rows an
  entry came from). Internal transfers between the household's own
  accounts produce no ledger entry at all — just a link between the
  two raw transactions.
- **Why transfers are detectable at all**: Barclays never marks a
  transfer as a transfer (`TRNTYPE=XFER`); instead both sides of a
  same-day, equal-and-opposite transfer encode the other account's
  sort code + account number in the `NAME` field. So "is this
  internal" is answered by checking the counterparty account against
  a known list of the household's own accounts — deterministic, not
  a heuristic.
- **Three things bolted onto that core**, in roughly descending order
  of necessity for a working v1:
  1. *Classification provenance* (rule/matcher/manual + confidence) —
     needed from day one so manual corrections aren't clobbered by
     re-running derivation.
  2. *Refund linking* (`transaction_links relation='refund'`) — a
     second amount/date matching pass, separate from transfer pairing.
  3. *Spend enrichment* ("soy sauce" flow) — a **third** amount/date
     matching pass that copies a transfer's reference text onto the
     spend entry it's judged to annotate. This is the most speculative
     piece: it's a fuzzy, confidence-scored guess, not deterministic
     like the transfer-pairing rule above it, and it's pure UX polish
     (better descriptions) rather than correctness (spend/not-spend,
     amounts). **Worth building last, or deferring past the initial
     Task 2 implementation** — the ledger is fully correct and usable
     without it.
- **Also present but low-cost**: an `income_entries` design (deferred,
  not implemented — just documented so the split is intentional) and a
  "Future: double-entry compatibility" section (notes only, no code
  implication now).
- **Verdict**: the core (immutable raw + derived spend ledger +
  deterministic transfer detection + provenance) is proportionate to a
  genuine data problem, not overengineering — it's grounded in patterns
  found in 939 real transactions, not hypothetical. The one part that
  does look like scope creep for a first implementation is spend
  enrichment's separate fuzzy-matching pass; suggest phasing Task 2 as
  schema + rules + transfer pairing + refund linking first, spend
  enrichment as a follow-up once the core ledger is in daily use.

## Problem

Raw imported transactions mix two fundamentally different things:

1. **Real-world money events** — money leaving the household to a
   merchant or person (spend), or entering it (income). These are what
   the Rebel Finance method categorises, and what the "gap"
   (income − spending) is computed from.
2. **Internal transfers** — money moving between the user's own
   accounts (current account → credit card to pay for a purchase,
   bills account → spending account to fund one, top-ups to savings).
   Counting
   these as spend double-counts every purchase that is paid for by a
   matching transfer.

The user's real workflow makes this concrete: a £10.56 Amazon purchase
on the credit card appears as *three* raw transactions — the CC line
("AMAZON 10.56"), the current-account debit
(`<sort code> <acct no> \tSOY SAUCE FT` −10.56), and the CC credit
("PAYMENT, THANK YOU"). Exactly one real-world spend happened, and the
best human description of it ("soy sauce") is on the *transfer*, not
on the merchant line.

Direct spending happens from a small set of accounts (two or three
current accounts and the credit card); everything else is internal
movement.

## Core design

**Raw transactions are immutable evidence; the ledgers are derived.**

```
imports ──▶ transactions (raw, never edited, deduped by FITID)
                    │
                    │  derivation pass (rules + transfer matching)
                    ▼
        ┌──────────────────────┬──────────────────────┐
        ▼                      ▼                      ▼
  spend_entries          income_entries         (internal transfers:
  ◀── categorisation,    (salary, interest,      no entry in either —
      budgets live here   cashback)              transaction_links only)
        │                      │
        └── *_entry_sources ──▶ transactions
```

- **Two independent derived ledgers** (decision + rationale in ADR
  0005): the **spend ledger** (`spend_entries`) — the whole domain of
  classifying and categorising spend and working to budgets — and the
  **income ledger** (`income_entries`) — a separate, much simpler
  domain that will eventually touch salary, tax, and pensions. They
  share derivation *mechanics* by convention, not a table. Internal
  transfers produce entries in **neither** ledger — they are recorded
  only as pairings between raw transactions (`transaction_links`,
  `relation = 'transfer'`).
- Only spend entries are categorised. Raw transactions never get a
  category. (The existing `transactions.category_id` column becomes
  redundant and should be dropped from the schema.)
- Provenance is an edge table, so one entry can draw on several raw
  transactions (CC line + annotating transfer) and — later — one raw
  transaction can explode into several entries (Amazon order import
  splitting a lump charge into line items).

### Scope: spend ledger first

A further consequence of the split: the income ledger can be deferred
entirely. Spend is what we're actually interested in right now, so this
design and its implementation delta cover the **spend ledger only** —
income-looking raw transactions (e.g. `DIRECTDEP` salary credits) are
simply left alone by the derivation pass until the income ledger gets
its own design. Nothing about the split requires building both.

```sql
CREATE TABLE spend_entries (
    id             INTEGER PRIMARY KEY,
    occurred_on    TEXT NOT NULL,           -- date of the real-world spend
    amount_minor   INTEGER NOT NULL,        -- signed, same convention as
                                            -- transactions (negative = out);
                                            -- a refund is a positive entry
    currency       TEXT NOT NULL,
    counterparty   TEXT,                    -- merchant or person, normalised
    description    TEXT NOT NULL,           -- best human description
    note           TEXT,                    -- user's own note, e.g. the
                                            -- transfer reference ("SOY SAUCE")
    category_id    INTEGER REFERENCES categories(id) ON DELETE SET NULL,

    -- classification provenance (applies to counterparty and category)
    classified_by  TEXT NOT NULL CHECK (classified_by IN
                       ('rule', 'matcher', 'manual')),
    confidence     REAL,                    -- NULL when manual
    rule_name      TEXT,                    -- which rule/matcher fired
    classified_at  TEXT NOT NULL
);

-- Which raw transaction(s) an entry derives from.
CREATE TABLE spend_entry_sources (
    spend_entry_id INTEGER NOT NULL REFERENCES spend_entries(id)
                       ON DELETE CASCADE,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id)
                       ON DELETE CASCADE,
    role           TEXT NOT NULL CHECK (role IN (
                       'source',      -- the raw row the entry represents
                       'annotation'   -- e.g. matched transfer carrying the note
                   )),
    UNIQUE (spend_entry_id, transaction_id)
);
```

The income ledger (`income_entries` + `income_entry_sources`) will
follow the same conventions when designed, but is out of scope here.
Once it exists, the Rebel Finance gap for a period is
`SUM(income_entries) − |SUM(spend_entries)|`.

### Account registry (household accounts)

Transfer detection needs to know the household's accounts (see the
ubiquitous language doc: the **household** is the accounting entity;
transfers within it are internal). Barclays
OFX gives each file's identity via `BANKACCTFROM` (`BANKID` = sort
code, `ACCTID` = account number), and names the counterparty of every
faster-payment transfer as `<sort code> <account no>` inside `NAME`
(it never emits the spec's `BANKACCTTO` — see the OFX KB article).

- Add `sort_code` and `account_number` columns to `accounts`, populated
  from `BANKACCTFROM` on import.
- Additionally, **optional config** lists the specific accounts of
  other household members (sort code + account number), so transfers
  between household members are recognised as internal rather than
  spend. Imported accounts are household members automatically; the
  config only needs the not-imported ones.
- No account-type pre-filter: derivation scans every account uniformly
  regardless of type. Transfer pairing/reconciliation (rules 1-2 below)
  is what keeps internal movement out of the ledger, not a check against
  a fixed set of "spending account" types — see ADR 0006.

### Derivation rules (per raw transaction)

Evidence for these patterns: 939 real transactions analysed in the
[OFX KB article](../kb/ofx/structure.md). Barclays never emits
`TRNTYPE=XFER`; transfers arrive as `OTHER` and must be recognised
from `NAME`.

Rules are evaluated **in this order**, top to bottom, first match wins.
This matters because `TRNTYPE` alone is not a reliable discriminator —
e.g. a standing order (`REPEATPMT`) into the user's own savings account
would otherwise match the "card payment / DIRECTDEBIT / PAYMENT /
REPEATPMT to a merchant" row below, misclassifying an internal transfer
as spend. The `NAME`-prefix account check must run first, regardless of
`TRNTYPE`, so it always wins the overlap.

| # | Raw pattern | Classification |
|---|---|---|
| 1 | `NAME` starts `<sort code> <acct no>` and that account ∈ household accounts | **Internal transfer** — no ledger entry; pair with counterpart if imported (`transaction_links`) |
| 2 | `NAME` starts `<sort code> <acct no>` and account unknown | Payment to an external account — **spend** if outbound, low confidence, review; inbound left for the income ledger |
| 3 | `NAME` = `<PERSON> \t<ref> FT` (no account visible) | **Spend** to a person (window cleaner, family) if outbound; inbound: reimbursement (see open questions) |
| 4 | Card payment (`ON <dd MMM> CPM`, `DIRECTDEBIT`, `PAYMENT`, `REPEATPMT` to a merchant) | **Spend** |
| 5 | Card refund (`CRM`/`CRE`/`BCC` suffix, positive amount) | **Refund** — positive-amount spend entry, linked to the original entry via `transaction_links` `relation='refund'` when findable |
| 6 | `DIRECTDEP`, other inbound credits from external parties | Income — **out of scope**, no spend entry; left untouched until the income ledger exists |
| 7 | `TRNTYPE=CASH` (cash withdrawal) | **Out of scope for now** — cash leaves the tracked boundary but what happens to it afterwards is invisible; revisit if it becomes material (1/939 observed transactions) |
| 8 | CC statement `Payment received` | Internal transfer (the credit side of a CC repayment from a current account) — no entry |
| 9 | CC statement `Purchase` | **Spend** |
| 10 | CC statement `Other` (e.g. Barclaycard Cashback) | Income — out of scope, review |

Note on rows 1–3: `NAME` is hard-capped at 32 characters (see the OFX
KB article), and the sort-code/account-number prefix eats into that
budget, so the free-text reference portion can be truncated (e.g.
"LAWNMOWER REPAIRS" arrives as "LAWNMOWER REPAI"). Reference-text
matching — both the transfer-pairing tie-break below and spend
enrichment — must tolerate truncation (prefix match) rather than
require exact equality.

Confidence: deterministic patterns (own-account match, card-payment
suffix, direct debits) get high confidence; person payments and
unknown-account transfers get lower confidence and surface in the
review queue.

### Transfer pairing

Both sides of an own-account transfer carry the *same user reference*,
each naming the other account (verified in real data; fictional
example: account A shows `209912 12345678 \tPIZZA OVEN FT` −89.00
while account B shows `209934 87654321 \tPIZZA OVEN FT` +89.00).
Pairing algorithm:

1. Candidate = equal-and-opposite `amount_minor`, counterparty account
   numbers pointing at each other's account, within a small date window
   (transfers are usually same-day; allow ±3 days for weekends).
2. Tie-break on matching reference text.
3. Record as `transaction_links (relation='transfer')` with a
   confidence; unpaired internal-looking transfers (counterpart file
   not yet imported) still classify as internal by the own-account rule
   alone.

### Spend enrichment (the "soy sauce" flow)

When a spend entry's source is a CC/merchant line and an internal
transfer of the same amount (within a date window) carries a user
reference, attach that transfer as an `annotation` source and copy its
reference into `ledger_entries.note`. The entry's `counterparty` stays
the merchant ("AMAZON"); the note carries the human meaning ("SOY
SAUCE"). Amount-based matching is approximate — annotation links are
suggestions with a confidence, correctable in the review UX.

Where spending happens *directly* from a current account via faster
payment, the transfer reference **is** the description — a payment to
a window cleaner with reference `WINDOW CLEAN FT` becomes a spend
entry with note "WINDOW CLEAN". But when the reference sits on an
*internal* transfer (bills account → spending account, "PIZZA OVEN"),
the transfer itself is not the spend — the *actual* spend is the later
card/merchant charge from the receiving account. The internal
transfer's reference can still be propagated as an annotation onto
that eventual spend by amount match.

## Data-quality constraint: Barclaycard CSV rounds amounts

The credit card's only export today is CSV
(`Date, Account/Card No, Amount, Subcategory, Memo`) and **every
amount is rounded to whole pounds** (all 205 rows end `.00`, while the
matching repayment transfers on the current-account side carry penny
precision — so it is the CSV export rounding, not the underlying
data). Consequences:

- CC-derived spend entries are approximate to ±£0.50 until a better
  source exists. Flag them (e.g. `confidence` reflects amount
  imprecision, or an explicit `amount_approximate` flag — decide during
  implementation).
- Exact-amount transfer pairing (current account ↔ CC repayment) must
  use a tolerance for the CC side, or match on the bank side only.
- Worth investigating whether Barclaycard offers a precise export
  elsewhere (PDF statements carry exact amounts; there may be an
  OFX/QIF option on a different screen). Until then the CSV is still
  worth importing — categories and merchants are right even if pennies
  are not.
- Other CSV quirks for the parser: UTF-8 BOM, `DD/MM/YYYY` dates,
  thousands separators (`-1,378.00`), sign convention inverted relative
  to bank statements (purchases positive, repayments negative), memos
  containing embedded tabs and newlines, masked card number
  (`************0002`) usable as the account identity.

## Re-classification UX and idempotent re-derivation

- Derivation is a re-runnable pass, run as part of `ledgr import` (for
  now — may also warrant a standalone `ledgr derive` later): for each
  raw transaction not yet linked to a ledger entry (or whose rules
  have changed), produce/update entries.
- **Manual always wins.** `classified_by='manual'` entries (and manual
  category assignments) are never overwritten by re-derivation; rule
  and matcher results are refreshable.
- TUI: a review queue screen filtered to low-confidence/uncategorised
  entries; single-key actions on any spend-ledger row to mark as
  internal transfer (removes the entry, records the link), mark as
  not-spend (e.g. actually income — removes the entry), set category,
  or edit the note. Every manual action stamps
  `classified_by='manual'`.

## Decisions taken this session

1. **Derived ledgers, not flags on raw transactions** — raw stays
   immutable evidence; the ledgers are rebuildable.
2. **Independent spend and income ledgers, not one table with a
   `kind`** — a single table was chosen first but resisted naming,
   which exposed that spend (classification, categories, budgets) and
   income (salary, tax, pensions) are different domains, not two kinds
   of one thing. See ADR 0005 for the full rationale. Only the spend
   ledger is being built now; income is deferred.
3. **Classification provenance is first-class** — every entry records
   whether it was classified manually or automatically, by which rule,
   and at what confidence.
4. **Transfer detection is deterministic where possible** — own-account
   registry + Barclays' `NAME` encoding, not heuristics-first.

## Future: double-entry compatibility

Double-entry accounting may be introduced later (tracked as its own
exploratory delta in the plan). This design maps onto it cleanly, so
nothing here should be built in a way that blocks it:

- Raw transactions remain the imported evidence layer (as in
  Firefly III / beancount importers).
- A spend entry becomes a posting to an *expense account*; the
  category hierarchy is an expense-account chart of accounts in
  waiting. An income entry becomes a posting from a *revenue account*
  — the independent-ledgers split (ADR 0005) mirrors the structural
  expense/revenue distinction.
- An internal transfer becomes the definitional double-entry case: a
  transaction whose both legs are household asset/liability accounts.
- The household-accounts membership list would become structural
  (asset/liability account typing) rather than config.

## Open questions

1. ~~Household membership~~ — **decided**: optional config listing the
   specific accounts of other household members (see Account registry
   above), so transfers between household members are not treated as
   spend.
2. ~~Inbound reimbursements~~ — **decided**: the concept is
   **Reimbursements and Refunds** (see the ubiquitous language doc) —
   inbound money paying back earlier spend is a sign-reversed
   spend-ledger entry linked to the original via
   `transaction_links (relation='refund')` where findable; never
   income. (From household members it's an internal transfer anyway.)
3. ~~Sinking funds~~ — **decided**: spend happens only when money
   actually leaves the household, so the *purchase* convention holds;
   transfers into savings pots are internal. Recorded under **Spend**
   in the ubiquitous language doc.
4. ~~Where does derivation run~~ — **decided (provisionally)**: as part
   of `ledgr import`. A standalone `ledgr derive` may be added later if
   needed (e.g. after rule changes without a new import).
5. ~~Precise CC data~~ — **in hand**: the rounded CSV is not good
   enough; the user will try Barclaycard's PDF statement export
   instead (task added to the Credit Card Transaction Import delta).
6. **Category taxonomy** — still tracked separately in the Spending
   Categorisation delta (Rebel Finance ~10 categories, user's
   spreadsheet to cross-check). The ledger design is taxonomy-agnostic:
   categories attach to ledger entries.
