# 8. Reference household accounts — config-only, never imported, no balance data

Date: 2026-07-12

## Status

Accepted

## Context

Transfer detection (`src/derive.rs`, ADR-adjacent work in Delta: Spend
Ledger) needs a **household registry** — the set of `(sort code,
account number)` pairs that count as "our own accounts" — to recognise
when money moving out of one account is really just landing in another
one of the household's, not real spend. Until now, that registry was
built entirely from `accounts` rows created by actually importing a
statement.

That breaks down for accounts the user will never import — the
concrete case that surfaced it: the user's wife has her own current
account. Her credit card will be imported normally (Credit Card
Transaction Import), but her personal bank account never will be —
there's no statement file, no `ImportFileParser`, nothing to import.
Without some way to register her account's identity, any transfer from
the user's accounts to hers (or vice versa) would be misclassified as
external spend/income, when it's really just money moving within the
household.

`Config.household_accounts` (`src/config.rs`) already existed for
exactly this — a hand-edited list of `(sort_code, account_number,
label)` entries, folded into the household set alongside imported
accounts' own `sort_code`/`account_number` at derivation time
(`derive_spend_entries`'s `extra_household_accounts` parameter). What
hadn't been settled was the terminology and the guarantee: is this a
"real" account that just happens to have no transactions yet, or
something categorically different?

## Decision

Name this concept a **Reference Household Account**: a household
account `ledgr` knows about *by reference only* — sort code and account
number, nothing else — used solely to make transfer detection complete.
It is never backed by an `accounts` table row, will never have a
balance, a transaction history, or an import — and that absence is
permanent by design, not a temporary gap waiting to be filled by a
future import.

This is deliberately kept separate from `accounts`:

- `accounts` rows represent things `ledgr` has actual data for (real
  transactions, a real or reconstructable balance).
- A reference household account represents household *membership*
  only — it exists purely to make the transfer-detection matching rule
  (`household_contains()` in `src/derive.rs`) correct, with nothing
  else attached.

`ledgr status` was updated to list configured reference household
accounts in a section separate from real accounts, explicitly labelled
as carrying no balance/transaction data, so their absence from the main
account list isn't mistaken for a bug.

## Consequences

- No schema change: `Config.household_accounts` already had the right
  shape (`HouseholdAccountRef { sort_code, account_number, label }`);
  this ADR just names and settles the concept, and fixes `ledgr
  status`'s display to match.
- Adding a reference household account is still a hand-edit of
  `~/.config/ledgr/config.toml` — no CLI setter yet (same gap as
  `account_names`/`household_accounts` generally). Worth a `ledgr
  add-household-account` command if this needs to happen often (e.g.
  more family accounts).
- A **manual spend entry** on a **proxy account** (see
  `doc/domain/ubiquitous-language.md`, raised 2026-07-12) is the
  companion mechanism for the same underlying problem — a reference
  household account excludes a partner's *transfers* from spend, but
  her own spend on her own (unimported) accounts still needs to be
  captured some other way, since `ledgr` has no transaction data for
  it at all. The two are complementary, not overlapping: reference
  household accounts stop money crossing *into* her account from being
  double-counted; a proxy account is how her spend gets recorded at
  all.
- If a reference household account's sort code/account number turns
  out to be wrong (transcription error, or the account is closed and
  replaced), there's no validation against it — an incorrect entry
  would silently either miss real transfers or (far less likely, given
  the address space) coincidentally match something unrelated. Some
  correctness rests on the user typing the right digits.
