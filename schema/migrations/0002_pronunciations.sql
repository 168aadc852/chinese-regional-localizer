PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS pronunciations (
    pronunciation_id INTEGER PRIMARY KEY,
    concept_id INTEGER NOT NULL REFERENCES concepts(concept_id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    romanization_scheme TEXT NOT NULL,
    reading TEXT NOT NULL,
    initial TEXT,
    final TEXT,
    tone TEXT,
    description TEXT,
    description_romanized TEXT,
    source_id TEXT NOT NULL REFERENCES sources(source_id),
    source_version_id INTEGER REFERENCES source_versions(source_version_id),
    upstream_record_id TEXT,
    upstream_url TEXT,
    confidence REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    UNIQUE(concept_id, locale, romanization_scheme, reading, source_id, source_version_id)
);

CREATE INDEX IF NOT EXISTS idx_pronunciations_concept
ON pronunciations(concept_id);

CREATE INDEX IF NOT EXISTS idx_pronunciations_reading
ON pronunciations(romanization_scheme, reading);

CREATE INDEX IF NOT EXISTS idx_pronunciations_source
ON pronunciations(source_id, source_version_id);
