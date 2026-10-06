PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sources (
    source_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('approved','approved_with_conditions','pending_review','reference_only','rejected')),
    pack TEXT NOT NULL CHECK (pack IN ('core','attribution','sharealike','reference_only','pending')),
    licence TEXT,
    commercial_use INTEGER CHECK (commercial_use IN (0,1) OR commercial_use IS NULL),
    modification_allowed INTEGER CHECK (modification_allowed IN (0,1) OR modification_allowed IS NULL),
    redistribution_allowed INTEGER CHECK (redistribution_allowed IN (0,1) OR redistribution_allowed IS NULL),
    attribution_required INTEGER CHECK (attribution_required IN (0,1) OR attribution_required IS NULL),
    share_alike INTEGER CHECK (share_alike IN (0,1) OR share_alike IS NULL),
    ingest_allowed INTEGER NOT NULL CHECK (ingest_allowed IN (0,1)),
    ingest_scope TEXT,
    excluded_scope TEXT,
    review_record TEXT NOT NULL,
    manifest_version INTEGER NOT NULL,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS source_versions (
    source_version_id INTEGER PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES sources(source_id),
    resource_key TEXT NOT NULL DEFAULT '',
    is_current INTEGER NOT NULL DEFAULT 1 CHECK (is_current IN (0,1)),
    version_label TEXT,
    revision_id TEXT,
    published_at TEXT,
    retrieved_at TEXT NOT NULL,
    upstream_url TEXT,
    checksum_sha256 TEXT,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_source_versions_source
ON source_versions(source_id);

CREATE INDEX IF NOT EXISTS idx_source_versions_current_resource
ON source_versions(source_id, resource_key, is_current, source_version_id);

CREATE TABLE IF NOT EXISTS concepts (
    concept_id INTEGER PRIMARY KEY,
    concept_type TEXT NOT NULL,
    canonical_key TEXT,
    domain TEXT,
    created_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_concepts_type
ON concepts(concept_type);

CREATE INDEX IF NOT EXISTS idx_concepts_canonical_key
ON concepts(canonical_key);

CREATE TABLE IF NOT EXISTS external_ids (
    external_id_id INTEGER PRIMARY KEY,
    concept_id INTEGER NOT NULL REFERENCES concepts(concept_id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    external_value TEXT NOT NULL,
    source_id TEXT REFERENCES sources(source_id),
    UNIQUE(namespace, external_value)
);

CREATE INDEX IF NOT EXISTS idx_external_ids_concept
ON external_ids(concept_id);

CREATE TABLE IF NOT EXISTS localized_names (
    localized_name_id INTEGER PRIMARY KEY,
    concept_id INTEGER NOT NULL REFERENCES concepts(concept_id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    text TEXT NOT NULL,
    name_type TEXT NOT NULL,
    domain TEXT,
    is_preferred INTEGER NOT NULL DEFAULT 0 CHECK (is_preferred IN (0,1)),
    confidence REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    UNIQUE(concept_id, locale, text, name_type)
);

CREATE INDEX IF NOT EXISTS idx_localized_names_lookup
ON localized_names(locale, text);

CREATE INDEX IF NOT EXISTS idx_localized_names_domain_lookup
ON localized_names(locale, domain, text);

CREATE INDEX IF NOT EXISTS idx_localized_names_concept_locale
ON localized_names(concept_id, locale);

CREATE TABLE IF NOT EXISTS name_evidence (
    evidence_id INTEGER PRIMARY KEY,
    localized_name_id INTEGER NOT NULL REFERENCES localized_names(localized_name_id) ON DELETE CASCADE,
    source_id TEXT NOT NULL REFERENCES sources(source_id),
    source_version_id INTEGER REFERENCES source_versions(source_version_id),
    upstream_record_id TEXT,
    upstream_url TEXT,
    upstream_revision TEXT,
    evidence_type TEXT,
    confidence REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    transformation_note TEXT,
    retrieved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_name_evidence_name
ON name_evidence(localized_name_id);

CREATE INDEX IF NOT EXISTS idx_name_evidence_source
ON name_evidence(source_id, source_version_id);

CREATE TABLE IF NOT EXISTS term_rules (
    rule_id INTEGER PRIMARY KEY,
    source_locale TEXT,
    target_locale TEXT NOT NULL,
    source_text TEXT NOT NULL,
    target_text TEXT NOT NULL,
    domain TEXT,
    rule_type TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    context_constraint TEXT,
    source_id TEXT NOT NULL REFERENCES sources(source_id),
    source_version_id INTEGER REFERENCES source_versions(source_version_id),
    upstream_record_id TEXT,
    upstream_url TEXT,
    confidence REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0,1))
);

CREATE INDEX IF NOT EXISTS idx_term_rules_lookup
ON term_rules(source_locale, target_locale, active, source_text, priority DESC);

CREATE INDEX IF NOT EXISTS idx_term_rules_source
ON term_rules(source_id, source_version_id);

CREATE TABLE IF NOT EXISTS build_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE VIEW IF NOT EXISTS current_name_evidence AS
SELECT ne.*
FROM name_evidence ne
LEFT JOIN source_versions sv ON sv.source_version_id = ne.source_version_id
WHERE ne.source_version_id IS NULL OR sv.is_current = 1;
