# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research**

## Objective of this phase

Establish a legally and technically traceable data foundation before writing the main localization application.

## Completed

- Defined the project as an offline-first CN / HK / TW Chinese regional localization tool.
- Agreed that GitHub is the single source of truth for humans and AI tools.
- Created the initial repository documentation and source registry structure.
- Expanded the source registry to 23 records after separating DATA.GOV.HK from the Combined DoJ Glossaries.
- Opened GitHub Issue #1 to manage authoritative licence verification and whitelist approval.
- Completed first-pass formal reviews with usable conclusions for **20 of 23** source records.
- Established initial packaging classes: permissive/core, attribution-required, ShareAlike-isolated, non-commercial/reference-only and pending-rights layers.
- Added `docs/REVIEW_PROGRESS.md` and `docs/LICENSE_PACKAGING.md`.

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
   - project surfaces reviewed show CC BY-SA version inconsistency, so snapshot-level licence confirmation is required.
3. OpenHowNet downloadable HowNet core data
   - repository/API code is MIT;
   - data is downloaded separately and a data-specific redistribution licence has not yet been independently confirmed.

## Next recommended work

1. Resolve the 3 remaining licensing ambiguities if possible.
2. Convert the source registry into a machine-readable manifest (`sources.yaml` or JSON) with licence/packaging flags.
3. Finalise canonical database schema with mandatory provenance/licence fields.
4. Define importer allowlists at source/file/field level.
5. Build a small test corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.
6. Build the first proof-of-concept SQLite database from the safest sources only (for example Wikidata, PanLex, OpenCC-compatible data, Unicode/CLDR and other approved non-ShareAlike resources).

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

Do not treat candidate sources as approved until licensing evidence is recorded in `data-registry/`. Importers must use explicit allowlists so that excluded/non-commercial/ShareAlike data cannot silently enter the wrong release pack.
