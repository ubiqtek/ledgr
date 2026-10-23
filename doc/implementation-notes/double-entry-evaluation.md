# Double-entry accounting evaluation

Delta: Double-Entry Accounting, Task 1 — evaluation of the Firefly III,
beancount, and GnuCash models against ledgr's needs, and a
recommendation. Companion to the plan's Task 1 decision notes
(2026-07-20) and `spend-ledger-design.md`'s "Future: double-entry
compatibility" section.

## The problem this solves

The Gap screen's "Untracked" figure structurally cannot converge to
zero under the current cash-only model, even with perfect
classification. A credit card charge moves Spend but not Cash Movement;
a card payment moves Cash Movement but not Spend or Income. So for any
month, `Untracked = card charges − card payments = Δ(credit card
balance)`, identically — it hits zero only when payments coincidentally
equal same-month charges, and flips sign when charges exceed payments
(mystery "income"). The June 2026 residual (£287.04 of Barclaycard
overpayment) is the proven concrete case; the £271 component of the
January–June £4,084 residual is the same effect over a longer window.

The property Jim actually wants is: **when everything is classified,
the books prove nothing escaped — structurally, not by ad-hoc SQL
investigation per discrepancy.** Every new residual currently costs a
manual investigation to explain. Double-entry bookkeeping is the
mechanism whose entire purpose is making that property an invariant:
every transaction balances to zero across accounts, so any unexplained
residual is *locatable* (it sits in a specific account) rather than a
free-floating difference between two independently computed totals.

Two fixes exist at different depths:

1. **Net Worth Movement (read-side fix, no double-entry).** Replace
   "Cash Movement" with a movement figure that folds the credit card
   balance in as a negative liability alongside current/savings. A
   charge then moves Spend and net worth together; a payment nets to
   zero (cash down, liability down by the same amount). This makes
   "Untracked → 0 when fully classified" true *for the credit card
   case*, with no schema change — it is correct algebra on data ledgr
   already has (card transactions are imported; `balance_as_of` works
   for the card account).
2. **A double-entry core (structural fix).** Make balancing an
   invariant of the data model itself, so the same guarantee holds for
   every future case (mortgage principal vs interest, pension
   contributions, Romina-account outflows, manual proxy spend) without
   each needing its own bespoke correction like fix 1.

Fix 1 is worth doing regardless — it is cheap and repairs the visible
number now. The rest of this document is about whether fix 2 is worth
its cost, and in what form.

## The three candidate models

### Firefly III

- **What it is:** a self-hosted PHP/Laravel web application with a
  relational database; an app to run, not a library or format.
- **Data model:** a `TransactionJournal` (one financial event) owning
  transaction rows that link to accounts. Four transaction shapes:
  *withdrawal* (asset → expense), *deposit* (revenue → asset),
  *transfer* (asset/liability → asset/liability), plus opening
  balances. Asset and liability accounts carry a running balance;
  expense and revenue accounts are period-summed sinks/sources.
- **Liability handling:** credit cards are liability accounts; a card
  purchase is a withdrawal from the liability to an expense account,
  and paying the card is a transfer from an asset account to the
  liability — exactly the shape that makes the Untracked problem
  disappear (the payment nets to zero against net worth).
- **But it is not true double-entry.** Transactions are constrained to
  rigid patterns — a withdrawal may split its destination but not its
  source; a deposit must land in exactly one asset account; transfers
  are strictly one-to-one. A transaction touching several accounts on
  both sides (the canonical example: a payslip with gross salary,
  tax, NI, and pension legs) cannot be expressed as one balanced entry
  and needs a chain of workaround transactions through an intermediate
  account. This matters directly for ledgr: Delta: Payslip Import is
  precisely the multi-leg case Firefly's model cannot represent
  cleanly, and it is already on the plan.
- **Fit with ledgr:** none as a dependency (PHP web app; adopting it
  means replacing ledgr, abandoning the local-TUI/SQLite model and the
  classification machinery). As a *design reference* its main value is
  negative evidence: it shows that a source/destination model with
  typed accounts gets you liability handling, but that hard-coding
  two-account transaction shapes is a ceiling ledgr would hit at
  payslips.

### beancount

- **What it is:** a plain-text accounting language and Python toolchain.
  The file is the database; tools parse and re-derive everything.
