# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research**

A parallel **Phase 0.5 proof-of-concept track** is now implemented and under automated validation using only manifest-approved source classes.

## Objective of this phase

Establish a legally and technically traceable data foundation before writing the main localization application.

## Completed

- Defined the project as an offline-first CN / HK / TW Chinese regional localization tool.
- Agreed that GitHub is the single source of truth for humans and AI tools.
- Created the initial repository documentation and source registry structure.
- Expanded the source registry to 23 records after separating DATA.GOV.HK from the Combined DoJ Glossaries.
- Opened GitHub Issue #1 to manage authoritative licence verification and whitelist approval.
- Completed first-pass formal reviews with usable conclusions for **20 of 23** source records.
- Established packaging classes: permissive/core, attribution-required, ShareAlike-isolated, non-commercial/reference-only and pending-rights layers.
- Added `docs/REVIEW_PROGRESS.md` and `docs/LICENSE_PACKAGING.md`.
- Added machine-readable ingestion policy at `data-registry/sources.yaml`.
- Added `docs/SOURCE_MANIFEST_SCHEMA.md` defining manifest semantics and hard safety rules.
- Updated `AGENTS.md` so all AI/data tooling must enforce the manifest before ingestion.
- Recorded decision D-008: the manifest is the machine-readable ingestion gate.
- Promoted `docs/DATABASE_SCHEMA.md` to implementation draft **v0.1**, including mandatory source/version provenance and pack separation.
- Added executable, syntax-validated SQLite schema at `schema/sqlite-v0.1.sql`.
- Opened GitHub Issue #2: **Phase 0.5 — Build manifest-enforced SQLite proof of concept**.
- Added `scripts/poc_builder.py`, which:
  - validates `sources.yaml`;
  - loads all source metadata into SQLite;
  - rejects data ingestion from blocked/pending sources;
  - enforces source IDs before inserting source versions, term rules and entity names;
  - preserves fixture provenance in `source_versions` and `name_evidence`;
  - runs SQLite `integrity_check` after building.
- Added clearly marked non-authoritative PoC fixtures under `data/fixtures/` covering:
  - CN→HK/TW technology terminology;
  - a film with CN/HK/TW regional titles;
  - a person with CN/HK/TW regional names;
  - a Hong Kong lexical signal.
- Added `tests/test_poc_builder.py` covering policy rejection, database build/integrity, regional lookup and provenance.
- Added GitHub Actions workflow `.github/workflows/poc-tests.yml` to run tests and build the demo database on push/PR.
- Added `docs/POC.md` documenting the PoC safety boundary and local test workflow.

## Approved / approved-with-conditions sources

### Core/permissive or attribution-capable
- OpenCC
- Wikidata
- Taiwan Government / verified NAER open terminology datasets
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

1. Combined DoJ Glossaries of Legal Terms
   - official XML downloads and Open Data label confirmed;
   - explicit reuse/adaptation rights for a derived redistributable terminology database are not yet confirmed.
2. DBnary
   - clearly ShareAlike;
   - reviewed project surfaces show licence-version inconsistency, so snapshot-level licence confirmation is required.
3. OpenHowNet downloadable HowNet core data
   - repository/API code is MIT;
   - data is downloaded separately and a data-specific redistribution licence has not yet been independently confirmed.

## Next recommended work

1. Confirm the Phase 0.5 GitHub Actions run passes; fix any test/build issue before extending the PoC.
2. Resolve the 3 remaining licensing ambiguities if possible under Issue #1.
3. Replace hand-authored PoC fixtures one source at a time with real source-specific importers, starting with a small permissive source.
4. Pin real upstream versions/snapshots and record checksums/retrieval timestamps.
5. Add source-specific tests for `ingest_scope` / `excluded_scope` enforcement.
6. Build a small evaluation corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.
7. Only after real importer/provenance tests are stable, begin the localization engine/API layer.

## Not started

- Production source importers
- Translation/localization engine
- Desktop application
- Mobile application
- Database auto-updater
- Release pipeline

## Current technical direction (not yet final implementation)

- Cross-platform application direction: Tauri 2
- Core direction: Rust
- Local database direction: SQLite
- Deterministic conversion layer: OpenCC and/or compatible regional conversion engine
- Entity/terminology layer: multi-source knowledge base
- Phase 0.5 tooling: Python + PyYAML for fast validation only, not a commitment to the production runtime
- Optional future ambiguity layer: local LLM, only if needed

## Important constraint

Do not treat candidate sources as approved until licensing evidence is recorded in `data-registry/`. Importers must enforce `data-registry/sources.yaml`, including `ingest_allowed`, `ingest_scope`, `excluded_scope` and pack separation. PoC fixture data is never authoritative upstream evidence.
