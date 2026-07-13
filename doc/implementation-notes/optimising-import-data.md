# Optimising import data — bank statement export formats

Comparing Barclays export formats for the **current account** (Jims
Premier Account) against OFX's `NAME`-field truncation problem (root
cause of several transfer-pairing workarounds in Delta: Transfer Ledger,
e.g. `SHARED BILLS ACCO` — see `doc/kb/ofx/structure.md`). Three exports
examined, same account/date range (573 transactions,
01/01/2026–13/07/2026): `data.csv`, `Transaction.pdf`, and `data.qbo`
(scratch-only, `~/Downloads`, not imported or committed).
`Transaction.pdf` is a current-account statement PDF, distinct from the
already-solved Barclaycard credit-card PDF (`BarclaycardPdfParser`).

## `data.qbo` is OFX, not a distinct format

QuickBooks' `.qbo` export is the identical OFX 1.02 SGML payload
`BarclaysOfxParser` already parses — same header (`OFXHEADER:100`,
`VERSION:102`), same tags (`FITID`, `TRNTYPE`, `LEDGERBAL`/`BALAMT`), same
`NAME` field, same truncation in the same place (`SHARED BILLS ACCO
	208794 231650`, missing the final two digits, identical to the real
`.ofx` files). No new data, no improvement, and no separate parser
needed — `.qbo` is just a different file extension some export menus use
for the same OFX payload `.ofx`/`.qfx` already produce. `src/import/pipeline.rs`'s
extension map (`parser_for`) currently only recognises `.ofx`/`.qfx` —
adding `.qbo` alongside them (same `BarclaysOfxParser`) would be a
one-line change, only relevant if a QuickBooks-labelled download option
ever gets used instead of Quicken/Money.

## Truncation

Every format truncates the free-text label — OFX's `NAME` field, CSV's
22-character `Memo` field, this PDF's ~18-character label line (e.g.
`"SHARED BILLS ACCO"`, cut from "SHARED BILLS ACCOUNT").

In `Transaction.pdf`, the sort code + account number for
`Funds Transfer`/`Standing Order`/`Direct Debit`/`Bill Payment` rows sits
on its own line, separate from the truncated label:

```
Standing Order
SHARED BILLS ACCO
208794 23165086 STO        -£3,415.00   £2,086.61
```

`208794 23165086` is full and untruncated. OFX has no equivalent
separation — label and account number share one 32-char `NAME` field, so
a long label truncates the account digits along with it, which is the
root cause of the `SHARED BILLS ACCO` transfer-pairing gap that Delta:
Transfer Ledger's self-reference-match tier exists to work around. This
PDF's layout can't produce that failure mode, regardless of label
length.

Merchant names on card `Debit` transactions are still truncated (~18
chars, e.g. `'CUBERT P.O. &LONDI'`, `'Trevilley Farm Sho'`) — cosmetic
spend-description quality, not something that breaks derivation logic
the way a truncated account number does.

`data.csv` has no equivalent line separation — label and account number
share one 22-char field — so it does not share the PDF's advantage.

## `Transaction.pdf` — structure

- Header: account name, sort code + account number, available/last-night
  balance, overdraft limit, transaction count and date range.
- **"Pending debit card transactions" section**: date, full untruncated
  merchant description including country code (e.g. `"ALLIANCE PARKING
  -IPS Doncaster GB"`), amount, full masked card number
  (`Card Number **** **** **** 4016`). Not currently imported by
  anything (only posted transactions would be).
- **Posted transactions table**: Date / Description / Money in / Money
  out / Balance. Each row: date, type (`Debit`/`Counter Credit`/
  `Funds Transfer`/`Standing Order`/`Bill Payment`), a label line
  (truncated per above), and — for transfer-type rows only — a separate
  `<sort code> <account number> <suffix>` line (card `Debit` rows get
  `ON <date> CPM` instead), then amount + running balance.
- Running balance is per-transaction; OFX only gives one `LEDGERBAL`
  anchor, CSV has no balance column at all.
