# Current Project State

Updated: 2026-10-08

## Status

**Phase 3J / Issue #38 desktop settings persistence is complete. Phase 0 source whitelist review now has a usable conclusion for all 24 machine source IDs.**

Persistence covers database choices and the private-dictionary enabled state only.
General Alpha preferences are not implemented. Runtime API v1 remains unchanged.

The project currently has:
- governed, licence-aware source ingestion;
- a completed first-pass source whitelist with unresolved-rights sources locked to reference-only/non-ingest status;
- SQLite v0.2 shared data with version/provenance history;
- Python reference localization plus Rust shared-core/runtime support for CN/HK/TW routes;
- private user-dictionary support;
- Tauri 2 desktop UI with validated database selection;
- versioned non-secret desktop database preferences, atomic saving, safe restart fallback and persisted dictionary enabled state;
- signed, authenticated shared-data update discovery/install/rollback;
- offline fixture-backed CI for Python, Rust, Tauri and package/catalog flows;
- protected `main` requiring PRs and the `test` status check.

## Open items

- OpenCC remains a partial/reference implementation rather than full upstream runtime parity.
- Desktop updates are manual only; retry/resume and production hosting are not implemented.
- Release trust-root rotation/delegation, key-vault/HSM operations and installer signing are not implemented.
- General Alpha preferences are not implemented; persistence covers database choices and private-dictionary enabled state only.
- Production Windows/macOS installers and mobile packaging are not started.

## Likely next work

1. Prepare the HanContext Alpha MVP specification and small, separately scoped issues; do not start Alpha implementation automatically.
2. Track remaining release hosting, retry/resume, key operations and production packaging work separately.

For detail, load only what the task needs:
- history: `docs/history/PROJECT_HISTORY.md`
- releases: `CHANGELOG.md`
- architecture decisions: `docs/adr/`
- task-specific guidance: `.ai/`
