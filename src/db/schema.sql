-- ledgr's initial schema.
--
-- Storage is plain SQLite. Relationships that don't fit a single table
-- (category hierarchies, transfer pairs, refund links) are modeled as edge
-- tables (`categories.parent_id`, `transaction_links`) rather than reaching
-- for a dedicated graph database, and traversed with recursive CTEs.
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
    external_id      TEXT
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

-- Generic edge table linking two transactions: transfer pairs between
-- accounts, refunds against an original charge, suspected duplicates, etc.
CREATE TABLE IF NOT EXISTS transaction_links (
    id                  INTEGER PRIMARY KEY,
    from_transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    to_transaction_id   INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    relation            TEXT NOT NULL CHECK (relation IN (
                            'transfer', 'refund', 'duplicate_of', 'related'
                         )),
    confidence          REAL,
    UNIQUE (from_transaction_id, to_transaction_id, relation)
);

CREATE INDEX IF NOT EXISTS idx_tx_links_from ON transaction_links(from_transaction_id);
CREATE INDEX IF NOT EXISTS idx_tx_links_to ON transaction_links(to_transaction_id);

-- Derived spend ledger — see doc/implementation-notes/spend-ledger-design.md.
-- Raw transactions stay immutable evidence; this is the categorised,
-- human-facing view of real-world spending, rebuilt by the derivation pass.
-- Internal transfers between household accounts produce no row here at all
-- (see transaction_links, relation='transfer').
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
