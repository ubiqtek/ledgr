-- ledgr's initial schema.
--
-- Storage is plain SQLite. Relationships that don't fit a single table
-- (category hierarchies, transfer pairs, refund links) are modeled as
-- self-referencing columns on the row that needs the relation
-- (`categories.parent_id`, `transfer_entries`' counterpart_* columns,
-- `spend_entries.refunds_spend_entry_id`) rather than a general-purpose edge
-- table or a dedicated graph database — every one of these relations is at
-- most one-to-one from the referencing row's side, which a column captures
-- more simply than an edge table's extra join.
--
-- Money is stored as integer minor units (e.g. pence) to avoid floating
-- point drift.

CREATE TABLE IF NOT EXISTS accounts (
    id             INTEGER PRIMARY KEY,
    name           TEXT NOT NULL,
    institution    TEXT,
    account_type   TEXT NOT NULL CHECK (account_type IN (
                       'current', 'savings', 'credit_card', 'pension',
                       'investment', 'other'
                   )),
    currency       TEXT NOT NULL DEFAULT 'GBP',
    -- Sort code + account number, populated from the import format's own
    -- account identity (e.g. OFX BANKACCTFROM) where available. Used by spend
    -- ledger derivation to recognise the counterparty of an internal transfer
    -- (Barclays packs "<sort code> <account no>" into a transaction's NAME —
    -- see doc/kb/ofx/structure.md) — not just to display the account.
    sort_code      TEXT,
    account_number TEXT,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Self-referencing edge for category hierarchy (e.g. "Groceries" under "Living Costs").
CREATE TABLE IF NOT EXISTS categories (
    id        INTEGER PRIMARY KEY,
    name      TEXT NOT NULL,
    parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);

-- One row per imported file, so re-importing the same file is a no-op.
CREATE TABLE IF NOT EXISTS imports (
    id           INTEGER PRIMARY KEY,
    account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    source_path  TEXT NOT NULL,
    file_hash    TEXT NOT NULL UNIQUE,
    imported_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    period_start TEXT,
    period_end   TEXT
);

-- Raw transactions are immutable evidence, never categorised directly — see
-- doc/implementation-notes/spend-ledger-design.md. Categorisation lives on
-- the derived spend_entries below.
CREATE TABLE IF NOT EXISTS transactions (
    id               INTEGER PRIMARY KEY,
    account_id       INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    import_id        INTEGER REFERENCES imports(id) ON DELETE SET NULL,
    posted_at        TEXT NOT NULL,
    amount_minor     INTEGER NOT NULL,
    currency         TEXT NOT NULL,
    description      TEXT NOT NULL,
    raw_description  TEXT,
    -- OFX TRNTYPE (or equivalent), e.g. 'OTHER', 'DIRECTDEBIT', 'DIRECTDEP',
    -- 'CASH'. Barclays never emits 'XFER' for transfers — see the OFX KB
    -- article — so this alone can't identify a transfer, but it does
    -- disambiguate direct debits/standing orders/cash/income patterns for
    -- spend ledger derivation.
    trn_type         TEXT,
    external_id      TEXT,
    -- Catch-all for import-format detail that doesn't fit any field above
    -- (e.g. a credit card statement's line-item extras). Unpopulated by
    -- most parsers; nullable so it costs nothing when unused.
    notes            TEXT
);

CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_posted_at ON transactions(posted_at);

-- A file re-imported under a different file hash (e.g. re-saved from
-- the bank's website) must not duplicate transactions the FITID/external_id
-- already identifies as the same one within an account. Partial (excludes
-- NULL external_id) since formats like generic CSV carry no stable ID.
CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_account_external_id
    ON transactions(account_id, external_id)
    WHERE external_id IS NOT NULL;

-- Balance "anchors" reported by the bank itself (OFX LEDGERBAL and
-- equivalents), one per import that carries one. The transaction list in
-- a given import often doesn't reach back to account opening, so a
-- balance can't be reliably derived by summing transactions alone; it must
-- be reconstructed from the nearest anchor plus the transactions between
-- that anchor and the target date.
CREATE TABLE IF NOT EXISTS balance_snapshots (
    id            INTEGER PRIMARY KEY,
    account_id    INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    import_id     INTEGER REFERENCES imports(id) ON DELETE SET NULL,
    balance_minor INTEGER NOT NULL,
    as_of         TEXT NOT NULL,
    UNIQUE (account_id, as_of)
);

CREATE INDEX IF NOT EXISTS idx_balance_snapshots_account ON balance_snapshots(account_id, as_of);

-- Every last-4-digits card number ever seen for a credit card account,
-- with when it was first observed. A card's number changes on reissue
-- (lost/stolen, expiry) with nothing in a statement export tying the old
-- and new numbers together automatically — see
-- doc/kb/barclaycard/pdf-export-structure.md — so this is a manually
-- confirmed history, not an inferred one: a fresh last4 with no existing
-- match becomes a brand new account until a human links it to an
-- existing one. Ordering by first_seen DESC gives the "current" number.
-- A last4 identifies at most one account at a time (UNIQUE on last4 alone,
-- not per-account) so that Db::link_card_number can reassign a last4 away
-- from a wrongly-auto-created account onto the correct one when a human
-- confirms a reissue.
CREATE TABLE IF NOT EXISTS account_card_numbers (
    id         INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    last4      TEXT NOT NULL UNIQUE,
    first_seen TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_account_card_numbers_account ON account_card_numbers(account_id);

-- Derived spend ledger — see doc/implementation-notes/spend-ledger-design.md.
-- Raw transactions stay immutable evidence; this is the categorised,
-- human-facing view of real-world spending, rebuilt by the derivation pass.
-- Internal transfers between household accounts (including credit card
-- payments) produce no row here at all — see `transfer_entries` instead.
CREATE TABLE IF NOT EXISTS spend_entries (
    id             INTEGER PRIMARY KEY,
    occurred_on    TEXT NOT NULL,
    -- Signed, same convention as transactions (negative = out); a refund is
    -- a positive entry.
    amount_minor   INTEGER NOT NULL,
    currency       TEXT NOT NULL,
    counterparty   TEXT,
    description    TEXT NOT NULL,
    note           TEXT,
    category_id    INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    -- The original charge this entry refunds, if this entry is itself a
    -- refund — replaces the old `transaction_links(relation='refund')` edge
    -- table (Delta: Transfer Ledger, Task 5): a refund has at most one
    -- original, so a self-referencing column on the refund's own row is
    -- simpler than a separate edge table, same reasoning as
    -- `transfer_entries`' counterpart_* columns. `NULL` for every
    -- non-refund entry, and for a refund whose original charge wasn't found
    -- (best-effort merchant-prefix match, see `Db::find_refund_original`).
    refunds_spend_entry_id INTEGER REFERENCES spend_entries(id) ON DELETE SET NULL,
    -- The transfer this manual spend entry was recorded from (`s` on
    -- `Screen::TransferMonth` — see Delta: Credit Card Transaction Import,
    -- Task 4/6). `NULL` for every entry not created that way (i.e. every
    -- normal derived spend entry). Lets the Transfers drill-down show
    -- whether a spend has already been recorded against a given transfer,
    -- and resolve back to it for review/amendment.
    transfer_entry_id INTEGER REFERENCES transfer_entries(id) ON DELETE SET NULL,
    classified_by  TEXT NOT NULL CHECK (classified_by IN
                       ('rule', 'matcher', 'manual')),
    confidence     REAL,
    rule_name      TEXT,
    classified_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_spend_entries_occurred_on ON spend_entries(occurred_on);
CREATE INDEX IF NOT EXISTS idx_spend_entries_category ON spend_entries(category_id);

-- Which raw transaction(s) an entry derives from.
CREATE TABLE IF NOT EXISTS spend_entry_sources (
    spend_entry_id INTEGER NOT NULL REFERENCES spend_entries(id) ON DELETE CASCADE,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    role           TEXT NOT NULL CHECK (role IN ('source', 'annotation')),
    UNIQUE (spend_entry_id, transaction_id)
);

CREATE INDEX IF NOT EXISTS idx_spend_entry_sources_transaction ON spend_entry_sources(transaction_id);

-- Derived transfer ledger — one transfer entry (row) per real-world
-- transfer, linking the two transactions that are its legs directly.
-- Mirrors spend_entries' provenance idiom, but unlike a spend entry (which
-- can have many transaction sources), a transfer entry has at most two:
-- the leg where money left an account (out_*) and the leg where it
-- arrived (in_*). Either side is NULL until its transaction is known —
-- classify() only ever sees one raw transaction at a time, so a transfer
-- entry is always created one-sided and then completed (by an UPDATE) once
-- the other leg is found, possibly much later (cross-file import timing)
-- or never (a Reference Household Account has no transactions.id to ever
-- point at). See doc/implementation-notes/transfer-ledger-design.md.
CREATE TABLE IF NOT EXISTS transfer_entries (
    id                  INTEGER PRIMARY KEY,
    occurred_on         TEXT NOT NULL,
    -- Unsigned magnitude — direction is structural (out_* vs in_*), not a
    -- sign convention, unlike transactions.amount_minor.
    amount_minor        INTEGER NOT NULL CHECK (amount_minor >= 0),
    currency            TEXT NOT NULL,

    -- The leg where money left an account. UNIQUE so re-running derivation
    -- over an already-recorded leg is a no-op. out_account_id/
    -- out_sort_code/out_account_number are set as soon as *either* leg is
    -- known: definitively (the real account, via out_transaction_id) once
    -- that leg's own transaction is found; as a prediction (this leg's own
    -- NAME decode of who *should* receive it) before then — the prediction
    -- can be wrong (see pair_method = 'self_reference_match'), which is
    -- exactly the signal used to tell tiers apart once the real
    -- transaction is found. out_sort_code/out_account_number (raw decoded
    -- digits) stay populated even after confirmation, so a counterpart
    -- that's a Reference Household Account (no accounts.id — ADR 0008) or
    -- an unregistered named individual can still be displayed without a
    -- transaction ever landing on this side.
    out_transaction_id  INTEGER UNIQUE REFERENCES transactions(id) ON DELETE CASCADE,
    out_account_id      INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    out_sort_code       TEXT,
    out_account_number  TEXT,
    out_description     TEXT,

    -- The leg where money arrived. Same shape/rules as out_*, mirrored.
    in_transaction_id   INTEGER UNIQUE REFERENCES transactions(id) ON DELETE CASCADE,
    in_account_id       INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    in_sort_code        TEXT,
    in_account_number   TEXT,
    in_description      TEXT,

    -- How the *pairing* (finding the second leg) was made — NULL until
    -- both legs are known. See the design doc's "Pairing algorithm".
    pair_method         TEXT CHECK (pair_method IN (
                            'description_match',    -- one leg's own NAME
                                                     -- cross-references the
                                                     -- other's account
                                                     -- (manual transfers)
                            'amount_date_match',     -- both legs' own NAME
                                                     -- decodes correctly
                                                     -- identify each other
                                                     -- (automated transfers,
                                                     -- mutual agreement)
                            'self_reference_match',  -- one leg's own NAME
                                                     -- decodes to *itself*
                                                     -- rather than the true
                                                     -- sender; the other
                                                     -- leg's correct decode
                                                     -- plus amount+date is
                                                     -- the whole signal
                                                     -- (e.g. the real SHARED
                                                     -- BILLS ACCO standing
                                                     -- order)
                            'credit_card_payment_match' -- a credit card
                                                     -- payment: bank-side
                                                     -- debit paired with the
                                                     -- card account's
                                                     -- payment-received line
                                                     -- by date + exact
                                                     -- amount, no NAME decode
                                                     -- involved
                         )),
    pair_confidence     REAL,

    -- Free-text annotation, same idiom as spend_entries.note/
    -- income_entries.note — e.g. recording why a transfer was made.
    note                TEXT,

    -- Classification provenance for "this is certainly an internal
    -- transfer" — from whichever leg's classify() result created this row.
    -- Deterministic (household-registry match), so confidence is
    -- high/near-1.0 for every rule that reaches InternalTransfer today;
    -- kept as a real column so manual correction has a place to record
    -- classified_by = 'manual', per the same manual-always-wins convention
    -- as spend_entries.
    classified_by       TEXT NOT NULL CHECK (classified_by IN
                            ('rule', 'matcher', 'manual')),
    confidence           REAL,
    rule_name            TEXT,
    classified_at        TEXT NOT NULL,

    CHECK (out_transaction_id IS NOT NULL OR in_transaction_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_transfer_entries_occurred_on
    ON transfer_entries(occurred_on);
CREATE INDEX IF NOT EXISTS idx_transfer_entries_out_account
    ON transfer_entries(out_account_id);
CREATE INDEX IF NOT EXISTS idx_transfer_entries_in_account
    ON transfer_entries(in_account_id);

-- Derived income ledger — see doc/planning/plan.md, Delta: The Gap, Task 1.
-- Deliberately thin per ADR 0005/0009: no categorisation, no taxonomy, just
-- enough to sum income for a period. Same provenance shape as
-- `spend_entries`, minus the columns that ledger needed but this one
-- doesn't yet (`category_id`, `refunds_spend_entry_id`) — add them if/when
-- income categorisation is designed, rather than pre-guessing the shape.
CREATE TABLE IF NOT EXISTS income_entries (
    id             INTEGER PRIMARY KEY,
    occurred_on    TEXT NOT NULL,
    -- Signed, same convention as transactions (positive = in).
    amount_minor   INTEGER NOT NULL,
    currency       TEXT NOT NULL,
    counterparty   TEXT,
    description    TEXT NOT NULL,
    note           TEXT,
    classified_by  TEXT NOT NULL CHECK (classified_by IN
                       ('rule', 'matcher', 'manual')),
    confidence     REAL,
    rule_name      TEXT,
    classified_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_income_entries_occurred_on ON income_entries(occurred_on);

-- Which raw transaction an income entry derives from. No `role` column
-- (unlike `spend_entry_sources`) — income has no annotation concept yet,
-- so every row is implicitly the source; add one if that changes.
CREATE TABLE IF NOT EXISTS income_entry_sources (
    income_entry_id INTEGER NOT NULL REFERENCES income_entries(id) ON DELETE CASCADE,
    transaction_id   INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    UNIQUE (income_entry_id, transaction_id)
);

CREATE INDEX IF NOT EXISTS idx_income_entry_sources_transaction ON income_entry_sources(transaction_id);
