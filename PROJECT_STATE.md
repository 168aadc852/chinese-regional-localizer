# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research remains open for 3 unresolved sources.**

**Phase 0.5 proof of concept is complete and CI-validated.**

**Phase 1A LSHK pronunciation importer is complete and CI-validated.**

**Phase 1B OpenCC regional phrase importer is complete and CI-validated.**

**Phase 1C Wikidata entity/localized-name importer is complete and CI-validated.**

The project now has working ingestion paths for pronunciation data, deterministic script/regional phrase rules, and stable real-world entity names. The next engineering step is a deterministic lookup/resolution engine that composes those layers and returns explainable results.

## Objective

Build a legally and technically traceable offline CN / HK / TW localization data foundation and deterministic engine before application UI work.

## Completed

- GitHub established as the single source of truth for humans and AI tools.
- Source registry expanded to 23 records.
- First-pass usable conclusions completed for 20 of 23 source records.
- Machine-readable ingestion policy added at `data-registry/sources.yaml`.
- Licence packaging classes defined: core, attribution, ShareAlike, reference-only and pending.
- SQLite schema v0.1 implemented with source/version provenance.
- Manifest-enforced Phase 0.5 PoC implemented and CI-validated; Issue #2 closed.

### Phase 1A — LSHK Jyutping Table

- Pinned upstream commit `dad2dd6d6f02fc51138ecc6818f7b38eba5c2ad3`.
- Added `schema/migrations/0002_pronunciations.sql` supporting multiple pronunciations per character.
- Added `scripts/import_lshk_jyutping.py` with manifest enforcement, local/pinned-download modes, strict TSV validation, Unicode checks, SHA-256 and full source-version provenance.
- Added fixtures/tests and `docs/PRONUNCIATION_SCHEMA.md`.
- CI passed and Issue #3 closed.

### Phase 1B — OpenCC regional phrase dictionaries

- Pinned upstream OpenCC commit `3ac34aa439a9908dd49fa92b5174b46314787ac2`.
- Added `scripts/import_opencc_dictionaries.py`.
- Initial real dictionary scope:
  - `STPhrases.txt` — `zh-CN -> zh-Hant` script stage;
  - `HKPhrases.txt` — `zh-Hant -> zh-HK` regional stage;
  - `TWPhrases.txt` — `zh-Hant -> zh-TW` regional stage.
- Preserved OpenCC's staged model rather than flattening regional conversion into one global replacement table.
- Preserved multiple target candidates in source order using rule priority plus `candidate_rank`/`candidate_count` metadata.
- Added separate `source_versions` rows per dictionary with pinned revision, upstream URL and SHA-256.
- Added strict header/line validation, identity-mapping support, fixtures/tests and `docs/OPENCC_IMPORTER.md`.
- CI passed and Issue #4 closed.

### Phase 1C — Wikidata entity/localized names

- Added `scripts/import_wikidata_entities.py` using Wikidata structured EntityData JSON within the approved CC0 scope.
- Added local offline fixtures for a person and a film plus explicit QID-to-concept-type mapping.
- Added locale normalization for `zh-CN`, `zh-HK`, `zh-TW`, `zh-Hans`, `zh-Hant`, `zh`, `en` and `mul`.
- Enforced a hard no-fabrication rule: a missing regional label is not synthesized from generic `zh`/`zh-Hans`/`zh-Hant`.
- Stored stable Wikidata QIDs in `external_ids`.
- Stored labels as preferred names and aliases as aliases without overwriting claims from other sources.
- Preserved per-entity `lastrevid`, modified/retrieval timestamps, revision-aware upstream URL, canonical JSON SHA-256 and per-name evidence.
- Added malformed-QID/entity validation and explicit type-map validation.
- Added `tests/test_wikidata_importer.py` and `docs/WIKIDATA_ENTITY_IMPORTER.md`.
- Recorded the importer baseline in `data-registry/wikidata.md`.
- GitHub Actions completed successfully after the final provenance/retrieval-time update on 2026-10-06.

## Approved / approved-with-conditions sources

### Core/permissive or attribution-capable
- OpenCC
- Wikidata
- Taiwan Government / verified NAER terminology datasets
- DATA.GOV.HK qualifying portal data
- Words.hk public-domain word-list/pronunciation subset
- Rime Cantonese (file-level split licensing)
- Unicode / Unihan / CLDR machine-readable Data Files
- MusicBrainz Core Data
- THUOCL published word-list/frequency package
- PanLex
- Chinese Open WordNet
- HKCanCor
- LSHK Jyutping Table
- individually reviewed Kaifangcidian datasets such as `kfcd/hyzd`

### ShareAlike-isolated sources
- Chinese Wikipedia
- Chinese Wiktionary
- CC-CEDICT
- CC-Canto
- ConceptNet
- CFDICT
- Rime `jyut6ping3.maps` under ODbL

## Pending rights/version clarification

1. Combined DoJ Glossaries of Legal Terms — adaptation rights for a redistributable derived terminology database remain unclear.
2. DBnary — snapshot-level CC BY-SA version must be pinned because reviewed surfaces show version inconsistency.
3. OpenHowNet downloadable core data — repository/API MIT licence is clear, but data-specific redistribution rights remain unconfirmed.

## Next recommended work

1. Build Phase 2A deterministic lookup/resolution engine.
2. Match known entities before generic terminology rules when an exact/longest entity name is found.
3. Compose staged script/regional rules for CN -> HK/TW rather than flattening OpenCC stages.
4. Return structured explanation/provenance for every applied change.
5. Build a compact evaluation corpus covering people, films, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.
6. Continue Issue #1 for the 3 remaining licensing ambiguities.
7. After the deterministic engine is stable, expose it through a small local API/CLI before desktop/mobile UI work.

## Not started

- Production desktop application
- Mobile application
- Database auto-updater/release pipeline

## Current technical direction

- Cross-platform app: Tauri 2
- Core runtime: Rust
- Local database: SQLite
- Data-build/import tooling: Python is acceptable where practical
- Deterministic conversion: staged OpenCC-compatible rules plus entity/localized-name resolution
- Entity/terminology layer: multi-source knowledge base
- Optional future ambiguity layer: local LLM only if deterministic/context rules are insufficient

## Important constraint

Importers must enforce `data-registry/sources.yaml`, including `ingest_allowed`, `ingest_scope`, `excluded_scope` and pack separation. Pending/reference-only/rejected sources must not enter redistributable data builds. Missing regional entity names must not be silently invented.
