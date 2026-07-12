# Transfer Detection — Technical Notes

Companion to the user-facing
[Transfer Detection](../user-guide/transfer-detection.md) guide, and to
[OFX Format — Rough Shape](ofx-format.md) (general file structure). This
document is specifically about how transfer data is encoded within that
structure, and why transfer detection has to work the way it does as a
consequence.

All account numbers, sort codes, and transaction descriptions below are
**made up** — not the user's real data — but kept in the same shape and
format as what Barclays actually sends (6-digit sort codes, 8-digit
account numbers, the same `NAME` conventions), so the structure is
faithful even though the specific digits/labels aren't real.

## Where a transfer's data lives in the file

Every transaction is one small `STMTTRN` block inside a flat, repeated
list (`BANKTRANLIST`) — see [OFX Format — Rough Shape](ofx-format.md) for
the full file layout. Each block has exactly five fields: `TRNTYPE`,
`DTPOSTED`, `TRNAMT`, `FITID`, `NAME`. A transfer-shaped one (fabricated
example, real shape):

```
<STMTTRN>
  <TRNTYPE>OTHER
  <DTPOSTED>20260615000000[-5:EST]
  <TRNAMT>6.17
  <FITID>10000000000000001
  <NAME>111222 99911223 	GAFFA TAPE FT
</STMTTRN>
```

`NAME` — the only free-text field, capped at 32 characters — is where
everything below is decoded from.

## Why isn't the destination account just in the transaction?

The OFX spec defines `BANKACCTTO`/`CCACCTTO` for exactly this: a nested
account aggregate (own `BANKID`/`ACCTID`) that can sit inside a
`STMTTRN`. Barclays never sends it — zero occurrences across every real
export `ledgr` has processed, for any transaction, transfer or
otherwise. Not a parsing gap: the field is defined, Barclays just
doesn't populate it.

That leaves `NAME` — a 32-character free-text field meant for "name of
payee or description of transaction" — as the only place any
account-identifying information appears. Barclays reuses it: instead of
a real payee name, a transfer's `NAME` starts (or, in the second shape,
ends) with the destination's sort code and account number as literal
digits, e.g. `111222 99911223` in the example above. The rest of this
document is about decoding that convention.

## Where the account identity actually comes from

`BANKACCTFROM` appears exactly **once per file, not once per
transaction** (see [OFX Format — Rough Shape](ofx-format.md)) — it sits
above `BANKTRANLIST` and identifies which single account every
transaction in that file belongs to. Individual `STMTTRN` blocks carry
no account identity of their own, which is why a transfer's *other*
side has to be decoded from `NAME` instead (previous section).

That file-level `BANKACCTFROM` block identifies the account via `BANKID`
and `ACCTID` — but not quite the way those names suggest:

- `BANKID` turns out to be a fixed identifier for Barclays' OFX server —
  every account exports the same value, regardless of the account's real
  sort code. It's not usable for matching.
- The real sort code + account number are concatenated inside `ACCTID`
  itself instead: the first 6 digits are the sort code, the remaining 8
  are the account number — confirmed by cross-checking against what shows
  up inside real transactions' `NAME` fields, which independently encode
  the same sort code + account number for transfers. So an `ACCTID` of
  (fabricated) `11122299911223` splits into sort code `111222` + account
  number `99911223`.

`ledgr` treats this split as conditional on the `ACCTID` being exactly 14
digits — every real one observed so far is. If a differently-shaped
`ACCTID` ever turns up (a different bank, a different account type), no
sort code is recorded for that account rather than guessing wrong — which
just means it won't participate in transfer matching until this is
extended, not that it gets silently misclassified.

## How a transaction's `NAME` encodes the other side of a transfer

