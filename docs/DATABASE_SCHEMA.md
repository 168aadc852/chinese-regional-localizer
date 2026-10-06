# Draft Database Schema

Status: conceptual draft — not yet implementation schema.

## Design goals

- Support CN / HK / TW forms without assuming every record has all three.
- Separate entities (people, films, organisations) from generic terms where useful.
- Preserve provenance and licensing metadata.
- Support aliases and multiple source claims.
- Allow user overrides without modifying canonical source data.
- Allow conflicting claims to coexist until resolved.

## Conceptual entities

### `sources`

Suggested fields:

- `source_id`
- `name`
- `canonical_url`
- `license_id`
- `status`
- `last_reviewed_at`

### `source_versions`

- `source_version_id`
- `source_id`
- `version_label`
- `revision_id`
- `published_at`
- `retrieved_at`

### `concepts`

Represents a generic term or real-world entity.

- `concept_id`
- `concept_type` (general_term, person, film, tv, organisation, place, product, legal_term, technical_term, etc.)
- `external_id_type`
- `external_id_value`

### `localized_names`

- `localized_name_id`
- `concept_id`
- `locale` (`zh-CN`, `zh-HK`, `zh-TW`, etc.)
- `text`
- `name_type` (preferred, alias, historical, official, colloquial, etc.)
- `source_id`
- `source_version_id`
- `confidence`

### `term_rules`

For deterministic terminology rules not tied to a single real-world entity.

- `rule_id`
- `source_locale`
- `target_locale`
- `source_text`
- `target_text`
- `domain`
- `priority`
- `context_constraint`
- `source_id`
- `source_version_id`

### `protected_terms`

Project-maintained or user-local rules preventing unwanted conversion.

### `user_overrides`

Stored locally and kept separate from canonical upstream-derived data.

## Provenance rule

A localized name or rule imported from an external dataset should be traceable to its source whenever practical.

## Licensing rule

Do not flatten data from differently licensed sources into a form that makes provenance or licence obligations impossible to recover.
