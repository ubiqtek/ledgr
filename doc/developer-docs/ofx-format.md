# OFX Format — Rough Shape

What a Barclays OFX export actually looks like, structurally. Not a full
spec reference — just enough to orient anyone reading about `ledgr`'s OFX
handling. All digits/references below are made up (not real account data),
kept in the same shape as the real thing.

## The rough shape

One file = one account = one `<OFX>` document, wrapping a sign-on section
(ignorable — Barclays' desktop export always reports success) and a
banking section. The part that matters:

```
<OFX>
  <BANKMSGSRSV1>
    <STMTTRNRS>
      <STMTRS>
        <CURDEF>GBP

        <BANKACCTFROM>            <- which account this file is for
          <BANKID>...
          <ACCTID>...
          <ACCTTYPE>CHECKING
        </BANKACCTFROM>

        <BANKTRANLIST>            <- the transactions
          <DTSTART>...
          <DTEND>...

          <STMTTRN> ... </STMTTRN>   <- one block per transaction,
          <STMTTRN> ... </STMTTRN>      repeated as many times as
          <STMTTRN> ... </STMTTRN>      there are transactions
          ...

        </BANKTRANLIST>

        <LEDGERBAL>                <- the bank's own balance, once
          <BALAMT>...
          <DTASOF>...
        </LEDGERBAL>

      </STMTRS>
    </STMTTRNRS>
  </BANKMSGSRSV1>
</OFX>
```

The part worth actually paying attention to is `BANKTRANLIST`: it's just
a flat, repeated list of `STMTTRN` blocks — no nesting, no grouping, one
block per transaction. Each `STMTTRN` is small and self-contained — five
fields: `TRNTYPE` (Barclays' own coarse classification — see
[Transfer Detection — Technical Notes](transfer-detection.md) for why
it's not that useful), `DTPOSTED`, `TRNAMT` (signed, negative = money
out), `FITID` (Barclays' own unique transaction ID, used for
de-duplication), and `NAME` (the only free-text field, capped at 32
characters).

`LEDGERBAL` appears once per file, after the transaction list — it's the
bank's own reported balance as of a point in time, not something derived
by summing the transactions.

## Format note

Barclays' export is OFX 1.x (SGML-flavoured), not OFX 2.x (well-formed
XML) — most tags are never explicitly closed; a closing tag only appears
on aggregates that contain other tags (`</BANKACCTFROM>`, `</STMTTRN>`,
`</OFX>`).