The sort code and account number of the other side of a transfer show up
as text inside a transaction's `NAME`. Two payment types matter here —
a **manual funds transfer** (a one-off Faster Payment the user sends
themselves) and an **automated transfer** (a direct debit or standing
order that recurs on its own) — and each tends to encode that
information in a different position within `NAME`, described below.
Either type can be internal (to a household account) or external
(spend); which one it is is decided purely by whether the sort
code/account number matches a known household account (see
[Rule ordering](#rule-ordering)), never by the payment type itself.

### Manual funds transfers — sort code and account number come first

```
<STMTTRN>
  <TRNTYPE>OTHER
  <DTPOSTED>20260615000000[-5:EST]
  <TRNAMT>-89.00
  <FITID>10000000000000004
  <NAME>111222 99911223 	GAFFA TAPE FT
</STMTTRN>
```

Here `111222` and `99911223` are the household account's sort code and
account number, followed by a free-text note (the sender's own
reference, here "GAFFA TAPE"), then a trailing `FT` — Barclays' own
abbreviation for "Funds Transfer", confirmed against
[Barclays' published statement abbreviations reference](https://www.barclays.co.uk/help/accounts/statements-balances/abbreviations/).
Barclays puts `FT` on both own-account transfers and person-to-person
Faster Payments alike, which is why `TRNTYPE` alone (`OTHER` for both)
can't tell them apart and the `NAME` content has to do the work
instead. Recognised whenever a `NAME` starts with exactly six digits,
then a space, then exactly eight digits — strict enough not to match an
ordinary merchant description that happens to start with a number.

### Automated transfers (direct debit or standing order) — a label comes first, sort code/account number last

```
<STMTTRN>
  <TRNTYPE>OTHER
  <DTPOSTED>20260615000000[-5:EST]
  <TRNAMT>-300.00
  <FITID>10000000000000005
  <NAME>HOLIDAY POT 111222 99944556
</STMTTRN>
```

Same kind of account reference, but a human label (the account's own
nickname, e.g. "HOLIDAY POT") comes first and the sort code/account
number are the *last two* whitespace-separated tokens instead of the
first two. The manual-transfer check above doesn't catch this — it only
looks at the start of the string — so a second check looks at the last
two tokens instead, accepting them as a sort code/account number pair
under the same six-then-eight-digit rule.

The reason this is treated more cautiously than a manual transfer's
leading reference: two real transactions from the user's own account
have exactly this same trailing pattern, but only one of them is
actually a transfer. (Fabricated digits below, same real shape — see the
note at the top of this document.)

```
HOLIDAY POT 111222 99944556      -30000   <- a known household account: internal transfer
SPORTS CLUB 111222 99977889      -25000   <- not a known household account: ordinary spend
```

Both look identical structurally — a label, then what could be a sort
code and account number. The only thing that tells them apart is
whether `111222 99977889` matches an account `ledgr` actually knows
about — here it's just a shop's account number that happens to be
shaped the same way.

A manual transfer's leading pattern is treated differently on a
non-match: a 6-then-8-digit prefix at the very start of the field is
classified as `external_account_payment` (spend to a real external
account) even when it doesn't match a household account. An automated
transfer's trailing pattern isn't — a non-match produces no
classification and falls through to the later, more general rules
(leaving `SPORTS CLUB` above as ordinary spend via those rules
instead). Reason: two digit-shaped tokens at the *end* of an arbitrary
label are a weaker signal than a strict prefix at the *start* of the
field, which an ordinary merchant name is far less likely to match by
coincidence.

### Truncation

`NAME`'s 32-character limit isn't a Barclays quirk — it's the OFX
standard's own cap. `STMTTRN`'s `NAME` element is typed
[`GenericNameType`](https://schemas.liquid-technologies.com/ofx/2.1.1/stmttrn.html),
which the OFX common schema defines with
[`maxLength="32"`](https://github.com/aaubry/CGD-NodeJs/blob/master/doc/OFX-Schema/OFX_Common.xsd).
A long label can push the account number past that cap:

```
<STMTTRN>
  <TRNTYPE>OTHER
  <DTPOSTED>20260615000000[-5:EST]
  <TRNAMT>-34.15
  <FITID>10000000000000006
  <NAME>GROUP BILLS ACC 111222 999334
</STMTTRN>
```

If the real account number is `99933445` (8 digits), only `999334` (6
digits) survives truncation here. The automated-transfer check accepts
6-to-8-digit trailing tokens for exactly this reason, and matching
against the household set tolerates a truncated match: an account number
ending in fewer digits than the one on file is still accepted, as long as
it's a genuine *prefix* of the full number (never a fuzzy/partial match
elsewhere in the string) — safe specifically because truncation always
drops trailing digits, never leading ones. The sort code always survives
intact, since it comes right after the label and before the account
number.

#### The missing `STO` marker

Automated transfers in the user's real data never carry a trailing
marker (no `FT`, no `STO`) after the account number, whereas a manual
transfer that happens to be a standing order in the same data does
(e.g. `111222 99977889 STO 111222 9997` — see the exception noted
below). Likely explanation: length. Three automated transfers (same
source account, identical amount, monthly cadence over 7 consecutive
months — a standing-order signature) each have a `NAME` at 30-31
characters, right at the 32-character cap, leaving no room for a
3-character marker once the label and full sort code/account number
are in. Not confirmed by any Barclays documentation (none describes
this trailing format at all), but consistent with every observation so
far, and all three have been confirmed as real transfers to accounts
the user owns or shares.

#### Position in `NAME` correlates with type — but classification doesn't rely on it

In the user's real data, a leading sort code/account number tends to
mean a one-off manual Faster Payment (unique reference each time, e.g.
`SUNGLASSES FT`), and a trailing one tends to mean a recurring standing
order/direct debit (fixed amount, fixed day-of-month, unchanged over
months) — one exception found: a short-reference standing order that
fits `STO` after the account number still uses the leading position.
So the real driver of *position* is reference/label length, not a hard
rule that leading always means manual and trailing always means
automated.

This correlation is incidental and the classification logic doesn't use
it. A manual transfer or a standing order/direct debit can each land in
a household account (internal transfer) or an external one (spend) —
the payment type says nothing about which. What decides it is exactly
what rules 1-2 below check: whether the sort code/account number
matches a known household account, not the payment type or where in
`NAME` it was found.

**Verification**: every distinct sort code/account number pair appearing
in a manual-transfer (leading) transaction across the user's full
transaction history was checked against known accounts. All resolved to
a real account — the user's own accounts, a partner's, or an account
whose identity wasn't obvious from the reference text alone until
checked directly with the user (23 occurrences under one unlabelled
account, since identified as a joint savings account used for
short-term shared costs). Zero false positives: nothing that looked like
a leading sort code/account number turned out to be anything other than
a genuine account.

## Rule ordering

Every transaction is checked against these rules, in order, first match
wins:

1. Leading `<sort> <account>` prefix → internal transfer if it matches a
   household account, otherwise treated as a payment to an external
   account (spend) if money is going out, or ignored if money is coming
   in from an unrecognised source.
2. Trailing `<label> <sort> <account>` suffix → internal transfer *only*
   if it matches a household account; otherwise falls through to the
   rules below rather than making an "external" call the way rule 1 does.
3. Card payment/refund and person-payment patterns, recognised from a
   short suffix at the end of the description (e.g. a card payment always
   ends with a specific 3-letter marker).
4. Barclays' own coarse transaction-type field, as a last resort.
5. Fallback: any remaining transaction where money is going out becomes a
   low-confidence spend entry rather than being silently dropped.

Rules 1 and 2 must run before 3-5: a transfer's `NAME` can otherwise
accidentally end in a token that looks like a card-payment marker, or its
transaction-type field can indicate a standing order/direct debit (into a
household savings account, for instance) — either of which would
misclassify it as spend if account-based matching didn't take priority.

## Known limitations

- Only validated against Barclays' OFX export format. Other institutions
  (once a Barclaycard/pension parser exists) will need their own
  encoding investigated — nothing here generalises across banks.
- The 14-digit `ACCTID` split assumption and the leading-shape's strict
  6+8-digit lengths are both derived from a small number of real
  accounts. A wider variety of account types (e.g. a credit card's
  `ACCTID`, if Barclays ever exports one via OFX) could break these
  assumptions silently — there's no automated check that flags an
  unrecognised `ACCTID` shape beyond falling back to "no sort code" for
  that account.
- A partner/joint account never imported into `ledgr` won't be recognised
  automatically; it has to be added by hand as a **reference household
  account** (see ADR 0008).
- Transfer *pairing* (linking the outbound and inbound legs of the same
  transfer into a single record) requires the **counterpart**
  transaction's own description to use the leading shape specifically —
  it doesn't currently try the trailing shape on that side. So a transfer
  where the trailing shape is what got the *outbound* leg classified as
  an internal transfer will still be excluded from spend correctly, but
  may not get paired if the *inbound* leg's own description also uses the
  trailing shape rather than the leading one. Unpaired legs are still
  individually correct (excluded from spend); only the explicit link
  between the two sides can be missing. Pairing also requires a ±3 day
  window and an exact opposite-signed amount match — a transfer that
  takes longer to land, or where a currency conversion means the two
  legs' amounts don't match exactly, won't be paired either.
