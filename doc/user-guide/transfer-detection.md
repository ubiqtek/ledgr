# Transfer Detection

Part of **[Spend Analysis](spend-analysis.md)**: how `ledgr` tells "money
moving between your own accounts" apart from "money you actually spent",
so internal transfers don't get double-counted as spend.

## The problem

Say you have a current account and a separate bills account, and you send
£500 from one to the other every month to cover direct debits. Barclays (or
any bank) records this as two completely separate transactions — a £500
debit on the current account, and a £500 credit on the bills account. If
`ledgr` treated every outgoing transaction as spend, that £500 transfer
would show up as spend, even though it never left your household.

## How ledgr recognises "your own account"

Every account you import is automatically registered as a **household
account** — `ledgr` remembers each account's sort code and account number.
When it processes a transaction, it looks at *who the money is going to or
coming from*, and checks whether that matches a household account. If it
does, the transaction is an **internal transfer**, not spend.

Banks encode "who the money is going to" as text inside the transaction
description — there isn't a clean, separate field for it. `ledgr` looks for
a sort code and account number embedded in that description. In practice
this shows up in two shapes:

- **At the start**: the description begins with the sort code and account
  number, e.g. a transaction named something like
  `"123456 87654321 UTILITIES"`.
- **At the end**: a label first, then the sort code and account number,
  e.g. `"HOLIDAY FUND 123456 87654321"`. Banks sometimes truncate long
  descriptions, which can shorten the account number — `ledgr` still
  matches it against the full account number on file.

If the sort code + account number match one of your own accounts, it's a
transfer and gets excluded from spend. If they match neither your accounts
nor a known partner/joint account, it's treated as a payment to someone
external — which *is* spend.

## What this means for your numbers

- Transfers between your own accounts never inflate your spend total,
  however many accounts you have or however money moves between them.
- If a transfer looks like it's being counted as spend, the most likely
  cause is that `ledgr` doesn't yet know the destination account's sort
  code/account number — this can happen for accounts that haven't had a
  fresh statement imported recently, since that's currently how `ledgr`
  learns an account's details. Re-importing a file for that account fixes
  it.
- A partner's or family member's account that you never import statements
  for can still be registered as a known household account by hand, so
  transfers to it are recognised too.

For the technical detail — the exact matching logic, real (anonymised)
examples from Barclays OFX exports, and known edge cases — see the
developer documentation:
[Transfer Detection — Technical Notes](../developer-docs/transfer-detection.md).
