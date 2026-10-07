# Current Project State

Updated: 2026-10-07

## Status

**Phase 3A Rust shared-core reference parity is implemented on the active branch and awaiting final CI/merge.**

Current foundations:
- data governance + machine-readable exact-resource ingestion policy;
- SQLite schema v0.2 with current/superseded source versions and licence-pack guardrails;
- update-safe/idempotent OpenCC, Wikidata and LSHK import paths;
- deterministic Python shared reference engine with conservative entity matching, matcher caching, context constraints, staged rules and original-to-final span alignment;
- separate private `user_dictionary.sqlite` for protected terms and fixed overrides;
- user-local precedence: protected > user override > shared entity > regional terminology > generic/script rule;
- project-authored realistic synthetic evaluation corpus and reproducible Python performance baseline tooling;
- first Rust shared-core reference runtime reading the same SQLite v0.2 database directly;
- Rust parity coverage against the same short and realistic corpus, plus ambiguity and false-positive regressions;
- strict Rust CI checks: rustfmt, Clippy and cargo tests;
- Apache-2.0 licence for project-authored software, with third-party data kept separately licensed.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.
- Rust user-local dictionary parity and the full explanation/alignment payload are not yet implemented.
- Tauri desktop/mobile UI and release/update delivery pipeline are not started.
- Performance numbers from GitHub-hosted runners are smoke signals only, not product targets.

## Likely next work

1. Phase 3B: port user-local protected terms / overrides to Rust and lock explanation/alignment parity.
2. Add Rust benchmark measurements using the same corpus after behavioral parity is complete.
3. Stabilize a small runtime API suitable for Tauri commands.
4. Start Tauri UI only after the Rust runtime contract is stable.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
