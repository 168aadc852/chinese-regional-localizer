# Current Project State

Updated: 2026-10-07

## Status

Development is paused after **Phase 2B**.

Completed core foundations:
- data governance + machine-readable source policy;
- SQLite provenance schema;
- LSHK pronunciation importer;
- OpenCC CN→Hant→HK/TW forward phrase/character/variant coverage;
- Wikidata localized-entity importer;
- deterministic reference engine with entity-first, longest-match and no-guess behavior.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- Production Rust engine, desktop/mobile UI and update/release pipeline are not started.

## Likely next work

1. User dictionary / protected terms.
2. Broader realistic evaluation corpus.
3. Rust port after reference behavior stabilizes.
4. Tauri UI after the core is stable.

## Technical direction

Python reference/build tooling + SQLite now; planned production core is Rust + Tauri 2. Normal localization remains offline-first.

Historical detail: `docs/history/PROJECT_HISTORY.md` and closed Issues #2–#7.