- No stable per-transaction ID — no `FITID` equivalent.
- 43 pages; `"Page N of 43"` footers land mid-transaction (same failure
  mode `BarclaycardPdfParser` already handles — order-tolerant,
  footer-stripping parsing, not a new problem class).

## De-duplication without `FITID`

A hash of `(date, amount, description)` is not safe as a de-dup key:
`data.csv` contains two genuine same-day, same-amount, same-truncated-
description pairs (`AMATA CAFE` ×2 on 06/07/2026, `STAGECOACH SERVICE`
×2 on 15/05/2026) that would collide.

The PDF's running balance disambiguates them — same date/amount/
description, but balances differ (`£1,193.28` vs `£1,196.28` for the
`AMATA CAFE` pair), since balance is monotonic and encodes each
transaction's position in the account's history. `hash(date, amount,
description, running_balance)` is collision-free where the three-field
hash isn't. This makes running balance a required part of any de-dup key
for this format — and `data.csv`, having no balance column, has no safe
de-dup key available at all.

Untested: whether running balance is stable across two exports of
overlapping date ranges (re-downloading the same window should reproduce
the same balance per row, but not verified against a real overlapping
re-download the way OFX's `FITID` de-dup was).

## Other options (external, unverified)

Three further options, not yet independently verified to exist or be
trustworthy — none investigated hands-on:

1. **Third-party PDF-to-CSV converters** (e.g. "BankConverter.co.uk",
   "BankConv", "DigiParser Barclays Solution") — upload a Barclays PDF
   statement, get back a structured CSV with the full untruncated text.
   Would need scrutiny before use: this means sending real bank
   statements (account numbers, balances, full transaction history) to
   a third-party service, which `ledgr`'s entire design (`CLAUDE.md`:
   "no data leaving the machine") exists to avoid. Not aligned with the
   project's privacy model regardless of parsing quality — would only be
   worth reconsidering if `ledgr` ever relaxed that constraint.
2. **Browser extension hooking into a live Barclays Online Banking
   session** (e.g. "Barclays Export Transactions") to pull a richer
   payload than the site's own "Export All" button exposes. Same
   objection as above, more acute: granting a third-party extension
   access to an authenticated banking session is a materially larger
   trust/security surface than a one-off file upload, and likely against
   Barclays' terms of service. Not recommended without much closer
   scrutiny of the specific extension's provenance and permissions.
3. **Open Banking** (e.g. via Xero/QuickBooks-style direct feeds) —
   bypasses file-based import entirely. This is already a tracked,
   in-progress piece of work: see `doc/planning/plan.md`, "Delta: Live
   Open Banking (Enable Banking)", Task 1 (evaluating feasibility and
   security model against Enable Banking specifically, not a generic
   Open Banking aggregator). Any Open Banking discussion should continue
   there rather than fork into a second thread here.

## Assessment

`Transaction.pdf` is worth pursuing: it structurally eliminates
account-number truncation, the actual source of the `SHARED BILLS ACCO`
transfer-pairing gap. A future `BarclaysStatementPdfParser` (current
account, distinct from `BarclaycardPdfParser`) could let tier-1
description-cross-reference matching succeed for transfers that
currently only pair via the weaker tier-3 self-reference route.

`data.csv` offers nothing OFX doesn't already have, minus `FITID`, and
lacks the PDF's account-number and balance advantages — not a promising
alternative.

## Open questions

1. Does the "Pending debit card transactions" section's untruncated data
   have any use once those transactions post and truncate?
2. Switch the primary current-account import format from OFX to this
   PDF outright, or use the PDF only to backfill/correct account numbers
   on transfers OFX already truncated, keeping OFX primary? Not
   evaluated.
3. Confirm the ~18-char label truncation width and the label/account-
   number separation hold across a wider real-data sample (so far only
   6 household counterpart accounts in one file) before committing to a
   parser design.

Neither file has been imported into the real database or committed;
`~/Downloads/data.csv` and `~/Downloads/Transaction.pdf` remain
scratch-only pending a decision on next steps.
