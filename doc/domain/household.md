# Household

The full story of the term **Household** — what it means in `ledgr`,
the alternatives considered, and why it was adopted. Summary entry:
[ubiquitous-language.md](ubiquitous-language.md).

## The concept

`ledgr` needs a name for the accounting entity whose money it tracks —
the boundary that separates *internal* money movement from *real*
money events:

- Money moving **within** the boundary (current account → credit card,
  bills account → spending account, top-ups to savings) is an
  **internal transfer**. It appears in neither the spend ledger nor
  the income ledger; counting it would double-count every purchase
  paid for by a matching transfer.
- Money **crossing** the boundary is a real event: outbound to a
  merchant or person is **spend**; inbound is **income**.

This is the personal-finance equivalent of "Company" / "Legal Entity"
/ "Accounting Entity" in the corporate world: the corporate entity is
the Company and its accounts are the company's accounts; here the
entity is the **Household** and its accounts are **household
accounts**.

The concept has teeth because of a real edge case: a partner's account
that `ledgr` does not import. Transfers to it are arguably internal
(shared finances), not spend — so membership of the household can't
simply be "the accounts ledgr has imported"; it needs an explicit,
user-maintained boundary.

## How the term arrived (provenance)

1. **Introduced without agreement.** The assistant extrapolated
   "household" from the Rebel Finance research (their tracking is
   household-level) and used it to name ADR 0005 ("household ledger")
   without the user ever having used the word. Rejected — not because
   the word was wrong, but because it smuggled an undecided scope
   assumption into the design. This incident is what prompted the
   ubiquitous language doc and the CLAUDE.md rule about not coining
   domain terms unilaterally.
2. **Reconsidered on its merits.** The underlying concept still needed
   a name ("which counterparty accounts are us?"), and the user
   reopened the question: household is not a bad term.
3. **Researched.** See evidence below.
4. **Adopted deliberately**, 2026-07-11.

## Alternatives considered

| Term | Camp | Verdict |
|---|---|---|
| **Own accounts** | informal (Firefly III import docs say "transfers between own accounts") | Names the *list*, not the *entity*; turns dishonest the moment a partner's un-imported account is listed ("own" isn't strictly true). Kept only as a casual synonym. |
| **Asset accounts** (+ liability) | double-entry: Firefly III, GnuCash, ledger/beancount | Elegant — the boundary becomes *structural*: a transfer is definitionally a transaction between two of your own asset/liability accounts; spend is asset → expense account; income is revenue → asset. But it imports double-entry vocabulary before `ledgr` is double-entry, and it still doesn't answer the partner-account membership question. Noted as the likely *mechanical* form of the boundary if double-entry lands (see the Double-Entry Accounting delta). |
| **On-budget / off-budget (tracking) accounts** | YNAB | A different concept entirely — which accounts participate in budgeting, not whose money it is. Not our word. |
| **Reporting entity** | formal accounting | Precisely the right concept, far too corporate for a personal tool. |
| **The books** | accounting metaphor | Charming ("is this account on the books?") but vague; doesn't name the human unit. Survives as informal usage: the household's books. |
| **Family Accounts** | proposed during the discussion | Warmer, but narrower — a single person or two housemates sharing bills aren't a "family" but are a household (the word economists chose for exactly this reason). Also names the account list rather than the entity. Rebel Finance use "family" only colloquially (getting the kids involved), never as the unit. |
| **Household** | economics ("household sector"), UK personal-finance guidance (MoneyHelper's "household budget"), Rebel Finance | **Adopted.** |

## Evidence for "household"

- **Rebel Finance use it in exactly this sense.** Week 1 course notes:
  *"Katie and I look at our total combined spending so we link all of
  our accounts into one Emma account for both of us. It's up to you
  whether you do this individually or as a **household**."* The word
  names the tracking unit — and they track jointly as a couple.
- **Economics** calls this unit the household ("household sector");
  UK guidance (e.g. MoneyHelper) frames everything as the "household
  budget" — the entity whose income and expenses are tracked as one.
- **The corporate analogy resolves the "list vs entity" question**:
  name the entity, derive the account list from it (household →
  household accounts), exactly as company → company accounts.

## The "household chores" objection

Concern raised: "household" colloquially evokes household expenses
(food, cleaning products), so the entity name could collide with a
spending *category*. Checked against Rebel Finance's own taxonomy:
their categories are Accommodation/Housing, Eating Out, Groceries,
Education, Transport, Clothes, Entertainment, Health, Misc — **no
"household" category exists**; cleaning products and food live under
Groceries/Misc. Resolution: the taxonomy must never contain a category
named "Household"; if a home-supplies category is ever wanted, call it
**"Home"**.

## Consequences

- The membership list (household accounts) is user-maintained config:
  imported accounts are members automatically; known-but-not-imported
  accounts (e.g. a partner's) can be added by sort code + account
  number. Exact membership is an open question in the
  [spend ledger design](../implementation-notes/spend-ledger-design.md).
- Transfer detection classifies a raw transaction as internal when its
  counterparty account is a household account.
- If double-entry accounting is introduced later (future delta), the
  membership test is expected to become structural — household
  accounts are the asset/liability accounts of the household's books —
  rather than a list.

## Sources

- [RFS Week 1 — Track Your Spending and Find Your Gap](https://rebeldonegans.com/finance/rfs/course-notes/week-1/)
- [MoneyHelper — Create a household budget](https://www.moneyhelper.org.uk/en/blog/everyday-money/create-a-household-budget-for-your-family)
- [Household budget — definition](https://diversification.com/term/household-budget)
- [Firefly III — accounts concept](https://docs.firefly-iii.org/explanation/financial-concepts/accounts/) and [transaction types](https://docs.firefly-iii.org/references/firefly-iii/transaction-types/)
