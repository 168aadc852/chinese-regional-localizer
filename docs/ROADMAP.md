# Roadmap

## Phase 0 — Data governance — completed

Source policy, licensing evidence, initial schema and test corpus.

## Phase 1 — Data proof of concept — completed

SQLite prototype, provenance model and initial LSHK/OpenCC/Wikidata importers.

## Phase 2A–2B — Localization reference core — completed

Staged script/regional conversion, entity resolution, longest-match/no-guess behavior and OpenCC phrase/character/variant coverage.

## Phase 2C — Core hardening — completed

- update-safe source-version lifecycle;
- idempotent importer refreshes;
- exact machine-readable ingest-resource gates;
- licence-pack separation enforcement;
- conservative entity matching;
- cached trie matchers;
- executable context constraints;
- original-to-final span alignment;
- expanded regression/CI checks;
- explicit project software licence.

## Phase 2D — User-local control

- separate `user_dictionary.sqlite`;
- protected terms;
- user overrides and precedence;
- import/export of user dictionaries;
- broader realistic evaluation/performance corpus.

## Phase 3 — Production core and minimal application

- Rust implementation matching reference tests;
- Tauri 2 text input/output UI;
- source/target selection and detection;
- diff/review with final-span highlighting;
- source explanation cards;
- safe local database update flow.

## Phase 4 — File workflows

TXT/Markdown, subtitles, CSV, then Office/EPUB where justified.

## Phase 5 — Cross-platform distribution

Windows, Android, macOS, iOS/iPadOS.

## Phase 6 — Optional ambiguity intelligence

Only after deterministic methods are measured. Consider a local model for unresolved contextual cases rather than routing every conversion through an LLM.