- **Data model:** the cleanest of the three. Dated immutable
  directives; the central one is a *Transaction* holding two or more
  *Postings*, each posting an (account, amount) pair, with the
  invariant that postings sum to zero. Accounts are hierarchical
  colon-separated names rooted in exactly five types: **Assets,
  Liabilities, Equity, Income, Expenses**. Two further directives map
  strikingly well onto things ledgr already has: *balance assertions*
  ("this account held X on this date" — a checkpoint the tool verifies
  against the journal, catching missing/duplicate entries) are
  conceptually identical to ledgr's `balance_snapshots` anchors, and
  the importer convention (raw bank data → generated directives,
  original files kept as evidence) matches ledgr's raw-transactions-
  are-immutable-evidence principle.
- **Liability handling:** structural, not special-cased. The card is
  `Liabilities:Barclaycard`; a charge posts `Expenses:… +X` /
  `Liabilities:Barclaycard −X`; a payment posts
  `Liabilities:Barclaycard +X` / `Assets:Current −X`. Net worth
  (Assets + Liabilities) moves with the charge and is untouched by the
  payment. The Untracked invariant holds by construction for *any*
  account, not just cards.
- **Fit with ledgr:** the Python toolchain is a non-starter to embed
  in a Rust binary. Rust reimplementations exist —
  [rustledger](https://github.com/rustledger/rustledger) (a
  beancount-compatible engine with parsing, validation, and an import
  pipeline), the `beancount-parser` crate, and
  [twilco/beancount](https://github.com/twilco/beancount)
  (core/parser/render crates) — but they are all built around the
  plain-text file as the source of truth, which conflicts with ADR
  0009 (persisted ledgers in SQLite, built at import, UI queries them).
  Adopting the *file format* would mean either abandoning SQLite as
  the store or maintaining two sources of truth. Adopting the *model*
  — five account types, balanced postings, balance assertions —
  requires no dependency at all: it is a schema shape. beancount is
  the strongest design reference of the three, and a beancount-format
  *export* would be a cheap future feature (giving access to Fava and
  the whole plain-text-accounting toolchain for free) without any
  structural commitment.

### GnuCash

- **What it is:** a mature desktop accounting application (C/C++,
  ~25 years old) with an optional SQL backend (SQLite/MySQL/Postgres).
- **Data model:** five core objects — Book, Commodity, Account,
  Transaction, **Split**. A transaction contains two or more splits;
  each split ties an amount to exactly one account; splits within a
  transaction must sum to zero. Accounts form a typed tree (asset,
  liability, income, expense, equity, plus subtypes like credit card
  and bank). This is beancount's model in relational-schema form —
  and GnuCash's SQL schema (a `splits` table at the centre, foreign
  keys to `transactions` and `accounts`) is the closest existing
  artefact to what a double-entry ledgr schema would look like in
  `schema.sql`.
- **Liability handling:** same structural story as beancount — credit
  cards are liability-type accounts, purchases post
  expense-vs-liability, payments post asset-vs-liability.
