# Current Project State

Updated: 2026-10-07

## Status

**Phase 3B Rust user-local control parity is implemented on the active branch and awaiting final clean CI/merge.**

Current foundations:
- data governance + machine-readable exact-resource ingestion policy;
- SQLite schema v0.2 with current/superseded source versions and licence-pack guardrails;
- update-safe/idempotent OpenCC, Wikidata and LSHK import paths;
- deterministic Python shared reference engine with conservative entity matching, matcher caching, context constraints, staged rules and original-to-final span alignment;
- separate private `user_dictionary.sqlite` for protected terms and fixed overrides;
- user-local precedence: protected > user override > shared entity > regional terminology > generic/script rule;
- project-authored realistic synthetic evaluation corpus and reproducible Python performance baseline tooling;
- Rust shared-core reference runtime reading the same SQLite v0.2 database directly;
- Rust shared-core parity coverage against the same short and realistic corpus, plus ambiguity and false-positive regressions;
- Rust user-local layer reading the existing user dictionary database directly;
- Rust parity coverage for protected terms, fixed overrides, locale scope, specificity, longest-match behavior, disabled rules, ambiguous override review and global spans;
- Rust CLI optional `--user-db` integration;
- strict Rust CI checks: rustfmt, Clippy and Cargo tests;
- Apache-2.0 licence for project-authored software, with third-party data kept separately licensed.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.
- The Rust shared-engine explanation payload is still smaller than Python's full provenance/evidence/alignment payload; Phase 3B closes the user-local fields and global span gap required for the next runtime API step.
- Tauri desktop/mobile UI and release/update delivery pipeline are not started.
- Performance numbers from GitHub-hosted runners are smoke signals only, not product targets.

## Likely next work

1. Phase 3C: stabilize a small Rust application/runtime API and close remaining shared explanation/provenance payload gaps needed by UI.
2. Add Rust benchmark measurements using the same corpus and compare against the Python baseline.
3. Define Tauri commands around the stable Rust runtime contract.
4. Start the desktop UI only after the command/API contract is covered by tests.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
