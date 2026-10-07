# Current Project State

Updated: 2026-10-07

## Status

**Phase 3H authenticated network update discovery/download is implemented pending final PR validation/merge.**

Current foundations:
- data governance + machine-readable exact-resource ingestion policy;
- SQLite schema v0.2 with current/superseded source versions and licence-pack guardrails;
- update-safe/idempotent OpenCC, Wikidata and LSHK import paths;
- deterministic Python reference engine and private `user_dictionary.sqlite` control layer;
- Rust shared-core and user-local parity for supported CN/HK/TW routes;
- versioned Rust Runtime API v1 with UI-oriented explanations/provenance;
- Tauri 2 desktop shell with thin frontend and Rust-managed validated database selection;
- immutable versioned shared-data packages with staging, current/previous activation and rollback;
- pinned Ed25519 release authenticity for package manifests and release catalogs, including signed expiry and catalog sequence rollback protection;
- Rust-side network update client with HTTPS-only configured origin, redirects disabled, bounded downloads and no generic frontend HTTP access;
- authenticated catalog discovery with the highest trusted sequence persisted atomically across restarts;
- manifest signature + exact catalog binding verified before database download;
- downloaded databases staged and passed through signed size/SHA-256, SQLite/package validation and the existing PackageStore before activation;
- failed network/auth/hash/SQLite/package installs leave the active database unchanged;
- fake-transport network regressions keep CI independent of the public Internet;
- CI includes package/catalog smoke plus Python/Rust/Tauri suites;
- Apache-2.0 licence for project-authored software, with third-party data separately licensed;
- `main` protected by an active GitHub ruleset requiring PRs and the `test` status check, with force pushes and deletion blocked.

## Open items

- Issue #1: 3 unresolved licensing reviews (DoJ Glossaries, DBnary snapshot licence, OpenHowNet core data).
- OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.
- Phase 3H provides the authenticated network core only; background scheduling, update UI, retry/resume and production hosting are not implemented.
- Remote trust-root rotation/delegation, HSM/key-vault integration and installer signing are not implemented.
- Desktop database chooser selections are session-only; general desktop preferences are not yet persisted as a product settings model.
- Production installers/signing and mobile packaging are not started.
- Performance numbers from GitHub-hosted runners are smoke signals only, not product targets.

## Likely next work

1. Wire narrow desktop commands/UI to check for and install authenticated shared-data updates without exposing arbitrary URLs.
2. Define production release hosting/CDN layout and retry/resume behavior for interrupted downloads.
3. Design release-key operational procedures/rotation without placing private keys in the repository.
4. Add Windows/macOS installer packaging and platform signing after update UX is stable.
5. Expand fixture-backed domains/source coverage while preserving Runtime API v1 compatibility.

Historical detail: `docs/history/PROJECT_HISTORY.md`, `CHANGELOG.md`, ADRs under `docs/adr/`, and closed GitHub Issues.
