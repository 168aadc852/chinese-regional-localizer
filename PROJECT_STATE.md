# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research remains open for 3 unresolved sources.**

**Phase 0.5 proof of concept is complete and CI-validated.**

**Phase 1A LSHK pronunciation importer is complete and CI-validated.**

**Phase 1B OpenCC regional phrase importer is complete and CI-validated.**

## Objective

Build a legally and technically traceable offline CN / HK / TW localization data foundation before the application UI and production localization engine.

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
- Added a separate `source_versions` record per imported dictionary with pinned revision, upstream URL and SHA-256.
- Added strict header/line validation, comments/blank-line handling and identity-mapping support.
- Added real-format fixtures and tests covering:
  - `布拉德·皮特 -> 畢·彼特` (HK);
  - `人工智能 -> 人工智慧` (TW);
  - `一见钟情 -> 一見鍾情` (script stage);
  - multi-candidate preservation;
  - malformed-header/line rejection.
- Added `docs/OPENCC_IMPORTER.md` and recorded the importer baseline in `data-registry/opencc.md`.
- CI passed the OpenCC importer test suite on 2026-10-06.

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

1. Close Issue #4 after recording Phase 1B completion.
2. Build the first real **entity/localized-name importer**, preferably from Wikidata structured data (CC0), using a deliberately small query/snapshot first rather than a full dump.
3. Demonstrate one person and one film/work with `zh-CN`, `zh-HK`, `zh-TW` labels/aliases plus stable Wikidata QIDs and provenance.
4. Build a small evaluation corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.
5. Continue Issue #1 for the 3 remaining licensing ambiguities.
6. After several importers are stable, implement the deterministic lookup/resolution layer that composes script, regional term and entity rules.

## Not started

- Production localization engine
- Desktop application
- Mobile application
- Database auto-updater/release pipeline

## Current technical direction

- Cross-platform app: Tauri 2
- Core runtime: Rust
- Local database: SQLite
- Data-build/import tooling: Python is acceptable where practical
- Deterministic conversion: OpenCC and/or compatible regional conversion engine
- Entity/terminology layer: multi-source knowledge base
- Optional future ambiguity layer: local LLM only if deterministic/context rules are insufficient

## Important constraint

Importers must enforce `data-registry/sources.yaml`, including `ingest_allowed`, `ingest_scope`, `excluded_scope` and pack separation. Pending/reference-only/rejected sources must not enter redistributable data builds.
