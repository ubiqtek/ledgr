# Barclaycard PDF transaction export: structure and identity problem

Knowledge-base article compiled 2026-07-12 from a real Barclaycard
Rewards Visa "Transactions" PDF export (205 transactions, 01/01/2026 to
12/07/2026) and general card-number research (ISO/IEC 7812). Written to
support **Delta: Credit Card Transaction Import** (Task 2: evaluate the
PDF export) and the account-identity design for a future `CreditCard`
`ImportFileParser`.

All card numbers below are placeholders/fictional — no real card number
is recorded in this document or anywhere else in the repo.

## What the PDF export contains

Header block (once per file):

| Field | Example | Notes |
|---|---|---|
| Card product name | `Barclaycard Rewards` | Not unique — shared by every customer on this product |
| Masked card number | `VISA 0002` | **Last 4 digits only.** No sort code, no full account number, no customer/account reference anywhere in the export |
| Current balance | `£613.73` | As of export date |
| Available credit / credit limit | `£10,086.27` / `£10,700.00` | |
| Statement window | `Showing 205 transactions from 01/01/2026 to 12/07/2026` | User-selected date range, not a billing-cycle statement |

Per-transaction rows:

| Field | Notes |
|---|---|
| `Date` | `dd MMM yyyy`, e.g. `10 Jul 2026` |
| Type tag | `Purchase` / `Payment received` / `Other` (the last covers Barclaycard Cashback rebates) |
| Description | Merchant descriptor + location, sometimes with the original foreign-currency amount embedded (e.g. `12.71 POUND STERLING USA`, `129.99 EURO CYPRUS`) — same "packed free text" idea as OFX `NAME` (see `doc/kb/ofx/structure.md`), but no fixed-width truncation observed here |
| Amount | Split across two columns, **Money in** / **Money out** — sign is positional (which column it's in), not a signed value like OFX `TRNAMT` |

**No stable per-transaction ID** (no `FITID` equivalent) — a re-exported
overlapping date range would need date+amount+description matching for
de-duplication, the same open problem already flagged for
`GenericCsvParser` (Bank Transaction Import, Task 2).

**Amounts are penny-precise**, unlike the Barclaycard CSV export (which
rounds to whole pounds — see Task 1's finding). This resolves Task 2's
open question: **the PDF is the better source for the spend ledger**,
CSV should be deprioritised or dropped in favour of it, subject to
confirming the PDF is actually parseable (text-extractable, not a
scanned image — it was, in this sample).

## Card number structure (ISO/IEC 7812)

A card PAN (Primary Account Number) is not a single opaque value — it
has a deterministic internal structure:

| Position | Name | Meaning |
|---|---|---|
| Digit 1 | Major Industry Identifier (MII) | `4` = Visa |
| Digits 1–6 (up to 8 on newer schemes) | **IIN / BIN** (Issuer Identification Number) | Identifies the issuing bank *and specific card product* — shared by every card issued under that product, not unique to one customer |
| Digits 7…N-1 | Individual account identifier | The part that's actually specific to the cardholder's account |
| Digit N (last) | Luhn check digit | Computed checksum, not account data |

The real card's BIN (`492913…`, first 6 digits) identifies it as a
Barclaycard Rewards Visa specifically — useful for recognising *what
kind* of card it is, but useless as an account identity key since it's
identical across every customer on that product.

**Reissue behaviour:** when a card is reissued (lost/stolen, expiry,
fraud block), issuers commonly generate a **new PAN**, not just a new
expiry date — the individual-account-identifier portion is not
guaranteed to survive a reissue. This means **nothing in the card number
itself is a safe long-term identity key** for `account_identity()`.

## The truncated card number inside bank transaction descriptions

Payments *to* the card, seen from the bank side (e.g. a Barclays current
account `NAME` field), read like:

```
MR JAMES BARRITT 49291328548900
```

This is the cardholder's name followed by **the first 14 digits of the
16-digit card PAN** — everything except the trailing 2 digits, which
includes the Luhn check digit. This is the same kind of fixed-width-field
truncation already documented for sort-code/account-number transfers in
`doc/developer-docs/transfer-detection.md` (Barclays' `NAME` field caps
at 32 characters — see `doc/kb/ofx/structure.md`).

This partial PAN is useful **corroborating evidence** that a payment is
going to a specific credit card, but is not a complete or reliable
matching key on its own:
- it's truncated (missing digits, including the checksum digit)
- even the untruncated PAN isn't stable across a reissue (see above)

## Recommended identity/matching strategy

Given neither the PDF export nor the bank-side description offers a
stable, reissue-proof account identifier:

1. **Don't key `CreditCard` account identity off the PAN or its
   truncated form.** Use a config-registered label instead (same pattern
   as `account_names` in `src/config.rs`), since the export gives
   nothing durable to auto-derive an identity from.
2. **Match card payments to bank-side transfers by date + exact amount**
   between the two accounts, not by account number. Verified against
   real data: two June 2026 payments from a Barclays current account
   (£295.81 on 1 Jun, £453.77 on 15 Jun) match — to the day and the
   penny — two `"PAYMENT, THANK YOU"` rows on the card statement. This
   sidesteps the reissue problem entirely, since it never depends on the
   card number.
3. This is a different transfer-detection mechanism to the existing
   sort-code/account-number matching used for inter-Barclays-current-
   account transfers (`derive.rs`) — the two will need to coexist once
   `CreditCard` accounts are real, not replace one another.

`AccountType::CreditCard` already exists in `src/model.rs` and the
`accounts.account_type` schema `CHECK` constraint (unused so far) — no
new account type needs adding, just a parser and the matching strategy
above.

## Sources

- Real Barclaycard Rewards Visa "Transactions" PDF export, examined
  2026-07-12 (205 transactions, penny-precise, no stable transaction ID).
- ISO/IEC 7812 (card numbering and registration procedures) — general
  IIN/BIN and Luhn-check-digit structure.
- Real Barclays current-account transaction descriptions showing the
  truncated card-payment reference, cross-checked 2026-07-12.
