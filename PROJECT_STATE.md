# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research remains open for 3 unresolved sources.**

**Phase 0.5 proof of concept is complete and CI-validated.**

**Phase 1A first real source importer is implemented and CI-validated.**

## Objective

Build a legally and technically traceable offline CN / HK / TW localization data foundation before the application UI and production localization engine.

## Completed

- GitHub established as the single source of truth for humans and AI tools.
- Source registry expanded to 23 records.
- First-pass usable conclusions completed for 20 of 23 source records.
- Machine-readable ingestion policy added at `data-registry/sources.yaml`.
- Licence packaging classes defined: core, attribution, ShareAlike, reference-only and pending.
- SQLite schema v0.1 implemented with source/version provenance.
- Manifest-enforced Phase 0.5 PoC implemented and CI-validated.
- Issue #2 closed as completed.
- First real importer selected: **LSHK Jyutping Table** under CC BY 4.0.
- LSHK upstream baseline pinned to commit `dad2dd6d6f02fc51138ecc6818f7b38eba5c2ad3`, `list.tsv` Git blob `522f41701dd10c4da08d82923ef7ae40d14b9ffb`.
- Added `schema/migrations/0002_pronunciations.sql` supporting multiple pronunciations per character.
- Added `scripts/import_lshk_jyutping.py` with:
  - explicit manifest permission check;
  - local-file mode and explicit commit-pinned download mode;
  - strict eight-column TSV validation;
  - character/Unicode consistency validation;
  - computed SHA-256;
  - source version/revision/URL/checksum provenance;
  - multiple Jyutping readings per character;
  - SQLite integrity checking.
- Added LSHK sample fixture and importer tests, including malformed-header and Unicode-mismatch rejection.
- CI passed the LSHK importer test suite on 2026-10-06.
- Added `docs/PRONUNCIATION_SCHEMA.md` and recorded the pinned baseline in `data-registry/lshk-jyutping-table.md`.

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

1. Close Issue #3 after recording its successful CI validation.
2. Add a second real importer that contributes directly to CN/HK/TW terminology or entity localization.
3. Prefer a small permissive/CC0 source before tackling full Wikidata dumps.
4. Add an evaluation corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.
5. Continue Issue #1 for the 3 remaining licensing ambiguities.
6. After several importers are stable, implement the deterministic lookup/resolution layer.

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
