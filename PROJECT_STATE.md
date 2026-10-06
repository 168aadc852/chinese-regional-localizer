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
- Created the first candidate source registry placeholders.

## In progress

- Review candidate open datasets and licences.
- Build an authoritative data-source whitelist.
- Refine the terminology/entity database schema.

## Next recommended work

1. Verify OpenCC source/data licensing and intended use.
2. Verify Wikidata licensing and useful fields for regional aliases.
3. Define how Wikipedia-derived data must be isolated and attributed.
4. Review Taiwan government terminology datasets individually.
5. Review Hong Kong government / Department of Justice terminology datasets individually.
6. Review CC-CEDICT, Words.hk and Rime Cantonese licensing boundaries.
7. Add Unicode/Unihan/CLDR as candidate sources and review their licensing/use cases.
8. Produce a small test corpus covering films, people, IT terms, transport, legal terms and ambiguous words.

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
