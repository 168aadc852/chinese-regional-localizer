# Project State

Last updated: 2026-10-06

## Current phase

**Phase 0 — Data governance and source research**

## Objective of this phase

Establish a legally and technically traceable data foundation before writing the main localization application.

## Completed

- Defined the project as an offline-first CN / HK / TW Chinese regional localization tool.
- Agreed that GitHub should be the single source of truth for humans and AI tools.
- Agreed to separate data-source research from application implementation.
- Created the initial repository documentation structure.
- Created candidate source registry records for 22 sources.
- Expanded the source pool beyond Wikipedia/OpenCC to include multilingual lexicons, Cantonese resources, semantic networks, WordNet-style resources and official terminology datasets.
- Opened GitHub Issue #1 to manage authoritative licence verification and whitelist approval.
- Completed first-pass formal reviews for 4 sources:
  - OpenCC — `approved_with_conditions`
  - Wikidata structured data — `approved`
  - Chinese Wikipedia — `approved_with_conditions`, isolated CC BY-SA data pack
  - Taiwan Government / NAER terminology open data — `approved_with_conditions` for individually verified OGL-Taiwan v1 datasets

## In progress

- Review the remaining candidate open datasets and licences.
- Build an authoritative data-source whitelist.
- Refine the terminology/entity database schema.
- Classify candidate sources into permissive core, attribution-required packs, ShareAlike packs and reference-only sources.

## Next recommended work

1. Review Hong Kong Government / Department of Justice terminology, separating DATA.GOV.HK terms from any DoJ-site-specific terms.
2. Review CC-CEDICT.
3. Review Words.hk licensing boundaries.
4. Review Rime Cantonese.
5. Review Unicode / Unihan / CLDR.
6. Review MusicBrainz.
7. Review high-value new candidates: PanLex, THUOCL, CC-Canto and Chinese Wiktionary.
8. Review HKCanCor and LSHK Jyutping Table for Hong Kong lexical/pronunciation support.
9. Review semantic resources (Chinese Open WordNet, ConceptNet and OpenHowNet) for ambiguity handling.
10. Produce a small test corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.

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

Do not treat candidate sources as approved until licensing evidence is recorded in `data-registry/`.
