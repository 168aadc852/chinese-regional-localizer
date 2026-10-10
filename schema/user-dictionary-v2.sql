PRAGMA foreign_keys = ON;
CREATE TABLE user_dictionaries (
    dictionary_id TEXT PRIMARY KEY CHECK(length(dictionary_id) > 0),
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)),
    system_managed INTEGER NOT NULL DEFAULT 0 CHECK(system_managed IN (0,1)),
    note TEXT
);
INSERT INTO user_dictionaries VALUES ('personal','Personal',1,1,NULL);
INSERT INTO user_dictionaries VALUES ('legacy','Legacy / Default',1,1,NULL);
CREATE TABLE user_terms (
    user_term_id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('protected','override')),
    source_text TEXT NOT NULL CHECK(length(source_text) > 0),
    replacement TEXT,
    source_locale TEXT,
    target_locale TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)),
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    dictionary_id TEXT NOT NULL DEFAULT 'legacy' REFERENCES user_dictionaries(dictionary_id),
    usage_context_id TEXT,
    legacy INTEGER NOT NULL DEFAULT 1 CHECK(legacy IN (0,1)),
    CHECK((kind='protected' AND replacement IS NULL) OR (kind='override' AND replacement IS NOT NULL))
);
CREATE UNIQUE INDEX uq_user_terms_scope ON user_terms(
    dictionary_id,kind,source_text,ifnull(source_locale,''),ifnull(target_locale,''),ifnull(usage_context_id,'')
);
CREATE INDEX idx_user_terms_lookup ON user_terms(enabled,source_locale,target_locale,source_text,usage_context_id);
CREATE TABLE user_metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
INSERT INTO user_metadata VALUES ('schema_version','2');
CREATE TRIGGER validate_new_preference_insert BEFORE INSERT ON user_terms
WHEN NEW.legacy=0 AND (NEW.target_locale IS NULL OR NEW.target_locale NOT IN ('zh-CN','zh-Hant','zh-HK','zh-TW'))
BEGIN SELECT RAISE(ABORT,'New preferences require a target locale'); END;
CREATE TRIGGER validate_new_preference_update BEFORE UPDATE ON user_terms
WHEN NEW.legacy=0 AND (NEW.target_locale IS NULL OR NEW.target_locale NOT IN ('zh-CN','zh-Hant','zh-HK','zh-TW'))
BEGIN SELECT RAISE(ABORT,'New preferences require a target locale'); END;
