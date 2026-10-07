# Current Project State

Updated: 2026-10-07

## Status

**Phase 3I desktop authenticated data-update UI is complete and merged.**

The project currently has:
- governed, licence-aware source ingestion;
- SQLite v0.2 shared data with version/provenance history;
- Python reference localization plus Rust shared-core/runtime support for CN/HK/TW routes;
- private user-dictionary support;
- Tauri 2 desktop UI with validated database selection;
- signed, authenticated shared-data update discovery/install/rollback;
- offline fixture-backed CI for Python, Rust, Tauri and package/catalog flows;
- protected `main` requiring PRs and the `test` status check.

## Open items

- Issue #1: unresolved licence reviews for DoJ Glossaries and the DBnary snapshot licence.
- OpenCC remains a partial/reference implementation rather than full upstream runtime parity.
- Desktop updates are manual only; retry/resume and production hosting are not implemented.
- Release trust-root rotation/delegation, key-vault/HSM operations and installer signing are not implemented.
- Desktop database choice is session-only; general product preferences are not persisted.
- Production Windows/macOS installers and mobile packaging are not started.

## Likely next work

1. Production release hosting/CDN layout plus interrupted-download retry/resume.
2. Release-key operational procedures and rotation.
3. Persisted non-secret desktop settings.
4. Windows/macOS packaging and signing.
5. Expand fixture-backed source/domain coverage without breaking Runtime API v1.

For detail, load only what the task needs:
- history: `docs/history/PROJECT_HISTORY.md`
- releases: `CHANGELOG.md`
- architecture decisions: `docs/adr/`
- task-specific guidance: `.ai/`