- **Fit with ledgr:** not embeddable (a GUI application, not a
  library; the Rust bindings ecosystem is thin and read-oriented).
  Its value is as proof that the split-table double-entry model works
  in exactly ledgr's storage substrate (SQLite) at real-world scale,
  and as a schema reference (via
  [piecash's object-model docs](https://piecash.readthedocs.io/en/master/object_model.html)).

### Comparison summary

| | Firefly III | beancount | GnuCash |
|---|---|---|---|
| Core unit | Journal + source/dest transactions | Transaction + ≥2 postings | Transaction + ≥2 splits |
| True balancing invariant | No (fixed two-sided shapes) | Yes | Yes |
| Account typing | Asset/Liability/Expense/Revenue | Assets/Liabilities/Equity/Income/Expenses | Typed tree, same five roots |
| Credit card fix | Yes (liability accounts) | Yes, structural | Yes, structural |
| Multi-leg payslip | Cannot express as one entry | Natural | Natural |
| Embeddable in Rust | No (PHP app) | Model yes; toolchain no (Rust ports exist but file-centric) | No (C++ app; schema reusable) |
| Adoption mode for ledgr | Replace ledgr — rejected | Adopt the *model*, optionally export the format later | Adopt the *schema shape* |

## The minimal internal model

The smallest change that makes "Untracked is zero when fully
classified" a structural invariant, building on
`spend-ledger-design.md`'s existing mapping:

**One new derived layer: a journal.** Two tables —
`journal_entries` (id, date, description) and `postings`
(journal_entry_id, account_id, amount_minor) — with the invariant
that each entry's postings sum to zero, enforced at derivation time
(and checkable by a single SQL query at any time). `accounts` gains a
five-way type dimension (Assets / Liabilities / Equity / Income /
Expenses) subsuming the existing `AccountType`; expense categories and
income sources become accounts in the same table rather than a
separate taxonomy.

**The journal is derived, not the source of truth.** This is the key
adaptation that keeps ledgr's existing principles intact rather than
importing beancount's wholesale:

- Raw transactions remain immutable evidence (unchanged).
- The derivation pass (`run_derivation`) currently decides, per raw
  transaction, "spend / income / transfer / refund / out of scope".
  Under the journal model it decides the same thing, expressed as:
  *which contra account balances this transaction's posting?* The bank
  leg is mechanical (the imported account, the imported amount); the
  contra leg — `Expenses:Groceries`, `Income:Salary`,
  `Assets:Savings` (a transfer), `Liabilities:Barclaycard` (a card
  payment) — is exactly what `classify()` already computes. A matched
  transfer's two raw transactions derive one journal entry with two
  asset/liability postings, precisely as a transfer entry pairs them
  today.
- **All the classification machinery survives unchanged in role**:
  rules, confidence scores, `classified_by`/`rule_name` provenance,
  manual-wins, config-driven Income Sources / Registered People /
  Reimbursement Sources. None of the three studied systems has an
  equivalent of this layer (beancount importers are typically
  hand-maintained matcher scripts; Firefly has crude rules; GnuCash
  has bayesian account suggestion) — it is ledgr's genuine value-add
  and sits *above* a double-entry core, deciding contra accounts,
  rather than being displaced by it. Provenance moves to the journal
  entry (or per-posting) rather than the spend/income entry.
- An unclassified transaction still gets a balanced entry, with its
  contra posting to a dedicated suspense account (working name only —
  the term must go through the ubiquitous-language process).
  **"Untracked" then stops being a computed residual between two
  totals and becomes the balance of that account** — a figure that is
  zero exactly when everything is classified, by construction, and
  that *lists its own contents* when it isn't. That is the structural
  version of what the June investigation did by hand.
- `spend_entries` / `income_entries` / `transfer_entries` become
  projections over the journal (or are retained as-is during a
  transition, cross-checked against it). ADR 0005's insight — spend
  and income are different *domains* with different workflows — is
  untouched: the Expenses and Income subtrees of the account chart are
  exactly that separation, and the TUI screens keep querying
  domain-shaped views. ADR 0009 (persisted at import, UI only
  queries) carries over to the journal directly.
- `balance_snapshots` become balance assertions: reconstructing an
  account's journal balance at an anchor date and comparing it to the
  bank-reported figure is a *verification* of the journal against
  external truth — a strictly stronger version of what
  `cash_balance_as_of` reconstruction does today, and it localises
  discrepancies to one account and date range.

**What the minimal model deliberately leaves out:** multi-currency
(GBP only, as now); accrual concepts (cash-basis throughout, per ADR
0005's rejected receivables ledger); equity beyond a single
opening-balances account (needed so pre-history balances have a contra
side); plain-text file format (SQLite remains the store; beancount
export is a possible later feature, not part of this).

### How existing domain vocabulary maps

| Term | Under the journal model |
|---|---|
| Account / Tracked Account | Unchanged — an Assets (or Liabilities, for the card) account with imports |
| Transaction | Unchanged — immutable evidence below the journal |
| Spend Entry | Projection of a posting to an Expenses account |
| Income Entry | Projection of a posting from an Income account |
| Transfer Entry | A journal entry whose postings are all asset/liability accounts — the definitional double-entry case |
| Credit Card Payment | Assets ↔ Liabilities entry; nets to zero against net worth (the fix) |
| Refund / Reimbursement | Negative posting to an Expenses account; the Refund's link to the original entry survives as entry-level metadata |
| Category | An Expenses sub-account; the taxonomy becomes a subtree of the chart of accounts |
| Reference Household Account | An Assets account with no imports — membership becomes structural typing, as `spend-ledger-design.md` predicted |
| Household Accounts | The set of asset/liability accounts — a query, not a config list |
| Proxy Account / Manual Spend Entry | An Assets account whose entries are manually derived — fits with no special-casing |
| Income Source / Registered Person / Reimbursement Source | **Not accounts.** They are counterparty registries feeding classification — rules for choosing contra accounts. The "starting to resemble a chart of accounts" observation is half-right: the chart of accounts they gesture at is the Expenses/Income subtree they *route money into*, and the journal model gives that chart a real home while the registries stay config |
| Balance Snapshot | A balance assertion verifying the journal |
| The Gap | Sum of Income postings + sum of Expenses postings for the period — same number, now with the guarantee that Gap − Net Worth Movement = suspense balance |

New candidate terms an ADR/design session would need to agree via
`doc/domain/ubiquitous-language.md`: the journal and entry concept
itself, *posting* (or *split* — beancount vs GnuCash naming), *chart
of accounts*, the five account types, and the suspense/unclassified
account. None should be coined unilaterally in code.

## Assessment against the prior ADRs

**ADR 0007** chose balance-snapshot accounts over double-entry, with
an explicit revisit trigger: "if daily use shows a real need for
postings-level detail that the interim model can't give". The trigger
has now fired — via a different route than the ADR anticipated (not
mortgage interest/principal splitting, but the Untracked invariant:
the cash-only Gap arithmetic is provably unable to reach zero, and
each residual costs a manual SQL investigation). ADR 0007's model is
not wasted: accounts, `balance_as_of`, and snapshots all carry
directly into the journal model (snapshots become assertions).
Adopting double-entry **overrides ADR 0007's "not now"** while keeping
its machinery.

**ADR 0005** (independent spend and income ledgers) is **confirmed,
not reversed**: the expense/income account-type split is the same
domain separation expressed structurally, and the ADR's core
principles (raw immutable, derived rebuildable, provenance
first-class) are exactly what makes a *derived* journal viable. What
changes is that the two ledgers stop being the bottom derived layer
and become projections of it.

**Cost.** This is the largest structural change since the spend ledger
itself, in a solo-maintained codebase: schema addition, a reworked
derivation pass, re-derivation/backfill of the real database (the
established backup-first discipline applies), a transition period
where journal and existing ledgers coexist and cross-check, and a
genuine design session on the chart of accounts. Mitigations: the
account/snapshot/classification layers all survive; the model being
copied (beancount/GnuCash's) is thoroughly proven; no new dependency
is taken; and the existing ledgers can remain the TUI's read surface
throughout, so the UI risk is near zero. The realistic failure mode is
under-designing the chart of accounts and equity/opening-balance
handling — both are where first-time double-entry implementations
stall — which is what the ADR-first step is for.

## Recommendation

**Adopt a minimal internal double-entry core — the
beancount/GnuCash-shaped model (typed accounts, balanced postings,
derived journal) in ledgr's own SQLite schema, with no external
dependency or format adoption.** Do not adopt Firefly III's model (its
fixed transaction shapes fail at the already-planned payslip case), do
not embed any beancount toolchain (Python, or file-centric Rust
ports), and do not defer again — the ADR 0007 revisit condition has
been met with algebraic proof, and the household's complexity trend
(named-entity registries, partner accounts, card, pension, mortgage
pending) is in one direction.

Suggested sequencing:

1. **Ship the Net Worth Movement fix first** (fold the card balance
   into the Gap screen's movement figure). Small, correct on its own
   terms, repairs the visible number now, and becomes the acceptance
   test for the journal later (the journal must reproduce it).
2. **Write the ADR** (overriding ADR 0007's deferral, confirming ADR
   0005). It needs to decide: the five account types and how existing
   `AccountType` maps onto them; journal-as-derived-projection vs
   journal-as-source-of-truth (this evaluation argues derived);
   posting vs split naming and the other vocabulary above; the
   suspense-account mechanism and its exact invariant; equity/opening
   balances for account pre-history; how `balance_snapshots` become
   assertions and what a failed assertion surfaces; the
   coexistence/cut-over plan for the three existing derived ledgers;
   and what is explicitly out of scope (multi-currency, accrual,
   plain-text format, beancount export).
3. **Build behind the existing UI**: journal derivation alongside the
   current ledgers, cross-checked (journal-projected spend/income
   totals must equal the existing ledgers' totals on the real
   database) before any screen switches its query source.

## Sources

- [Firefly III — Transactions](https://docs.firefly-iii.org/explanation/financial-concepts/transactions/)
  and [Accounts](https://docs.firefly-iii.org/explanation/financial-concepts/accounts/)
- [Firefly III is not double-entry accounting — Kenneth Ballard](https://www.kennethballard.com/?p=9483)
- [Beancount language syntax](https://beancount.github.io/docs/beancount_language_syntax/)
- [beancount/core/data.py — directive definitions](https://github.com/beancount/beancount/blob/master/beancount/core/data.py)
- [GnuCash SQL schema (piecash object model)](https://piecash.readthedocs.io/en/master/object_model.html)
  and [GnuCash guide — data entry concepts](https://www.gnucash.org/docs/v5/C/gnucash-guide/basics-entry1.html)
- [rustledger](https://github.com/rustledger/rustledger),
  [twilco/beancount crates](https://github.com/twilco/beancount),
  [beancount-parser crate](https://docs.rs/beancount-parser/latest/beancount_parser/)
