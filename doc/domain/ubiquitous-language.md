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
`BANKACCTFROM`) or masked card number. Also called a **tracked
account** (below) when the distinction from a **reference household
account** needs to be explicit. *Origin: banking domain.*

### Tracked Account — established

An **account** (above), named specifically to contrast with a
**reference household account**: a tracked account has its own
`accounts` table row, real imported transactions, and a balance —
`ledgr` actively tracks its activity, not just its identity. Listed
under "Tracked Accounts" in `ledgr status`. *Origin: the user,
2026-07-12, naming the `ledgr status` accounts table.*

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
outflow to a merchant or person, linked to the transaction(s) it derives
from. The only place classification, categorisation, and budget work
happens. See ADR 0005 and
`doc/implementation-notes/spend-ledger-design.md`.
*Origin: the user, design session 2026-07-11.*

### Spend Entry — established

One row in the **spend ledger**: `ledgr`'s judgement that a
**transaction** represents real spend, plus its classification
provenance (rule/confidence/note). Distinct from the transaction it
derives from — the transaction is the raw imported fact; the spend
entry is the derived, categorisable, human-facing record built from it.
*Origin: the user, design session 2026-07-11 (named as part of Spend
Ledger); given its own entry 2026-07-13 for clarity against
**transaction**.*

### Transfer Ledger — candidate

The derived ledger of internal transfers between household accounts:
one **transfer entry** per real-world transfer between two accounts,
built during `ledgr import`'s derivation pass and never re-derived live
by the UI (ADR 0009 — the same principle the spend ledger already
followed). See `doc/implementation-notes/transfer-ledger-design.md`.
Not yet agreed as a formal term — the user asked for the transaction/
transfer-entry terminology to be made precise (2026-07-13), which this
records, but hasn't explicitly signed off on "Transfer Ledger" itself as
the name.
*Origin: the assistant, proposed 2026-07-13 (Delta: Transfer Ledger),
naming mirrors the already-established **Spend Ledger**; pending the
user's confirmation.*

### Transfer Entry — candidate

One row in the **transfer ledger**: `ledgr`'s judgement that two
**transactions** (or one transaction and a household counterpart that
will never itself be imported) are the two legs of the same transfer,
plus how that pairing was made. Distinct from the transactions it
derives from, exactly as a **spend entry** is distinct from the
transaction(s) it derives from — a transaction is the raw imported fact;
a transfer entry is the derived record linking two of them. **Not** one
row per leg (an earlier version of this design was corrected by the user
mid-session, 2026-07-13, precisely because "transfer entry" had been
used to mean a single transaction's own row — see the design doc's
history companion for the full correction).
*Origin: the assistant, proposed 2026-07-13 (Delta: Transfer Ledger);
pending the user's confirmation.*

### Income Ledger — established (deferred)

The independent derived ledger of real inflows (salary, interest,
cashback). A separate domain from spend — it will eventually involve
tax and pensions. Deliberately not being built yet.
*Origin: the user, design session 2026-07-11 (ADR 0005).*

### Internal Transfer — established

