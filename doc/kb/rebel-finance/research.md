# Rebel Finance — Categorisation Method Research

Research for the `ledgr` spending-categorisation system. Compiled 2026-07-11 via web
research. Confidence levels are flagged throughout: **[verified]** = direct evidence
from their own pages; **[inference]** = reasonable read of the material; **[unverified]**
= could not confirm.

## Overview: who / what Rebel Finance is

**Rebel Finance** is the personal-finance brand of **Alan and Katie Donegan** (UK-based),
who publish under "**Rebel Donegans**" / "**The Donegans**". Their flagship free programme
is **Rebel Finance School (RFS)** — a 10-week financial-education course delivered
annually (e.g. RFS 2024, RFS 2026) with accompanying course notes, YouTube videos, and
downloadable spreadsheet tools. **[verified]**

The method is a **FIRE-flavoured (Financial Independence / Retire Early)** framework built
around one central metric — "**the gap**" (income minus spending) — rather than a named
percentage rule like 50/30/20. It is closer in spirit to zero-sum / "grow the gap" FIRE
budgeting than to strict envelope budgeting. **[verified for the gap concept; inference
that it is deliberately *not* a 50/30/20 system — they never cite one]**

Primary sources found:
- Spending tracker template — https://rebeldonegans.com/finance/take-control/spending-tracker-template/
- RFS Week 1 course notes ("Track Your Spending and Find Your Gap") — https://rebeldonegans.com/finance/rfs/course-notes/week-1/
- RFS full course notes / roadmap — https://rebeldonegans.com/finance/rfs/course-notes/
- YouTube channel "The Donegans / Rebel Donegans" — https://www.youtube.com/@rebeldonegans

## Categorisation method

### Top-level spending categories (verbatim)

RFS Week 1 recommends a **maximum of ~10 categories** so the data stays analysable. The
categories they use themselves, with the sub-categories/examples they give: **[verified —
these are quoted from the Week 1 notes]**

| Category | Examples / sub-categories given |
|---|---|
| **Accommodation / Housing** | Airbnb, hotels (and, by their classification rules, mortgage repayments, rent) |
| **Eating out** | Restaurants, "discretionary naughties" |
| **Groceries** | Food for home cooking |
| **Education** | Books, courses, videos |
| **Transport** | Car costs (insurance, servicing), flights, trains, public transport, car hire |
| **Clothes** | Clothing and footwear |
| **Entertainment** | Cinema, shows, spas, fun activities |
| **Health** | Prescriptions, gym, dentist, opticians |
| **Misc** | Cash, gifts, technology, other |

Design principles they state explicitly: **[verified]**
- Keep it to **~10 top-level categories max** — enough to see patterns, not so many that
  analysis is impossible.
- **Sub-categories are optional** — keen users can nest (e.g. Transport → flights / trains
  / car), but this is not required.
- Goal of tracking is to **find your biggest three spending categories** and one "money
  leak" to cut. It is diagnostic, not prescriptive-budget-enforcement.

Note: the downloadable **Spending Tracker spreadsheet** ships with **customisable**
categories via a dropdown; example categories seen in the template include *Supermarket,
Entertainment, Working Lunch, Coffee*. So the ~10 above are a **recommended default set,
not a fixed enum** — users are expected to edit them. **[verified]**

### The "Gap" — the organising concept

**Gap = Income − Spending.** **[verified]** Everything hangs off this:
- Positive gap → you have choices; grow the gap → grow your choices; invest the gap →
  "buy your freedom".
- The point of categorisation is to understand and grow the gap, not to hit category
  budgets.

### Classification rules (what is / isn't "spending")

These are the load-bearing rules for `ledgr` — how a transaction maps to spending vs
not-spending. **[verified from Week 1 notes + spending-tracker page]**

**Counts as spending:**
- Insurance (health, life, mortgage protection)
- Utility bills (electricity, water, etc.)
- **Mortgage repayments** — counted as spending even on an owner-occupied home
- Rent

**Does NOT count as spending (excluded from the gap outflow):**
- **Transfers between your own accounts** — moving money personal-account-to-personal-account.
- **Pension / SIPP / ISA / investment contributions** — "Pensions are not spending… they
  are your Gap in action." Recorded as **transfers/investments**, not spending, because the
  money is kept and invested, not consumed.
- **Credit-card repayments** — mark as a **transfer**, not spending, to avoid
  double-counting (the original purchases were already categorised as spending when made).

**Sinking funds** (saving up for a known future cost — holiday, new car): they give **two
acceptable treatments**, user's choice: **[verified]**
1. Count the **monthly contribution** as spending as you save, **or**
2. Count the **actual purchase** as spending when it happens.
The app should let the user pick one convention and apply it consistently (mixing the two
double-counts).

