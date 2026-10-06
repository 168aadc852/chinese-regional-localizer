# Pronunciation Schema Extension

Status: implemented by `schema/migrations/0002_pronunciations.sql`.

The core v0.1 schema did not have a suitable place for pronunciation data. Phase 1A adds a dedicated `pronunciations` table rather than overloading localized names or terminology rules.

## Why a separate table

A single Chinese character can have multiple Cantonese readings. Each reading needs its own Jyutping form, initial, final, tone and source provenance.

## `pronunciations`

Fields:

- `pronunciation_id`
- `concept_id` → `concepts`
- `locale` — LSHK imports use `yue-Hant-HK`
- `romanization_scheme` — `Jyutping`
- `reading`
- `initial`
- `final`
- `tone`
- `description`
- `description_romanized`
- `source_id`
- `source_version_id`
- `upstream_record_id`
- `upstream_url`
- `confidence`

The uniqueness rule allows several readings for the same character but prevents duplicate insertion of the same reading from the same source version.

## LSHK mapping

`list.tsv` maps as follows:

- `CH` → character localized name / concept identity
- `UCODE` → Unicode external ID and canonical key
- `JP` → `reading`
- `INIT` → `initial`
- `FINL` → `final`
- `TONE` → `tone`
- `DESC` → `description`
- `DESC_JP` → `description_romanized`

Every imported pronunciation points to `lshk-jyutping-table` plus a pinned `source_versions` row containing revision, URL and computed SHA-256.
