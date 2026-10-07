# Current Project State

Updated: 2026-10-07

## Status

**Phase 3D minimal Tauri 2 command bridge and desktop shell is implemented.**

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
- Tauri 2 desktop shell under `desktop/`, using static vanilla HTML/CSS/JavaScript and a Rust-managed database configuration;
- desktop frontend invokes one `localize_text` command backed by Runtime API v1 and does not query SQLite directly;
- development database configuration through Rust-side `CRL_SHARED_DB` and optional `CRL_USER_DB` startup state;
- beginner Windows development guide for building the demo DB and launching the desktop shell;
- CI validation for Tauri Linux prerequisites, rustfmt, compile, Clippy and desktop unit tests;
- strict shared Rust CI checks: rustfmt, Clippy, Cargo tests and benchmark smoke;
- Apache-2.0 licence for project-authored software, with third-party data kept separately licensed.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.
- Runtime API v1 explanation enrichment is designed for the current fixture-backed sources; future source/importer additions may add fields without changing existing v1 meanings.
- The current Tauri shell is development-only: production database packaging/updating, installer/signing and mobile packaging are not started.
- The current database path configuration is startup/environment based; a user-facing settings/database chooser still needs a Rust-side path-selection design.
- Performance numbers from GitHub-hosted runners are smoke signals only, not product targets.

## Likely next work

1. Add a safe desktop settings/database chooser layer without exposing arbitrary paths to frontend JavaScript.
2. Design packaged shared-database delivery, version checks and offline-safe update/rollback behavior.
3. Add Windows/macOS installer packaging and signing only after desktop data/update behavior is stable.
4. Expand fixture-backed domains and source coverage while preserving Runtime API v1 compatibility.
5. Start mobile packaging only after the desktop runtime/update contract is stable.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
