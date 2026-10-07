# Current Project State

Updated: 2026-10-07

## Status

**Phase 3E safe desktop database settings and chooser is implemented pending final PR merge.**

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
- Tauri 2 desktop shell under `desktop/`, using static vanilla HTML/CSS/JavaScript;
- desktop frontend uses narrow Tauri commands and never queries SQLite directly;
- Rust-managed shared/user database configuration with native file pickers;
- Rust-side read-only SQLite compatibility validation before accepting selected databases;
- full local database paths remain inside Rust; frontend receives filenames/status only;
- optional private user dictionary can be selected or cleared for the current session;
- environment-variable startup configuration remains available as a development fallback;
- beginner Windows development guide for building the demo DB and launching the desktop shell;
- CI validation for Python, Rust shared runtime and Tauri format/compile/Clippy/unit tests;
- Apache-2.0 licence for project-authored software, with third-party data kept separately licensed;
- `main` is protected by an active GitHub ruleset requiring PRs and the `test` status check, with force pushes and deletion blocked.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.
- Runtime API v1 explanation enrichment is designed for the current fixture-backed sources; future source/importer additions may add fields without changing existing v1 meanings.
- The current Tauri shell is development-only: production database packaging/updating, installer/signing and mobile packaging are not started.
- Phase 3E database chooser selections are session-only; persistent preferences across app restarts are not implemented.
- Performance numbers from GitHub-hosted runners are smoke signals only, not product targets.

## Likely next work

1. Design packaged shared-database delivery, version checks and offline-safe update/rollback behavior.
2. Decide and implement safe persistence for desktop database preferences without weakening the Rust-managed path boundary.
3. Add Windows/macOS installer packaging and signing only after desktop data/update behavior is stable.
4. Expand fixture-backed domains and source coverage while preserving Runtime API v1 compatibility.
5. Start mobile packaging only after the desktop runtime/update contract is stable.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
