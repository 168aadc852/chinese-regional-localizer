# Current Project State

Updated: 2026-10-07

## Status

**Phase 2D user-local control is implemented on the active development branch and awaiting CI/merge.**

Current foundations:
- data governance + machine-readable exact-resource ingestion policy;
- SQLite schema v0.2 with current/superseded source versions and licence-pack guardrails;
- update-safe/idempotent OpenCC, Wikidata and LSHK import paths;
- pinned LSHK Git-blob verification;
- deterministic shared reference engine with conservative entity matching, matcher caching, context constraints, staged rules and original-to-final span alignment;
- separate private `user_dictionary.sqlite` schema for protected terms and fixed overrides;
- user-local wrapper precedence: protected > user override > shared entity > shared regional term > generic/script rule;
- user dictionary management CLI plus optional `--user-db` support in the localization CLI;
- regression coverage for shared core hardening plus user-local precedence/persistence;
- Apache-2.0 licence for project-authored software, with third-party data kept separately licensed.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full runtime parity.
- Production Rust engine, desktop/mobile UI and release/update delivery pipeline are not started.

## Likely next work

1. Broader realistic evaluation corpus and performance corpus.
2. Measure matcher/database behavior at larger scale and lock benchmark targets.
3. Rust port after reference behavior remains stable under those tests.
4. Tauri UI after the core/runtime contract is locked.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
