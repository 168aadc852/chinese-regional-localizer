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

## In progress

- Review candidate open datasets and licences.
- Build an authoritative data-source whitelist.
- Refine the terminology/entity database schema.
- Classify candidate sources into permissive core, attribution-required packs, ShareAlike packs and reference-only sources.

## Next recommended work

1. Verify OpenCC source/data licensing and intended use.
2. Verify Wikidata licensing and useful fields for regional aliases.
3. Review high-value new candidates: PanLex, THUOCL, CC-Canto and Chinese Wiktionary.
4. Define how Wikipedia/Wiktionary/DBnary/ConceptNet ShareAlike-derived data must be isolated and attributed.
5. Review Taiwan government terminology datasets individually.
6. Review Hong Kong government / Department of Justice terminology datasets individually.
7. Review HKCanCor and LSHK Jyutping Table for Hong Kong lexical/pronunciation support.
8. Review semantic resources (Chinese Open WordNet and OpenHowNet) for ambiguity handling.
9. Produce a small test corpus covering films, people, IT terms, transport, legal terms, Cantonese/HK terms and ambiguous words.

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
