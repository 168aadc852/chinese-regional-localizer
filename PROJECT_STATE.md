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
- Expanded the source registry to 23 records after splitting DATA.GOV.HK from the Combined DoJ Glossaries.
- Opened GitHub Issue #1 to manage authoritative licence verification and whitelist approval.
- Completed first-pass formal reviews for 14 source records:
  - OpenCC — `approved_with_conditions`
  - Wikidata structured data — `approved`
  - Chinese Wikipedia — `approved_with_conditions`, isolated CC BY-SA data pack
  - Taiwan Government / NAER terminology open data — `approved_with_conditions` for individually verified OGL-Taiwan v1 datasets
  - DATA.GOV.HK — `approved_with_conditions`; portal terms allow commercial/non-commercial download, distribution and reproduction, but adaptation rights are not assumed
  - CC-CEDICT — `approved_with_conditions`, isolated CC BY-SA 4.0 pack
  - Words.hk — `approved_with_conditions`; public-domain word-list/pronunciation subset approved, full dictionary excluded from commercial-capable pack
  - Rime Cantonese — `approved_with_conditions`; CC BY 4.0 main content separated from ODbL map data
  - Unicode / Unihan / CLDR — `approved_with_conditions` for clearly identified Unicode-3.0 Data Files only
  - MusicBrainz — `approved_with_conditions`; Core Data CC0 only, Supplementary Data excluded
  - THUOCL — `approved_with_conditions`; published repo word lists/frequency signals only, with MIT/provenance safeguards
  - PanLex — `approved` under CC0 official snapshots
  - CC-Canto — `approved_with_conditions`, isolated CC BY-SA 3.0 pack
  - Chinese Wiktionary — `approved_with_conditions`, isolated CC BY-SA 4.0 text-derived pack
- Split the Combined DoJ Glossaries into a separate `pending_review` record because explicit reuse/adaptation rights have not yet been confirmed.

## In progress

- Review remaining candidate open datasets and licences.
- Build an authoritative data-source whitelist.
- Refine the terminology/entity database schema.
- Classify sources into permissive core, attribution-required packs, ShareAlike packs and reference-only sources.

## Next recommended work

1. Review DBnary.
2. Review ConceptNet.
3. Review Chinese Open WordNet.
4. Review HKCanCor.
5. Review LSHK Jyutping Table.
6. Review Kaifangcidian / Open Chinese Dictionary datasets individually.
7. Review CFDICT.
8. Review OpenHowNet.
9. Continue looking for an explicit reuse/adaptation licence for the Combined DoJ Glossaries.
10. Define a machine-readable source manifest/schema reflecting the licence classes now emerging.
11. Produce a small test corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.

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
