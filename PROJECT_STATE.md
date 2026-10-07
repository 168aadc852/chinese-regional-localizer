# Current Project State

Updated: 2026-10-07

## Status

**Phase 3G signed release metadata authenticity is implemented pending final PR validation/merge.**

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
- Tauri 2 desktop shell with thin static frontend and Rust-managed database settings/native file pickers;
- full local database paths stay inside Rust; frontend receives filenames/status only;
- versioned shared-data package manifest v1 with runtime compatibility, pack type, byte size, SHA-256 and source provenance summary;
- governed Python release builder that re-checks exact source allow-lists and licence-pack consistency before packaging;
- Rust package validation for safe paths, SHA-256, SQLite integrity, required tables and pack agreement;
- immutable versioned local package store with staged validation, `current`/`previous` activation state and rollback;
- failed package installation leaves the currently active database unchanged;
- Ed25519 detached signature envelope v1 for exact package-manifest and release-catalog bytes;
- pinned trusted public-key set with key IDs derived from public-key SHA-256 fingerprints and support for multiple pinned keys;
- domain-separated package/catalog signatures using strict Ed25519 verification;
- release catalog v1 with monotonic sequence rollback protection, signed expiry and exact package-manifest SHA-256/identity binding;
- deterministic unsigned release-catalog builder and offline signing CLI that reads production secret material only from an explicitly supplied external file;
- production private release keys are not stored in the repository or application bundle;
- CI includes package/catalog smoke plus Python/Rust/Tauri suites;
- Apache-2.0 licence for project-authored software, with third-party data kept separately licensed;
- `main` protected by an active GitHub ruleset requiring PRs and the `test` status check, with force pushes and deletion blocked.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.
- Runtime API v1 explanation enrichment is designed for the current fixture-backed sources; future source/importer additions may add fields without changing existing v1 meanings.
- Phase 3G authenticates offline release metadata only: Internet update discovery/download is not implemented.
- Remote trust-root rotation/delegation, HSM/key-vault integration and installer signing are not implemented.
- Desktop database chooser selections are session-only; persistent user preferences across app restarts are not implemented.
- Production installers/signing and mobile packaging are not started.
- Performance numbers from GitHub-hosted runners are smoke signals only, not product targets.

## Likely next work

1. Add a safe network update discovery/download layer that requires Phase 3G catalog/package authenticity before invoking the Phase 3F package store.
2. Persist the highest trusted catalog sequence and active package preference safely across app restarts.
3. Design release-key operational procedures/rotation without placing private keys in the repository.
4. Add Windows/macOS installer packaging and platform signing after database-update behavior is stable.
5. Expand fixture-backed domains/source coverage while preserving Runtime API v1 compatibility.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
