# 6. No account-type gate on spend derivation — scan every account uniformly

Date: 2026-07-12

## Status

Accepted

## Context

The first implementation of spend ledger derivation (`src/derive.rs`)
gated spend/refund classification on `AccountType::is_spending_account()`
— true for `Current` and `CreditCard`, false for `Savings`, `Pension`,
`Investment`, `Other`. The reasoning at the time: in the real data
examined, only current accounts and credit cards ever produced
spend-shaped transactions; savings accounts only ever saw transfers.
Gating on account type was meant to stop a savings account matching one
of the spend-shaped rules (e.g. a `REPEATPMT` standing order) from being
misclassified as spend when it was really a transfer whose counterpart
hadn't been imported yet.

Revisiting this while scoping Delta: The Gap surfaced two problems:

- The user's real spending accounts are effectively "most of what's
  imported" (current account, an online spending account, a bills
  account, and — once its parser lands — the credit card). A fixed
  per-type allowlist doesn't buy much precision when it already covers
  nearly everything.
- Maintaining a type-based gate implicitly claims to know, in advance,
  every account type that could ever produce real spend. That claim
  doesn't hold once assets like a pension or a mortgage are drawn into
  scope (Delta: The Gap's asset/liability discovery) — those accounts
  might reasonably see occasional real cash movements that aren't pure
  transfers, and a type gate would silently swallow them as
  `OutOfScope` with no signal anything was missed.

Transfer detection (rules 1-2: `NAME` starts `<sort code> <account
no>` matched against the household registry) already independently
identifies and pairs internal movement, regardless of account type. The
account-type gate was therefore redundant with — not a necessary
complement to — the mechanism that actually does the job.

## Decision

Remove the account-type gate entirely. `derive_spend_entries` scans
every account uniformly; classification rules 1-2 (transfer detection
via the household account registry) are solely responsible for keeping
internal movement out of the ledger. `AccountType::is_spending_account()`
is deleted from `src/model.rs`.

## Consequences

- Simpler derivation: one classification path, no per-account-type
  branch to keep in sync with which types "count."
- Correctness now rests entirely on transfer pairing/reconciliation
  being reliable. If a savings/pension/investment account ever produces
  a spend-shaped transaction that transfer detection doesn't recognise
  (e.g. its counterpart account was never imported and isn't in the
  household config), it will be classified as spend rather than silently
  dropped — arguably the safer failure mode, since it stays visible in
  the ledger (at whatever confidence the matching rule gives it) instead
  of disappearing with no trace.
- No behavioural change expected against the currently-imported real
  data (savings accounts have only ever shown transfers there), so this
  is safe to ship without re-validating against real data — worth
  re-checking once pension/mortgage accounts are actually imported
  (Delta: The Gap's asset/liability discovery).
