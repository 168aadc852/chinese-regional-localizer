# Database Schema v0.1

Status: **implementation draft for the first SQLite proof of concept**.

This schema is intentionally source/provenance-first. The project must be able to explain where a regional term or proper name came from and keep differently licensed data separable.

## Design goals

- Support `zh-CN`, `zh-HK`, `zh-TW` without requiring all three forms on every record.
- Separate real-world entities from generic terminology rules.
- Preserve source, source version, upstream record/revision and transformation provenance.
- Allow multiple sources to support the same localized name.
- Allow conflicting claims to coexist until ranking/review resolves them.
- Keep ShareAlike/attribution/core packs separable.
- Keep user overrides separate from canonical upstream data.
- Support deterministic offline lookup before any optional LLM layer.

## 1. `sources`

One row per machine-readable entry in `data-registry/sources.yaml`.

Fields:

- `source_id TEXT PRIMARY KEY`
- `name TEXT NOT NULL`
- `status TEXT NOT NULL`
- `pack TEXT NOT NULL`
- `licence TEXT`
- `commercial_use INTEGER` — `1`, `0`, or `NULL`
- `modification_allowed INTEGER`
- `redistribution_allowed INTEGER`
- `attribution_required INTEGER`
- `share_alike INTEGER`
- `ingest_allowed INTEGER NOT NULL`
- `ingest_scope TEXT`
- `excluded_scope TEXT`
- `review_record TEXT NOT NULL`
- `manifest_version INTEGER NOT NULL`
- `updated_at TEXT`

Hard rule: no importer may insert source-derived rows when the matching manifest entry has `ingest_allowed != 1`.

## 2. `source_versions`

Represents the exact upstream snapshot/dump/revision used in a build.

Fields:

- `source_version_id INTEGER PRIMARY KEY`
- `source_id TEXT NOT NULL REFERENCES sources(source_id)`
- `version_label TEXT`
- `revision_id TEXT`
- `published_at TEXT`
- `retrieved_at TEXT NOT NULL`
- `upstream_url TEXT`
- `checksum_sha256 TEXT`
- `notes TEXT`

A database build should never say only “from Wikipedia” or “from Wikidata”; it should identify the source version/snapshot whenever practical.

## 3. `concepts`

Represents a generic lexical concept or a real-world entity.

Fields:

- `concept_id INTEGER PRIMARY KEY`
- `concept_type TEXT NOT NULL`
- `canonical_key TEXT`
- `domain TEXT`
- `created_at TEXT`

Suggested `concept_type` values:

- `general_term`
- `person`
- `film`
- `tv`
- `book`
- `music_artist`
- `music_work`
- `organisation`
- `place`
- `product`
- `legal_term`
- `technical_term`
- `game`
- `sports_entity`
- `other`

`canonical_key` is an optional project-stable identity key, not a display name.

## 4. `external_ids`

Links concepts to stable external identifiers.

Fields:

- `external_id_id INTEGER PRIMARY KEY`
- `concept_id INTEGER NOT NULL REFERENCES concepts(concept_id)`
- `namespace TEXT NOT NULL`
- `external_value TEXT NOT NULL`
- `source_id TEXT REFERENCES sources(source_id)`

Recommended unique constraint:

`UNIQUE(namespace, external_value)`

Examples of namespaces:

- `wikidata`
- `musicbrainz_artist`
- `musicbrainz_work`
- `isbn`
- other stable open identifiers added later

## 5. `localized_names`

Stores unique localized/display names independently from evidence, allowing several sources to support the same name.

Fields:

- `localized_name_id INTEGER PRIMARY KEY`
- `concept_id INTEGER NOT NULL REFERENCES concepts(concept_id)`
- `locale TEXT NOT NULL`
- `text TEXT NOT NULL`
- `name_type TEXT NOT NULL`
- `domain TEXT`
- `is_preferred INTEGER NOT NULL DEFAULT 0`
- `confidence REAL`

Recommended unique constraint:

`UNIQUE(concept_id, locale, text, name_type)`

Suggested `name_type` values:

- `preferred`
- `official`
- `alias`
- `historical`
- `colloquial`
- `transliteration`
- `original_title`

Locales should use BCP-47-like identifiers, with first-class support for:

