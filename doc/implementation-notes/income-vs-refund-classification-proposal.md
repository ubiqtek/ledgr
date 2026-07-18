# Income vs Refund classification — proposal

**Status: proposal — nothing here is agreed yet.** Resolves the
inconsistency between the stated domain principle ("Reimbursements and
Refunds ... never treated as income", doc/domain/ubiquitous-language.md)
and the current implementation, which routes only one narrow case
(inbound person `FT` payments) to the spend ledger while credit card
cashback and other inbound credits land in the income ledger.

## The test

The Gap is external money in minus external money out. An inbound
transaction distorts the Gap in one of two ways if misfiled: counted as
income when it merely undoes recorded spend, it inflates both sides;
filed as a refund when nothing in the spend ledger corresponds to it,
it understates spend that genuinely happened. The test therefore hinges
on the spend ledger, not on intuitions about whether money feels "new".

Apply two questions in order to any inbound transaction that has
already been ruled out as an internal transfer:

1. **Reversal** — does this inflow exist *because of* a prior outflow
   from the household, and is it bounded by it? (A refund never exceeds
   what was paid; it is caused by the payment.)
2. **Ledger visibility** — did that original outflow pass through the
   household's accounts, i.e. does it appear (or would it appear, once
   imported) in the spend ledger?

**Yes to both → Refund/Reimbursement**: a sign-reversed spend-ledger
entry, linked to the original spend entry where one can be found,
`refunds_spend_entry_id` NULL where it cannot (cashback, claim
payouts). **No to either → Income.**

