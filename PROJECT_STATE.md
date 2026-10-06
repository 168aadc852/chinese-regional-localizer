# Project State

Last updated: 2026-10-06

## Current phase

- **Phase 0** — data governance remains open for 3 unresolved sources.
- **Phase 0.5** — SQLite/manifest PoC complete and CI-validated.
- **Phase 1A** — LSHK pronunciation importer complete and CI-validated.
- **Phase 1B** — initial OpenCC phrase importer complete and CI-validated.
- **Phase 1C** — Wikidata entity/localized-name importer complete and CI-validated.
- **Phase 2A** — deterministic localization/resolution engine complete and CI-validated.
- **Phase 2B** — OpenCC character/regional-variant expansion complete and CI-validated.

The project can now build a small offline SQLite database, recognize regional entity names, perform phrase and character script conversion, apply HK/TW regional terminology and variants, preserve phrase exceptions, and return source-aware explanations/review flags.

## Objective

Build a legally and technically traceable offline CN / HK / TW localization foundation and deterministic engine before application UI work.

## Major completed capabilities

### Governance / data safety

- GitHub is the durable source of truth for humans and AI tools.
- 23 source records are tracked; 20 have usable first-pass conclusions.
- `data-registry/sources.yaml` is the machine-readable ingestion gate.
- Licence packs remain separable: core, attribution, ShareAlike, reference-only and pending.
- SQLite schema v0.1 preserves source/version/evidence provenance.

### Phase 1A — LSHK Jyutping

- Pinned upstream commit `dad2dd6d6f02fc51138ecc6818f7b38eba5c2ad3`.
- Multiple readings per character supported through `schema/migrations/0002_pronunciations.sql`.
- Strict TSV validation, SHA-256, revision/URL provenance and offline tests.
- Issue #3 closed.

### Phases 1B / 2B — OpenCC

Pinned upstream commit:

`3ac34aa439a9908dd49fa92b5174b46314787ac2`

Current imported forward resources:

Script `zh-CN -> zh-Hant`:
1. `STPhrases.txt`
2. `STCharacters.txt`

Hong Kong `zh-Hant -> zh-HK`:
1. `HKPhrases.txt`
2. `HKVariantsPhrases.txt`
3. `HKVariants.txt`

Taiwan `zh-Hant -> zh-TW`:
1. `TWPhrases.txt`
2. `TWVariantsPhrases.txt`
3. `TWVariants.txt`

Implemented behavior:

- staged conversion is preserved instead of flattening dictionaries;
- dictionary-level short-circuit precedence is represented separately from candidate rank;
- multiple target candidates remain stored in source order;
- phrase exceptions win through longest-match behavior before character fallback;
- each dictionary has its own source version, pinned revision, URL and SHA-256;
- strict OpenCC header/line validation remains enforced.

Regression coverage now includes:

- `布拉德·皮特 -> 畢·彼特` (HK regional phrase);
- `人工智能 -> 人工智慧` (TW regional vocabulary);
- `一见钟情 -> 一見鍾情` (ST phrase);
- `见 -> 見` (ST character fallback);
- `檯 -> 枱` (HK character variant);
- `爲 -> 為` (TW character variant);
- `張棟樑` remains unchanged because the Taiwan phrase exception prevents the `樑 -> 梁` character rule from damaging the name;
- identity and multi-candidate mappings.

OpenCC implementation/docs:

- `scripts/import_opencc_dictionaries.py`
- `tests/test_opencc_importer.py`
- `data/fixtures/opencc/`
- `docs/OPENCC_IMPORTER.md`

Issues #4 and #7 cover the initial phrase and expanded variant work respectively.

### Phase 1C — Wikidata entities

- Wikidata structured EntityData is used only within reviewed CC0 scope.
- Stable QIDs, explicit concept types, regional labels/aliases, revisions, retrieval timestamps, URLs and canonical entity checksums are preserved.
- Missing `zh-CN` / `zh-HK` / `zh-TW` labels are not invented from generic Chinese labels.
- Issue #5 closed.

### Phase 2A — deterministic reference engine

- `src/localizer_engine.py` defines tested behavior before the future Rust port.
- `scripts/localize_text.py` provides a local CLI.
- `scripts/build_demo_database.py` creates a small offline fixture database.
- Entity names are resolved before generic terms and protected from later conversion.
- Longest match wins before rule priority.
- Equal-priority conflicts and ambiguous entities are preserved and flagged `review_needed` instead of guessed.
- CN→HK/TW uses explicit staged routes.
- Applied decisions return source/QID/rule/version/URL/checksum evidence where available.
- D-009 records the precedence/no-guess contract.
- Issue #6 closed.

## Pending licensing clarification

1. Combined DoJ Glossaries of Legal Terms — derived/adaptation rights remain unclear.
2. DBnary — exact snapshot CC BY-SA version must be pinned.
3. OpenHowNet downloadable core data — data-specific redistribution rights remain unconfirmed.

## Next recommended work

1. Add a separate user dictionary / protected-term layer with explicit precedence over canonical rules.
2. Expand the regression/evaluation corpus beyond fixtures into realistic paragraphs covering IT, transport, entertainment, legal terminology, punctuation and ambiguous words.
3. Add reverse-direction resources/routes only after their OpenCC semantics are explicitly modelled and tested.
4. Continue Issue #1 for the three unresolved data licences.
5. Once reference behavior is stable, port the deterministic engine semantics to Rust.
6. Then expose the stable core through Tauri desktop/mobile UI and database updater workflows.

## Not started

- Production Rust localization engine
- Desktop application
- Mobile application
- Production database auto-updater/release pipeline

## Current technical direction

- Cross-platform app: Tauri 2
- Production core: Rust
- Local database: SQLite
- Data-build/import tooling: Python where practical
- Deterministic conversion: staged OpenCC-compatible rules + entity/localized-name resolution + future user overrides
- Optional local LLM: only for unresolved ambiguity after deterministic/context rules

## Important constraints

- Importers must enforce `data-registry/sources.yaml`.
- Pending/reference-only/rejected data must never silently enter redistributable builds.
- Regional names must not be invented when evidence is missing.
- Ambiguity must be surfaced rather than guessed.
- OpenCC generated dictionaries, reverse directions and full runtime segmentation/match semantics are not yet claimed as implemented.