Money moving between the user's own accounts (including credit-card
repayments). Produces **no** spend/income entry in the **spend ledger**
(excluded from spending per the Rebel Finance method), but does produce
a **transfer entry** in the **transfer ledger** — a different ledger
from spend/income, tracking money movement *within* the household rather
than crossing its boundary. (Until 2026-07-13, internal transfers were
recorded only as a `transaction_links` pairing with no dedicated ledger;
see Delta: Transfer Ledger.) *Origin: Rebel Finance ("transfers between
your own accounts"), confirmed by the user.*

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

The set of accounts belonging to the household: **tracked accounts**
plus **reference household accounts** (below). Transfers between
household accounts are internal. If double-entry accounting lands
later, the membership test is expected to become structural
(asset/liability account typing) rather than a list. *Origin: design
session 2026-07-11.*

### Reference Household Account — established

A household account `ledgr` knows about *by reference only* — sort
code + account number, hand-maintained in
`Config.household_accounts` — with no `accounts` table row, no
balance, no transaction history, and no import, ever. Reserved
specifically for accounts that will **never** be imported — the
concrete case is a partner's own bank account that only she can
download statements for (only her credit card becomes an ordinary
**tracked account**): registering her account as a reference is what
lets transfers to/from it be recognised as internal rather than
external spend/income. **Not** the right home for one of the user's
own accounts that simply hasn't had a statement imported yet — that's
still a tracked account, just with no transaction history until an
import happens (correction 2026-07-12: Shared Shopping Account, sort
`208794`/account `...3394`, was briefly registered here before an OFX
export for it was actually imported — it was always importable, just
hadn't been yet). Listed separately from real accounts in `ledgr
status`. Distinct from a **proxy account**, which exists to hold
*manual spend entries*, not just to mark household membership.
Optionally carries a `name` (the household member's full name, e.g.
`"ROMINA SCARAMAGLI"`) used to recognise a person-to-person `NAME`
field that carries no sort code/account number at all — Barclays shows
these as either the full name (paying them, your saved payee nickname)
or `"<Surname> <first initial>"` (them paying you, the sender name
Faster Payments echoes back); see `derive::matches_household_member_name`.
Discovered 2026-07-12: a `Manual Funds Transfer` from/to a household
member sometimes carries no account digits to match on at all, so the
sort code/account number check alone missed it, misclassifying it as
`reimbursement`/`person_payment` (a rule intended for genuine external
people) instead of an internal transfer.
*Origin: the user, 2026-07-12; decision trail: ADR 0008.*

### Manual Funds Transfer — candidate

A one-off transfer the user sends themselves (a Faster Payment), as
opposed to an **automated transfer**. Recognised in Barclays OFX data by
the destination's sort code/account number appearing at the *start* of
`NAME` (`parse_account_prefix` in `src/derive.rs`), usually followed by
a Barclays `FT` marker. Can be an **internal transfer** (household
account) or **spend** (external account) — the transfer/payment
distinction is decided by matching the sort code/account number (or,
when the `NAME` carries no digits at all, the household member's
registered `name` — see **Reference Household Account** above) against
known household accounts, never by whether the transfer was manual or
automated. *Origin: the user, 2026-07-12, reviewing
`doc/developer-docs/transfer-detection.md`.*

### Automated Transfer — candidate

A transfer initiated by the bank on a recurring basis rather than by
the user each time: a **standing order** or **direct debit**, as
opposed to a **manual funds transfer**. Recognised in Barclays OFX data
by the destination's sort code/account number appearing at the *end* of
`NAME`, after a human label (`parse_trailing_account_suffix` in
`src/derive.rs`); Barclays' `STO`/`DIRECTDEBIT` markers are often
truncated away by `NAME`'s 32-character cap. Like a manual funds
transfer, can be internal or external — same matching rule, not
decided by payment type. *Origin: the user, 2026-07-12, same session as
Manual Funds Transfer above.*

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

### Proxy Account — candidate

An `Account` row that stands in for spend which happens on accounts
`ledgr` will never see the real data for (e.g. a partner's own bank
accounts, when only her credit card is imported) — it carries no real
sort code/account number, so it can never be matched as an internal
transfer or collide with a real account. Exists purely so a **manual
spend entry** has something to attach a raw `Transaction` row to,
keeping the existing spend-entry provenance model (`spend_entry_sources`
always references a real `transactions.id`) intact rather than
special-casing manual entries with no source row. *Origin: the user,
2026-07-12, discussing how to record a partner's un-imported spend.*

### Manual Spend Entry — candidate

A spend entry the user types in directly (e.g. "spent £200 this month
on food") rather than one derived from an imported transaction —
`classified_by = 'manual'` (already in the schema's `CHECK` constraint,
just not yet reachable via any UI/CLI path). Backed by a `Transaction`
row on a **proxy account** so it fits the existing derivation/provenance
model rather than requiring schema changes. Not yet designed how the
manual-entry flow itself works (CLI command vs TUI form). *Origin: the
user, 2026-07-12.*

