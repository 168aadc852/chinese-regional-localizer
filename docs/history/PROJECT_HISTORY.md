# Project History

Read this file only when a task needs prior implementation context. Do not preload it.

Full historical detail remains available in Git commit history and closed Issues #2–#7.

## Milestones

- Phase 0: data governance/source whitelist established. 23 sources tracked; 20 have usable first-pass conclusions; 3 licensing questions remain open in Issue #1.
- Phase 0.5: manifest-enforced SQLite proof of concept completed.
- Phase 1A: LSHK Jyutping importer completed with provenance, checksums and multi-reading support.
- Phase 1B: OpenCC phrase importer completed with staged CN→Hant→HK/TW semantics and candidate preservation.
- Phase 1C: Wikidata EntityData importer completed with QID identity, CN/HK/TW localized names and no-fabrication behavior.
- Phase 2A: deterministic reference localizer completed: entity-first, longest-match, no-guess conflicts, explainable provenance.
- Phase 2B: OpenCC coverage expanded to STCharacters, HK/TW variant phrase exceptions and character variants.

## Key implementation anchors

- Source policy: `data-registry/sources.yaml`
- SQLite schema: `schema/sqlite-v0.1.sql`
- Pronunciation migration: `schema/migrations/0002_pronunciations.sql`
- Reference engine: `src/localizer_engine.py`
- OpenCC importer: `scripts/import_opencc_dictionaries.py`
- Wikidata importer: `scripts/import_wikidata_entities.py`
- LSHK importer: `scripts/import_lshk_jyutping.py`
- Engine behavior docs: `docs/LOCALIZER_ENGINE.md`

## Unresolved licensing items

- Combined DoJ Glossaries adaptation/redistribution scope
- DBnary snapshot licence version
- OpenHowNet core-data redistribution rights
