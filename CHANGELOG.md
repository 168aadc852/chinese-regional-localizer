# Changelog

## 2026-10-07

- Refactored `AGENTS.md` from a preload checklist into a minimal task/context router.
- Added task-specific on-demand guides under `.ai/` for coding, data review, importers, database, engine, testing, docs and releases.
- Reduced `PROJECT_STATE.md` to current status only; moved compact historical context to `docs/history/PROJECT_HISTORY.md` while full detail remains in Git/closed Issues.
- Split architecture decisions into individual ADR files under `docs/adr/`; `docs/DECISIONS.md` is now a compatibility pointer.
- Added D-010 requiring on-demand AI context loading to reduce repeated token cost.
- Minimized Gemini, Claude and Copilot repository instructions so they defer to `AGENTS.md` without preloading additional files.

## 2026-10-06

- Created the initial governance-first repository structure.
- Added AI-agent handoff documentation.
- Added draft data policy, data-source registry, schema and roadmap.
- Expanded the data-source registry to 23 records after separating DATA.GOV.HK from the Combined DoJ Glossaries.
- Completed usable first-pass reviews for 20 of 23 source records.
- Added machine-readable `data-registry/sources.yaml` ingestion policy.
- Added source manifest schema and licence packaging model.
- Added SQLite schema v0.1 with provenance and licence-pack metadata.
- Implemented manifest-enforced Phase 0.5 SQLite PoC builder.
- Added non-authoritative CN/HK/TW test fixtures and automated tests.
- Added GitHub Actions CI; PoC unit tests and demo database build passed.
- Added Phase 1A real importer for the LSHK Jyutping Table with pinned revision, SHA-256 provenance, strict TSV validation, multi-reading support and CI tests.
- Added pronunciation schema migration and documentation.
- Added Phase 1B real OpenCC phrase importer pinned to commit `3ac34aa439a9908dd49fa92b5174b46314787ac2`.
- Added staged `STPhrases`, `HKPhrases` and `TWPhrases` support without flattening CN/HK/TW conversion semantics.
- Preserved OpenCC multi-candidate mappings, identity mappings, dictionary/line provenance and per-file SHA-256 source versions.
- Added OpenCC parser/importer tests covering HK/TW proper-name and terminology examples; CI passed.
- Added Phase 1C Wikidata EntityData importer for CC0 structured entity labels and aliases.
- Added stable QID identity, explicit entity types, locale-specific CN/HK/TW names, aliases, revision-aware provenance, retrieval timestamps and canonical per-entity SHA-256.
- Added a hard no-fabrication rule: generic Chinese labels are not silently promoted to missing regional labels.
- Added offline Wikidata fixtures and tests for a person and a film; CI passed after final provenance updates.
- Added Phase 2A deterministic localization/reference engine in `src/localizer_engine.py` and CLI in `scripts/localize_text.py`.
- Added entity-first longest-match resolution, entity-span protection, staged OpenCC-compatible term processing and no-guess conflict handling.
- Added structured provenance/explanation output for entity and term-rule decisions.
- Added `data/fixtures/evaluation_cases.json`, one-command fixture DB builder and integrated engine tests.
- Verified person CN→HK/TW localization, Taiwan terminology conversion, script-stage conversion, mixed entity+term conversion, entity protection, longest-match precedence, and ambiguous entity/rule review behavior in CI.
- Added `docs/LOCALIZER_ENGINE.md` and decision D-009 defining deterministic localization precedence and no-guess policy.
- Added Phase 2B OpenCC character/variant coverage: `STCharacters.txt`, `HKVariantsPhrases.txt`, `HKVariants.txt`, `TWVariantsPhrases.txt` and `TWVariants.txt`.
- Added dictionary-level base priorities reflecting reviewed OpenCC short-circuit order, separate from candidate rank.
- Expanded OpenCC fixtures/importer tests from 3 to 8 dictionaries.
- Added regression coverage for `见→見`, `檯→枱`, `爲→為` and Taiwan phrase-exception protection of `張棟樑` from the `樑→梁` character rule.
- Updated OpenCC documentation to state current forward coverage and remaining generated/reverse/runtime limitations; all Phase 2B tests passed CI.
