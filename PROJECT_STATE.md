# Current Project State

Updated: 2026-10-07

## Status

**Phase 3C stable Rust runtime API, shared explanation enrichment and benchmark tooling are implemented on the active branch and awaiting final clean CI/merge.**

Current foundations:
- data governance + machine-readable exact-resource ingestion policy;
- SQLite schema v0.2 with current/superseded source versions and licence-pack guardrails;
- update-safe/idempotent OpenCC, Wikidata and LSHK import paths;
- deterministic Python reference engine and private `user_dictionary.sqlite` control layer;
- user-local precedence: protected > user override > shared entity > regional terminology > generic/script rule;
- project-authored realistic synthetic evaluation corpus and Python benchmark tooling;
- Rust shared-core and user-local parity for the supported CN/HK/TW routes;
- versioned Rust Runtime API v1 as the application-facing execution boundary;
- Runtime API shared-only and optional private user-dictionary modes;
- UI-oriented structured explanations with entity QID/evidence, term-rule source/version/upstream provenance and global character spans;
- Rust CLI delegated to Runtime API v1;
- reproducible Rust benchmark command reporting determinism, throughput and latency;
- strict Rust CI checks: rustfmt, Clippy and Cargo tests;
- Apache-2.0 licence for project-authored software, with third-party data kept separately licensed.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.
- Runtime API v1 explanation enrichment is designed for the current fixture-backed sources; future source/importer additions may add fields without changing existing v1 meanings.
- Tauri desktop/mobile UI and release/update delivery pipeline are not started.
- Performance numbers from GitHub-hosted runners are smoke signals only, not product targets.

## Likely next work

1. Phase 3D: add a minimal Tauri 2 command bridge that calls Runtime API v1 without duplicating localization logic.
2. Add a minimal desktop shell for text input, locale selection, localized output and review/change inspection.
3. Keep database paths/update delivery explicit and offline-first before broader UI polish.
4. Expand fixture-backed domains and source coverage while preserving runtime API compatibility.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
