# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research remains open for 3 unresolved sources.**

**Phase 0.5 proof of concept is complete and CI-validated.**

**Phase 1A LSHK pronunciation importer is complete and CI-validated.**

**Phase 1B OpenCC regional phrase importer is complete and CI-validated.**

**Phase 1C Wikidata entity/localized-name importer is complete and CI-validated.**

**Phase 2A deterministic localization/resolution engine is complete and CI-validated.**

The project can now build a small offline SQLite database, recognize regional entity names, compose staged terminology rules, localize text deterministically and return source-aware explanations/review flags.

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
- Preserved OpenCC's staged model and multi-candidate order.
- Added separate source versions, checksums, strict validation, fixtures/tests and `docs/OPENCC_IMPORTER.md`.
- CI passed and Issue #4 closed.

### Phase 1C — Wikidata entity/localized names

- Added `scripts/import_wikidata_entities.py` using Wikidata structured EntityData JSON within the approved CC0 scope.
- Added stable QID identity, explicit concept types, labels/aliases, regional locale separation and full source/revision/retrieval/checksum provenance.
- Enforced no-fabrication of missing regional labels.
- Added offline fixtures/tests and `docs/WIKIDATA_ENTITY_IMPORTER.md`.
- CI passed and Issue #5 closed.

### Phase 2A — Deterministic localization/resolution engine

- Added Python behavior/reference engine at `src/localizer_engine.py`.
- Added CLI at `scripts/localize_text.py`.
- Added one-command fixture database builder at `scripts/build_demo_database.py`.
- Added evaluation corpus at `data/fixtures/evaluation_cases.json`.
- Entity handling:
  - source-locale names/aliases are matched longest-first;
  - unique entity + unique preferred target-regional name is localized directly;
  - localized entity spans are protected from later generic rules;
  - ambiguous entities or missing/conflicting target names are preserved and marked `review_needed`.
- Term-rule handling:
  - explicit route stages are composed instead of flattened;
  - deterministic longest phrase wins before priority;
  - highest-priority target candidate wins for the same source phrase;
  - lower candidates remain exposed as alternatives;
  - equal-priority conflicting targets are preserved and marked for review;
  - repeated global string replacement is not used.
- Explainability output includes entity/QID or rule provenance, revisions, URLs, checksums and confidence where available.
- Added integration tests in `tests/test_localizer_engine.py` covering entity localization, staged terminology, entity protection, longest-match behavior, ambiguous entity/rule handling and provenance.
- GitHub Actions passed all integrated tests and demo-build steps on 2026-10-06.
- Added `docs/LOCALIZER_ENGINE.md`.
- Recorded D-009: deterministic localization precedence and no-guess policy.

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

1. Expand OpenCC ingestion beyond phrase dictionaries to the character/variant resources required for broad real-world text conversion.
2. Add user/protected-term precedence and a separate user dictionary model after core conversion coverage is broader.
3. Expand the evaluation corpus with ordinary sentences, punctuation, overlapping names, IT, transport, legal and ambiguous terms.
4. Continue Issue #1 for the 3 remaining licensing ambiguities.
5. After deterministic behavior and coverage stabilize, port the tested engine semantics to the planned Rust runtime.
6. Only then expose the stable core to Tauri desktop/mobile UI.

## Not started

- Production Rust localization engine
- Desktop application
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

Importers must enforce `data-registry/sources.yaml`, including `ingest_allowed`, `ingest_scope`, `excluded_scope` and pack separation. Pending/reference-only/rejected sources must not enter redistributable data builds. Missing regional entity names and ambiguous decisions must not be silently invented.