This is not a new principle. Question 1 restates the established
"Reimbursements and Refunds" definition ("inbound money that pays back
earlier spend"). Question 2 makes explicit what "earlier spend" means:
spend the ledgers can see. Money that left the household invisibly —
tax deducted at source before net pay ever arrived — was never spend in
ledgr's terms, so its return is not a reversal of anything recorded.

One case fits neither bucket: a **loan advanced to the household**. It
crosses the boundary inward but creates an equal obligation outward —
counting it as income inflates the Gap with money that must be given
back. Recommended handling below; whether it needs a formal third
classification is an open question.

## The seven cases

| # | Transaction | Reversal? | Ledger-visible? | Classification | Reasoning |
|---|---|---|---|---|---|
| 1 | Salary (`AZIMO LTD ... BGC`) | No | — | **Income** | Reverses nothing; the canonical inflow. |
| 2 | Barclaycard cashback (type `Other`) | Yes — a percentage rebate on card spend | Yes — that spend is in the spend ledger | **Refund** | Money already spent partially coming back; unlinkable to specific purchases, which `Refund` already tolerates (`refunds_spend_entry_id` NULL). |
| 3 | SimplyHealth payouts (`SIMPLYHEALTH ... BGC`) | Yes — reimburses dental/optical bills | Yes — those bills were household spend | **Refund** | Structurally identical to cashback: caused by and bounded by recorded spend, no single linkable original. |
| 4 | HMRC PAYE credit (`HMRC PAYE ... BGC`) | Yes — over-collected tax returning | **No** — PAYE is deducted before net pay arrives; the tax never appeared as spend | **Income** | Filing it as a refund would net down spend that was never recorded, understating true spend. See below. |
| 5a | Settling-up (`F Crichton NORWAY CAR BGC`, likely `ARIA ... RE CHASE BGC`) | Yes — pays back their share of a shared cost | Yes — the household paid the shared cost as spend | **Refund** (reimbursement) | The existing inbound-`FT` reimbursement case, arriving via BGC instead. |
| 5b | Loan (`Wendy Barritt LOAN ETC BGC`) | Depends on direction — see open questions | — | **OutOfScope** (provisional) | A loan advanced is a liability inflow, not income; a repayment of money the household lent is a reimbursement. Direction is unknowable from the description. |
| 5c | Genuine no-strings gift | No — nothing paid out caused it | — | **Income** | The domain doc already lists gifts as income. Distinguishing a gift from settling-up needs human review — see open questions. |
| 6 | Book sale (`WORLD OF BOOKS LTD ... BGC`) | No — proceeds of a market sale, not a payback of the purchase price | — | **Income** | Realising residual value from an asset is a new economic event; the original book purchase stays as spend. Close call — see below. No third bucket warranted at these amounts. |
| 7 | Lottery win (`ALLWYN ENT LTD ... BGC`) | No — winnings are not bounded by or a return of the stake | — | **Income** | The ticket stays as spend; the prize is a windfall, arbitrarily unrelated to it in size. |

**Close calls.** #6 is the genuinely arguable one: selling a
recently-bought item at near its purchase price looks refund-shaped.
The test still says Income because the sale is caused by a buyer's
decision, not by the purchase — but a household that declutters heavily
could reasonably prefer a "disposal proceeds" treatment that nets
against spend. #5b's direction problem is unresolvable from data alone.

## The HMRC/cashback tension, resolved

The gut reactions — cashback is not income, the tax refund is — appear
inconsistent (both are "money the household already parted with, coming
back") but the test vindicates both, on grounds neither gut articulated:

- **Cashback** reverses spend that **is in the spend ledger**. Counting
  it as income double-inflates the Gap: spend stays overstated and
  income is padded with recycled money.
- **The PAYE refund** reverses a deduction that **never touched the
  ledgers** — only net pay ever arrived. There is no recorded spend for
  it to net against; to the household's books it is new external money.

The distinction is ledger visibility, not "who sent it" or "how new it
feels". Corollary worth noting: a refund of a **self-assessment** tax
bill previously *paid from a tracked account* would fall the other way
— that payment was recorded spend, so its refund is a Refund. Same
counterparty, opposite classification, and the test handles it without
a special case.

## Proposed classification rules

All new rules slot **before** the generic `BGC` suffix rule in
`classify()` (src/derive.rs) — first match wins, and `bank_giro_credit`
becomes the residual bucket. All apply only when `amount_minor > 0`.

| Order | Rule | Match | Result | `rule_name` | Confidence |
|---|---|---|---|---|---|
| 1 | Employment income | NAME starts with a configured employer name — new config list `income_sources`, mirroring `household_accounts` | Income | `employment_income` | 0.95 |
| 2 | Merchant cashback | TRNTYPE `Other`, positive amount (i.e. the existing `credit_card_cashback` match, reclassified) | Refund | `cashback` | 0.85 |
| 3 | Claim reimbursement | NAME starts with a configured insurer/claim payer name — config list `claim_payers` (e.g. `"SIMPLYHEALTH"`) | Refund | `claim_reimbursement` | 0.85 |
| 4 | Tax refund | NAME starts `"HMRC PAYE"` | Income | `tax_refund` | 0.8 |
| 5 | Person BGC reimbursement | Inbound BGC from a configured known external person — config list `known_people` (names matched like `household_accounts[].name`) | Refund | `person_reimbursement` | 0.6 |
| 6 | Residual BGC | Existing `bank_giro_credit` rule, unchanged in behaviour | Income | `bank_giro_credit` | drop 0.75 → **0.5** |

Notes:

- Rule 6's confidence drops because once rules 1–5 absorb the
  explicable cases, whatever remains genuinely needs human review —
  0.75 currently overstates how much is known about a bare BGC credit.
- Rule 5 mirrors the existing inbound-`FT` `reimbursement` rule (0.6)
  for consistency: an unexplained inbound payment from a known person
  defaults to settling-up, reclassified manually when it is actually a
  gift. This is in tension with the stated instinct that the Aria
  payment is income — see open questions.
- `Classification::Refund` currently has no `confidence` field (0.7 is
  hardcoded at the insert site). Rules 2/3/5 need per-rule confidence,
  so the variant grows a `confidence: f64` — a mechanical change.
- The loan case gets **no rule**: `"LOAN"` in free text is too weak a
  signal to act on. It falls through to rule 5 (if the sender is a
  configured known person) or rule 6, and is corrected by hand.

## New domain terms — needs to be agreed with the user

Candidates only; none are recorded in the ubiquitous-language doc by
this proposal.

1. **Gift** — a no-strings inbound payment from outside the household;
   income, distinct from a reimbursement. Currently only an example
   under Income Ledger, not a term.
2. **Loan** — money advanced to the household creating an obligation;
   neither income nor refund. Accepting it implies at least an
   OutOfScope convention now and possibly liability modelling later.
3. **Known Person** — a named external individual registered in config
   so their payments classify consistently (the external counterpart of
   Reference Household Account). Needed if rule 5 is accepted.
4. **Income Source** — a configured employer/payer name driving the
   `employment_income` rule.
5. **Cashback** — worth adding as a named example under the existing
   Reimbursements and Refunds entry (not a new concept, but its current
   miscoding as income suggests the entry should name it explicitly).
6. No pull-forward of **Category** is required: everything here is
   decided at classification time, ledger membership only. "Windfall"
   vs "wages" vs "gift" inside the income ledger remains the deferred
   taxonomy question, as the domain doc already states.

## Open questions for the user

1. **Aria and person BGC defaults** — rule 5 defaults known-person
   inbound payments to reimbursement (matching the FT rule), but the
   stated instinct was that Aria's payment is income. Which default is
   less wrong for this household: reimbursement-until-proven-gift, or
   gift-until-proven-reimbursement? (The test itself is neutral — it
   depends on facts the data does not carry.)
2. **Known-person registration** — should rule 5 require registered
   names (like `household_accounts`), or attempt a generic
   "looks like a personal name" pattern? Registered names are proposed;
   a generic pattern risks classifying company payments as personal.
3. **The Wendy Barritt LOAN payment** — was this a loan *to* the
   household (out of scope / liability), or repayment of money the
   household previously lent out (reimbursement, if that lend was
   recorded as spend)?
4. **Book-sale proceeds** — is Income acceptable, or does asset
   disposal deserve its own treatment (e.g. netting against spend)? At
   ~£50 occurrences, Income is proposed as the pragmatic answer.
5. **HMRC rule scope** — match `"HMRC PAYE"` only (proposed), or all
   HMRC credits? A self-assessment refund of a bill paid from a tracked
   account should be a Refund, so a blanket HMRC→Income rule would be
   wrong for that case.
6. **Residual BGC confidence** — agree the drop from 0.75 to 0.5 once
   the specific rules exist?