### Irregular / annual expenses
Handled via the sinking-fund mechanism above (save monthly, or record the lump when it
lands). No separate "annualise everything" rule was found. **[verified for sinking funds;
inference that this is their answer to annual/irregular costs]**

### Joint vs personal spending
Not given a formal splitting rule in the public Week 1 material. Their categories are
household-level, and Week 5 ("How to Talk About Money") covers running a **monthly finance
meeting** as a household — implying spending is tracked at the household/joint level rather
than split per-person. **[inference — no explicit bill-splitting rule found]**

### Savings vs investment
A **core distinction** in the framework (Week 6: "most people think they are investing when
they are not"). **[verified]**
- **Cash savings** (emergency fund, buffers, sinking funds) — safety/liquidity, part of
  net worth but not "investing".
- **Investment / "Freedom Fund"** — assets deliberately put to work for long-term growth
  (index funds via ISA/SIPP/pension). This is where the gap is deployed.
- Both live on the **separate Net Worth Tracker** (monthly asset snapshots), *not* in the
  cash-flow Spending Tracker. Defined-benefit pensions and investment accounts belong on
  the Net Worth Tracker too.

### The financial roadmap (priority order)
Their prescribed sequence, which contextualises the categories: **[verified]**

**Positive gap → build safety (starter buffer, then fuller emergency fund) → attack
expensive debt → invest (build the Freedom Fund).**

Course arc for reference (10 weeks): W1 track spending/find gap · W2 net worth & "four
buckets" · W3 money mindset · W4 compound interest · W5 talking about money / monthly
meeting · W6 start investing / Freedom Fund · W7 index funds · W8 implementation · W9
retirement number · W10 sustainable drawdown. **[verified]**

### "True cost" framing (not categorisation, but part of their method)
They reframe recurring spend as forgone investment: multiply a **monthly** cost by **194**,
or a **weekly** cost by **840**, to estimate its 10-year invested opportunity cost (e.g.
£15.99/mo × 194 ≈ £3,102). Potentially a nice `ledgr` feature but **not** part of the
categorisation taxonomy. **[verified]**

## Open questions / gaps — confirm against the real spreadsheet

The user said they can obtain the actual spreadsheet. These are the points to verify there,
because I could **not** confirm them from public material:

1. **Exact default category list in the shipped spreadsheet.** The ~10 categories above are
   from the *course notes*; the spreadsheet's built-in dropdown may differ slightly (it
   showed Supermarket / Working Lunch / Coffee etc.). Confirm the canonical default enum.
2. **Fixed vs user-editable categories.** Evidence says editable — confirm whether `ledgr`
   should ship a fixed taxonomy or a seed list the user extends.
3. **Sub-category structure.** Is there a formal two-level hierarchy in the sheet, or is
   nesting purely ad-hoc? Confirm the data model (flat vs parent/child).
4. **Income categorisation.** The material is almost entirely spending-side. How income is
   categorised (salary, side income, refunds, interest) is unconfirmed.
5. **Transfer / investment handling in the sheet mechanics.** Confirmed *conceptually*
   (pensions/CC-payments = transfers, excluded from gap). Confirm how the sheet *flags* a
   row as transfer vs spending (a category value? a separate column?) — this drives the
   `ledgr` transaction schema.
6. **Sinking-fund convention.** Two options exist; check which (if either) the sheet
   defaults to, so `ledgr` picks a sensible default.
7. **Joint/personal split.** No public rule found. Confirm whether the sheet supports
   per-person tagging or is purely household-level.
8. **Data Converter / CSV import mapping.** The sheet has a "Data Converter" for bank CSVs
   and supports batch-categorising by merchant name — worth inspecting for `ledgr`'s import
   + auto-categorisation rules.

## Sources

- [Rebel Finance — Spending Tracker Template](https://rebeldonegans.com/finance/take-control/spending-tracker-template/)
- [RFS Week 1 — Track Your Spending and Find Your Gap](https://rebeldonegans.com/finance/rfs/course-notes/week-1/)
- [RFS — Full Course Notes / Roadmap](https://rebeldonegans.com/finance/rfs/course-notes/)
- [The Donegans / Rebel Donegans — YouTube](https://www.youtube.com/@rebeldonegans)
- [RFS 2026 Week 6 — How Do I Start Investing? (YouTube)](https://www.youtube.com/watch?v=1Dv9MAXIOSI)
- [Rebel Finance School — The Rebel School](https://therebelschool.com/programmes/finance/)
