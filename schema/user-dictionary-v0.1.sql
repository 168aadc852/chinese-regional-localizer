PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS user_terms (
    user_term_id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('protected','override')),
    source_text TEXT NOT NULL CHECK (length(source_text) > 0),
    replacement TEXT,
    source_locale TEXT,
    target_locale TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0,1)),
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (
        (kind = 'protected' AND replacement IS NULL)
        OR (kind = 'override' AND replacement IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_user_terms_lookup
ON user_terms(enabled, source_locale, target_locale, source_text, kind, priority DESC);

CREATE UNIQUE INDEX IF NOT EXISTS uq_user_terms_scope
ON user_terms(
    kind,
    source_text,
    ifnull(source_locale, ''),
    ifnull(target_locale, '')
);

CREATE TABLE IF NOT EXISTS user_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR REPLACE INTO user_metadata (key, value)
VALUES ('schema_version', '0.1');
