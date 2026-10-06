# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research**

A parallel **Phase 0.5 proof-of-concept track** is now ready to begin using only manifest-approved sources.

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

1. Resolve the 3 remaining licensing ambiguities if possible under Issue #1.
2. Begin Issue #2 with source/file/field-level importer allowlists based on `data-registry/sources.yaml`.
3. Create the first SQLite database from `schema/sqlite-v0.1.sql`.
4. Build a small test corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.
5. Build the first proof-of-concept dataset using only approved non-ShareAlike sources.
6. Demonstrate provenance for every returned result and safe rejection of a pending source.
7. Only after the proof of concept is stable, begin the localization engine/API layer.

## Not started

- Production database importer
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
- Optional future ambiguity layer: local LLM, only if needed

## Important constraint

Do not treat candidate sources as approved until licensing evidence is recorded in `data-registry/`. Importers must enforce `data-registry/sources.yaml`, including `ingest_allowed`, `ingest_scope`, `excluded_scope` and pack separation.
