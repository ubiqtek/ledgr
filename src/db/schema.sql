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
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    institution  TEXT,
    account_type TEXT NOT NULL CHECK (account_type IN (
                     'checking', 'savings', 'credit_card', 'pension',
                     'investment', 'other'
                 )),
    currency     TEXT NOT NULL DEFAULT 'GBP',
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Self-referencing edge for category hierarchy (e.g. "Groceries" under "Living Costs").
CREATE TABLE IF NOT EXISTS categories (
    id        INTEGER PRIMARY KEY,
    name      TEXT NOT NULL,
    parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);

-- One row per imported file, so re-importing the same statement is a no-op.
CREATE TABLE IF NOT EXISTS statements (
    id           INTEGER PRIMARY KEY,
    account_id   INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    source_path  TEXT NOT NULL,
    file_hash    TEXT NOT NULL UNIQUE,
    imported_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    period_start TEXT,
    period_end   TEXT
);

CREATE TABLE IF NOT EXISTS transactions (
    id               INTEGER PRIMARY KEY,
    account_id       INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    statement_id     INTEGER REFERENCES statements(id) ON DELETE SET NULL,
    posted_at        TEXT NOT NULL,
    amount_minor     INTEGER NOT NULL,
    currency         TEXT NOT NULL,
    description      TEXT NOT NULL,
    raw_description  TEXT,
    category_id      INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    external_id      TEXT
);

CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_posted_at ON transactions(posted_at);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);

-- Balance "anchors" reported by the bank itself (OFX LEDGERBAL and
-- equivalents), one per statement that carries one. The transaction list in
-- a given statement often doesn't reach back to account opening, so a
-- balance can't be reliably derived by summing transactions alone; it must
-- be reconstructed from the nearest anchor plus the transactions between
-- that anchor and the target date.
CREATE TABLE IF NOT EXISTS balance_snapshots (
    id            INTEGER PRIMARY KEY,
    account_id    INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    statement_id  INTEGER REFERENCES statements(id) ON DELETE SET NULL,
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
