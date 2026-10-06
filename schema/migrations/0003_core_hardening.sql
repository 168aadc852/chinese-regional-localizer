PRAGMA foreign_keys = ON;

ALTER TABLE source_versions ADD COLUMN resource_key TEXT NOT NULL DEFAULT '';
ALTER TABLE source_versions ADD COLUMN is_current INTEGER NOT NULL DEFAULT 1 CHECK (is_current IN (0,1));

CREATE INDEX IF NOT EXISTS idx_source_versions_current_resource
ON source_versions(source_id, resource_key, is_current, source_version_id);

CREATE INDEX IF NOT EXISTS idx_localized_names_domain_lookup
ON localized_names(locale, domain, text);

DROP INDEX IF EXISTS idx_term_rules_lookup;
CREATE INDEX IF NOT EXISTS idx_term_rules_lookup
ON term_rules(source_locale, target_locale, active, source_text, priority DESC);

CREATE VIEW IF NOT EXISTS current_name_evidence AS
SELECT ne.*
FROM name_evidence ne
LEFT JOIN source_versions sv ON sv.source_version_id = ne.source_version_id
WHERE ne.source_version_id IS NULL OR sv.is_current = 1;

CREATE VIEW IF NOT EXISTS current_pronunciations AS
SELECT p.*
FROM pronunciations p
LEFT JOIN source_versions sv ON sv.source_version_id = p.source_version_id
WHERE p.source_version_id IS NULL OR sv.is_current = 1;

INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('schema_version', '0.2');