- `zh-CN`
- `zh-HK`
- `zh-TW`
- `zh-Hans`
- `zh-Hant`
- `en`

Additional locales may be stored when useful for entity matching.

## 6. `name_evidence`

Preserves provenance for each localized-name claim.

Fields:

- `evidence_id INTEGER PRIMARY KEY`
- `localized_name_id INTEGER NOT NULL REFERENCES localized_names(localized_name_id)`
- `source_id TEXT NOT NULL REFERENCES sources(source_id)`
- `source_version_id INTEGER REFERENCES source_versions(source_version_id)`
- `upstream_record_id TEXT`
- `upstream_url TEXT`
- `upstream_revision TEXT`
- `evidence_type TEXT`
- `confidence REAL`
- `transformation_note TEXT`
- `retrieved_at TEXT`

Suggested `evidence_type` values:

- `direct_label`
- `alias`
- `redirect`
- `conversion_rule`
- `dictionary_entry`
- `official_term`
- `derived_from_corpus`
- `manual_review`

This table is central to explainable conversion: a UI can show why a name was chosen and where it came from.

## 7. `term_rules`

For deterministic regional terminology conversion not necessarily tied to a unique real-world entity.

Fields:

- `rule_id INTEGER PRIMARY KEY`
- `source_locale TEXT`
- `target_locale TEXT NOT NULL`
- `source_text TEXT NOT NULL`
- `target_text TEXT NOT NULL`
- `domain TEXT`
- `rule_type TEXT NOT NULL`
- `priority INTEGER NOT NULL DEFAULT 0`
- `context_constraint TEXT`
- `source_id TEXT NOT NULL REFERENCES sources(source_id)`
- `source_version_id INTEGER REFERENCES source_versions(source_version_id)`
- `upstream_record_id TEXT`
- `upstream_url TEXT`
- `confidence REAL`
- `active INTEGER NOT NULL DEFAULT 1`

Suggested `rule_type` values:

- `character`
- `lexical`
- `regional_term`
- `proper_name`
- `protected`
- `exception`

Do not use a naive global search-and-replace implementation. Matching should eventually support longest-match, priorities, protected spans and context constraints.

## 8. `build_metadata`

Records how a distributable SQLite pack was built.

Fields:

- `key TEXT PRIMARY KEY`
- `value TEXT NOT NULL`

Recommended keys:

- `schema_version`
- `build_id`
- `built_at`
- `manifest_version`
- `pack_type`
- `generator_version`
- `source_count`

## 9. User-local data must be separate

Do **not** mix personal user overrides into the canonical downloaded database.

Use a separate local database such as `user_dictionary.sqlite` for:

### `protected_terms`
- text/pattern
- locale/domain scope
- created_at

### `user_overrides`
- source text
- preferred target text
- source/target locale
- domain/context
- priority
- created_at / updated_at

This allows the canonical database to be replaced during updates without losing user preferences.

## Pack separation

Recommended future files:

- `regional_core.sqlite`
- `regional_attribution.sqlite`
- `regional_sharealike.sqlite`
- `user_dictionary.sqlite`

The exact final packaging may change after the proof of concept, but a database build must never make it impossible to recover source/licence obligations.

## Importer safety contract

Before importing a source or file, the importer must:

1. load `data-registry/sources.yaml`;
2. locate the exact manifest `id`;
3. require `ingest_allowed: true`;
4. enforce `ingest_scope` and `excluded_scope`;
5. assign the correct `pack`;
6. create/update `sources` and `source_versions` first;
7. attach source/version provenance to every imported rule or evidence row;
8. reject pending/reference-only/rejected sources by default.

## Conflict strategy

Do not overwrite conflicting data simply because a later importer runs.

Examples:

- two sources may disagree on a Hong Kong film title;
- an official terminology source may disagree with community usage;
- a historic alias may still be useful even if no longer preferred.

Store the claims/evidence separately. Preference/ranking logic belongs in a later resolution layer.

## Minimum proof-of-concept target

The first SQLite proof of concept should use only a small set of approved, non-ShareAlike sources and demonstrate:

- one generic CN/HK/TW terminology example;
- one film/entity example;
- one person/entity example;
- one Cantonese/Hong Kong lexical signal;
- provenance display for every result;
- safe refusal to import a `pending_review` source.
