# Spend Analysis

`ledgr` builds a **spend ledger** from your raw imported bank transactions —
this is the layer that answers "how much did I actually spend, and on
what?" without being thrown off by money that just moved between your own
accounts.

## Why not just sum the transactions?

If you have more than one account — say a current account, a joint bills
account, and a savings pot — money regularly moves between them: you top up
the bills account from your current account, or sweep spare cash into
savings. Each of those movements shows up as **two** raw transactions (one
leaving an account, one arriving in another), even though no real spending
happened. Summing every outgoing transaction across all your accounts would
double-count that movement as if it were spend, badly overstating what you
actually spent.

`ledgr` solves this by deriving a separate **spend ledger** from your raw
transactions:

- Real spending (shops, direct debits, card payments, payments to other
  people) becomes a **spend entry**.
- Money moving between your own accounts is recognised as an **internal
  transfer** and excluded from spend entirely.
- Refunds and reimbursements are linked back to what they're paying for,
  rather than counted as income.

How `ledgr` tells "your own account" apart from everyone else's is its own
topic — see **[Transfer Detection](transfer-detection.md)**.

## What counts as spend

Every raw transaction is classified once, during `ledgr import`, using a set
of rules applied in order:

1. **Transfer to a household account** → excluded (internal transfer, not
   spend).
2. **Payment to an account that isn't yours** → spend.
3. **Card payment** (`CPM` transactions) → spend.
4. **Card refund** (`CRM`/`CRE`/`BCC` transactions) → a negative spend entry
   (reduces spend for that merchant, not counted as income).
5. **Payment to a named person** (Faster Payment, `FT`) → spend.
6. **Reimbursement from a named person** → a negative spend entry, same
   reasoning as a card refund.
7. **Direct debit / standing order / other outgoing payment** → spend.
8. **Salary or cash withdrawal** → out of scope for now (income has its own,
   separate ledger — see the note below).
9. Anything that doesn't match the above, but is money leaving an account →
   still recorded as a low-confidence spend entry, so nothing silently goes
   missing. It's flagged for review rather than dropped.

Money *arriving* that doesn't match a known pattern is left out of scope
entirely — guessing "not spend" for unrecognised inbound money is safe;
guessing "not spend" for unrecognised outbound money would make your spend
total wrong with no way to notice.

## What's not in scope yet

- **Income** isn't summed yet — that's the next piece being built (The Gap:
  income − spend).
- **Categorisation** (groceries vs eating out vs subscriptions) isn't
  applied yet — spend entries exist, but by-category breakdowns are a
  follow-up.
- A review screen for low-confidence spend entries (the "anything that
  doesn't match" case above) is planned but not built yet — for now, low
  confidence entries just sit in the ledger at a lower trust level.
