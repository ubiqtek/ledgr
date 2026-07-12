# Ubiquitous language

The domain vocabulary of `ledgr`. Code, schema, docs, and conversation
should all use these words with these meanings — one concept, one name.

Conventions:

- Every term records where it came from, because words introduced
  without agreement ("household" at first, "statement") have had to be
  unpicked or re-litigated. Provenance keeps us honest.
- Status: **established** (in use, agreed) · **candidate** (proposed,
  not yet agreed) · **rejected** (do not use; kept here so it isn't
  reinvented).

## Terms

### Account — established

One of the user's real bank/pension/investment accounts (current
accounts, savings, a credit card, pensions).
Identified at import time by sort code + account number (OFX
`BANKACCTFROM`) or masked card number. *Origin: banking domain.*

### Transaction — established

One raw line as imported from the bank: immutable evidence, never
edited or categorised directly. De-duplicated per account by
`external_id` (OFX `FITID`). *Origin: banking domain.*

### Import — established (renamed from "Statement")

One imported file, recorded in the `imports` table (was `statements`)
so re-importing the same file is a no-op. Previously called
"Statement" — introduced by the assistant in the original schema
without discussion, and not quite honest: what lands in the inbox are
**exports/downloads** with arbitrary, user-chosen date ranges (OFX
downloads, the Barclaycard CSV), not statements in the banking sense
(a periodic document the bank issues). "Import" was chosen over
*export*/*download*/*import file* despite already naming the
`ledgr import` command/run: one **import** is one file; running
`ledgr import` processes a batch of zero or more imports in one run
(`ImportSummary`) — coherent, not a collision. *Origin:
assistant-invented, 2026-07 schema; renamed by the user, Delta:
Statement/Import Naming Cleanup, 2026-07-12.*

### Inbox — established

The synced folder the user drops downloaded exports into
(`inbox_dir`); `ledgr import` processes pending files and moves them
to `processed/`. *Origin: agreed in Bank Statement Import delta.*

### Spend — established

Money leaving the household to a merchant or person. Spend happens
**only when money actually leaves the household** — movement between
household accounts is never spend, so saving monthly into a pot for a
future purchase is not spend; the eventual purchase is (this settles
the Rebel Finance "sinking fund" convention). *Origin: the user,
design session 2026-07-11.*

### Reimbursements and Refunds — established

Inbound money that pays back earlier spend: a merchant refund (e.g. a
card refund for a returned item) or a reimbursement from a person
outside the household (e.g. a friend paying you back for their
concert ticket). Recorded in the **spend ledger** as a sign-reversed
entry linked to the original spend
(`transaction_links, relation = 'refund'`) so spending nets down to
what the household truly paid — never treated as income. (Money back
from a household member is simply an internal transfer.)
*Origin: design session 2026-07-11.*

### Spend Ledger — established

The derived ledger of real-world spending: one **spend entry** per
outflow to a merchant or person, linked to the raw transaction(s) it
derives from. The only place classification, categorisation, and
budget work happens. See ADR 0005 and
`doc/implementation-notes/spend-ledger-design.md`.
*Origin: the user, design session 2026-07-11.*

### Income Ledger — established (deferred)

The independent derived ledger of real inflows (salary, interest,
cashback). A separate domain from spend — it will eventually involve
tax and pensions. Deliberately not being built yet.
*Origin: the user, design session 2026-07-11 (ADR 0005).*

### Internal Transfer — established

Money moving between the user's own accounts (including credit-card
repayments). Produces an entry in **neither** ledger; recorded only as
a pairing between the two raw transactions (`transaction_links`,
`relation = 'transfer'`). Excluded from spending per the Rebel Finance
method. *Origin: Rebel Finance ("transfers between your own
accounts"), confirmed by the user.*

### Household — established

The accounting entity whose money ledgr tracks — the personal-finance
equivalent of "Company" / "Legal Entity" in the corporate world. Money
moving *within* the household is an internal transfer; money crossing
its boundary is spend or income. A household may be one person or
several. Never a spending *category* (use "Home" if a home-supplies
category is ever wanted). Full discussion, alternatives considered,
and evidence: [household.md](household.md). *Origin: Rebel Finance /
economics; adopted 2026-07-11.*

### Household Accounts — established

The set of accounts belonging to the household: the imported accounts
plus a config-maintained list of known-but-not-imported ones (e.g. a
partner's account). Transfers between household accounts are internal.
Exact membership (which partner/family accounts are in) is still to be
decided. If double-entry accounting lands later, the membership test
is expected to become structural (asset/liability account typing)
rather than a list. *Origin: design session 2026-07-11.*

### Reference / Note — candidate

The user's own free-text label on a transfer (e.g. "SOY SAUCE"),
carried in the OFX `NAME` field and propagated onto the
derived spend entry as its human description. "Reference" = the raw
field; "note" = the spend-entry column. *Origin: banking domain
("payment reference") + design session 2026-07-11.*

### Spend Enrichment — established

Copying the reference from a matching internal transfer onto the spend
entry it funded, when the transfer's reference is more informative
than the merchant/card line alone (e.g. a "PIZZA OVEN" transfer
reference attached as the **note** on the later "ARGOS" card spend).
Amount/date fuzzy match, confidence-scored, correctable — enrichment
of description only, never changes what counts as spend or its
amount. Previously called "note propagation" in the spend ledger
design doc; renamed for clarity. *Origin: design session 2026-07-11;
named 2026-07-11 (this session).*

### Balance Snapshot — established

A bank-reported balance anchor (OFX `LEDGERBAL`) stored per import,
from which the balance at any date is reconstructed (nearest anchor ±
transactions). *Origin: agreed during Bank Statement Import delta.*

### Category — established (taxonomy TBD)

A classification applied to spend entries only (never to raw
transactions), following the Rebel Finance method (~10 top-level
categories, optional nesting). Exact taxonomy still to be confirmed.
*Origin: Rebel Finance.*

### The Gap — established

Income minus spending for a period; the central Rebel Finance metric.
Computed across the two ledgers once both exist.
*Origin: Rebel Finance.*

