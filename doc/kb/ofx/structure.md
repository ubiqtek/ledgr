# OFX statement structure

Knowledge-base article on the OFX (Open Financial Exchange) statement
format: what the specification defines, and what Barclays actually emits
in its exports. Compiled 2026-07-11 from the OFX specification (via the
[OFX 2.1.1 schema documentation](https://schemas.liquid-technologies.com/OFX/2.1.1/)
and the [OpenExchange message set specification](https://www.xml.coverpages.org/OFEXFIN2.html))
cross-checked against ledgr's three real Barclays current-account
exports (939 transactions).

## Spec versions and encodings

- **OFX 1.x** — SGML-style, colon-separated header block, tags often
  unclosed. This is what Barclays emits (`OFXHEADER:100`,
  `VERSION:102`, `ENCODING:USASCII`, `CHARSET:1252`).
- **OFX 2.x** — well-formed XML with an XML declaration. Same
  aggregates and element names; the structural content below applies to
  both.

A bank statement response lives at
`OFX → BANKMSGSRSV1 → STMTTRNRS → STMTRS`, which contains:

| Element | Meaning |
|---|---|
| `CURDEF` | Default currency for the statement (Barclays: `GBP`) |
| `BANKACCTFROM` | Which account this statement is for: `BANKID` (sort code), `ACCTID` (account number), `ACCTTYPE` |
| `BANKTRANLIST` | `DTSTART`/`DTEND` plus the list of `STMTTRN` transactions |
| `LEDGERBAL` | Bank-reported balance (`BALAMT` + `DTASOF`) — the "official" balance anchor ledgr stores as a balance snapshot |
| `AVAILBAL` | Available balance (funds accessible now, net of holds). Barclays: not emitted |

## The `STMTTRN` aggregate (one transaction)

Elements in schema order. "Barclays" column = observed in the real
exports.

| Element | Req | Spec meaning | Barclays |
|---|---|---|---|
| `TRNTYPE` | ✓ | Transaction classification (enum below) | ✓ (coarse — see below) |
| `DTPOSTED` | ✓ | Date posted | ✓ |
| `DTUSER` | | Date user initiated the transaction | — |
| `DTAVAIL` | | Date funds become available | — |
| `TRNAMT` | ✓ | Signed amount, decimal string; negative = money out | ✓ |
| `FITID` | ✓ | Transaction ID issued by the FI, *A-10*; "used to detect duplicate downloads". Unique per account | ✓ (ledgr's `external_id`) |
| `CORRECTFITID` / `CORRECTACTION` | | FI-issued correction of an earlier transaction | — |
| `SRVRTID` | | Server transaction ID | — |
| `CHECKNUM` | | Cheque number | — |
| `REFNUM` | | Reference number | — |
| `SIC` | | Standard Industry Code of the payee | — |
| `PAYEEID` / `PAYEE` | | Structured payee data (choice with `NAME`) | — |
| `NAME` | choice | "Name of payee or description of transaction", **max 32 chars (A-32)** | ✓ (the only descriptive field) |
| `EXTDNAME` | | Extended name, when 32 chars isn't enough | — |
| `BANKACCTTO` / `CCACCTTO` | | Destination account, "if this was a transfer to an account and the account information is available" | — (see below) |
| `MEMO` | | "Extra information (not in NAME)", up to 255 chars | — |
| `CURRENCY` / `ORIGCURRENCY` | | Per-transaction currency override | — |

Key spec semantics:

- **`FITID` is the de-dup key.** Scoped to the account; a re-downloaded
  statement carries the same FITIDs, so `(account, FITID)` uniquely
  identifies a transaction. ledgr already relies on this
  (`idx_transactions_account_external_id`).
- **`NAME` is hard-capped at 32 characters** — this, not any Barclays
  quirk, is why long user references get truncated (a reference like
  "LAWNMOWER REPAIRS" arrives as `LAWNMOWER REPAI` once the
  counterparty account prefix has used up part of the budget).
- **`BANKACCTTO` is the spec's proper way to mark a transfer's
  destination.** Barclays does not emit it, so transfer detection must
  fall back to parsing `NAME` (below).

### `TRNTYPE` enumeration (all 17 values)

| Value | Meaning |
|---|---|
| `CREDIT` | Generic credit |
| `DEBIT` | Generic debit |
| `INT` | Interest earned or paid |
| `DIV` | Dividend |
| `FEE` | FI fee |
| `SRVCHG` | Service charge |
| `DEP` | Deposit |
| `ATM` | ATM debit or credit |
| `POS` | Point of sale debit or credit |
| `XFER` | **Transfer** |
| `CHECK` | Cheque |
| `PAYMENT` | Electronic payment |
| `CASH` | Cash withdrawal |
| `DIRECTDEP` | Direct deposit |
| `DIRECTDEBIT` | Merchant-initiated debit (direct debit) |
| `REPEATPMT` | Repeating payment / standing order |
| `OTHER` | Other |

## What Barclays actually emits

Observed distribution across the three real current-account exports
(939 transactions):

| TRNTYPE | Count | What it turns out to mean in practice |
|---|---|---|
| `OTHER` | 624 | Card payments, faster-payment transfers, person-to-person payments — i.e. almost everything interesting |
| `DIRECTDEBIT` | 148 | Direct debits (reliable) |
| `PAYMENT` | 117 | Bill payments |
| `DIRECTDEP` | 27 | Incoming credits (salary etc.) |
| `REPEATPMT` | 22 | Standing orders |
| `CASH` | 1 | Cash withdrawal |
| `XFER` | **0** | **Barclays never uses it** — own-account transfers arrive as `OTHER` |

So **`TRNTYPE` alone cannot identify transfers**. The information is in
`NAME`, which Barclays packs with recognisable sub-formats (tab- and
space-delimited within the 32-char budget). All examples below are
fictional but format-faithful; the patterns themselves were verified
against the real exports:

| Pattern in `NAME` | Meaning | Example |
|---|---|---|
| `<sort code> <account no> \t<reference> FT` | Faster-payment transfer; counterparty account identified by sort code + account number; `<reference>` is the sender's free-text note | `209912 12345678 \tPIZZA OVEN FT` (−89.00) |
| `<sort code> <account no> STO \t…` | Standing order to that account | `209912 12345678 STO \t209912 1234` |
| `<PERSON NAME> \t<reference> FT` | Faster payment to/from a person (no account visible) | `J SMITH \tWINDOW CLEAN FT` |
| `<MERCHANT> \tON <dd MMM> CPM` | Card payment (contactless/chip) to merchant | `PETROL STATION 12 \tON 09 JUL CPM` |
| `<MERCHANT> \tON <dd MMM> CRM` / `CRE` | Card refund / credit | `GARAGE SERVICES \tON 26 FEB CRM` (+40.00) |
| `<MERCHANT> \tON <dd MMM> BCC` | Card credit (e.g. online refund) | `AMZNMktplace \tON 16 MAR BCC` |
| `<MERCHANT> <COUNTRY>\tAMOUNT IN <CCY>\t<amount>` | Foreign-currency card payment, original amount embedded | `PIZZA RESTAURANT NORWAY\tAMOUNT IN NOK` |

Critical observation for transfer pairing: **both sides of an
own-account transfer carry the same reference**, each naming the *other*
account. E.g. (fictional accounts) account A shows
`209912 12345678 \tPIZZA OVEN FT` −89.00 while account B shows
`209934 87654321 \tPIZZA OVEN FT` +89.00. Given the set of the user's
own sort-code/account-number pairs (known from each file's
`BANKACCTFROM`), internal transfers are deterministically identifiable
from a single side, and pairable when both sides are imported
(equal-and-opposite amount + matching reference + counterparty account
pointing back).

Other observations:

- Descriptions can embed literal tab characters (already handled by
  `clean_description()` in `barclays_ofx.rs`).
- `LEDGERBAL` is present once per file and matches the account's real
  balance (ledgr stores it as a balance snapshot); `AVAILBAL` is absent.
- Every transaction carries a `FITID`; 0 of 939 were missing.
- Sign convention is as per spec: negative `TRNAMT` = money out.

## Sources

- [OFX 2.1.1 schema — StatementTransaction](https://schemas.liquid-technologies.com/OFX/2.1.1/statementtransaction.html)
- [OFX 2.1.1 schema — TransactionEnum](https://schemas.liquid-technologies.com/OFX/2.1.1/transactionenum.html)
- [OpenExchange message set specification (OFX financial messages)](https://www.xml.coverpages.org/OFEXFIN2.html)
- Real Barclays OFX exports in the ledgr inbox (`processed/data*.ofx`), examined 2026-07-11
