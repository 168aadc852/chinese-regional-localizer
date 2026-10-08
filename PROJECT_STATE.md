# Current Project State

Updated: 2026-10-08

## Status

**Phase 3I desktop authenticated data-update UI is complete and merged. Phase 0 source whitelist review now has a usable conclusion for all 24 machine source IDs.**

Phase 3J / Issue #38 implementation is tested and open for PR review; its GitHub
`test` check passed. It is not closed or merged until protected-main review/merge.

Implementation: [PR #45](https://github.com/168aadc852/chinese-regional-localizer/pull/45).
Working identity is HanContext; repository/application identifiers remain unchanged.

The project currently has:
- governed, licence-aware source ingestion;
- a completed first-pass source whitelist with unresolved-rights sources locked to reference-only/non-ingest status;
- SQLite v0.2 shared data with version/provenance history;
- Python reference localization plus Rust shared-core/runtime support for CN/HK/TW routes;
- private user-dictionary support;
- Tauri 2 desktop UI with validated database selection;
- versioned non-secret desktop database preferences, atomic saving, safe restart fallback and persisted dictionary enabled state (Phase 3J implementation);
- signed, authenticated shared-data update discovery/install/rollback;
- offline fixture-backed CI for Python, Rust, Tauri and package/catalog flows;
- protected `main` requiring PRs and the `test` status check.

## Open items

- OpenCC remains a partial/reference implementation rather than full upstream runtime parity.
- Desktop updates are manual only; retry/resume and production hosting are not implemented.
- Release trust-root rotation/delegation, key-vault/HSM operations and installer signing are not implemented.
- Phase 3J review/CI/merge remains pending; persistence covers database choices and dictionary enabled state only, not general Alpha preferences.
- Production Windows/macOS installers and mobile packaging are not started.

## Likely next work

1. Review and merge Phase 3J / Issue #38 through protected `main`.
2. Review the HanContext Alpha MVP specification and small, separately scoped issues; do not start Alpha implementation automatically.
   Planning is in `docs/HANCONTEXT_ALPHA_MVP.md` and `docs/PRODUCT_REQUIREMENTS.md`.
   Six unimplemented proposals are tracked in Issues #46–#51, with dependencies and separate approval gates.
3. Track remaining release hosting, retry/resume, key operations and production packaging work separately.

Planned order: Desktop Alpha → CLI → MCP Server → production Windows/macOS
packaging/signing → editor/plugin integrations → optional AI assistance. Each client
reuses the same Core / Runtime. Context inheritance, context-sensitive My Terms,
alternatives, modes and Alpha UX remain unimplemented. Next review the specification
and issues and approve each scope separately, without automatically starting work.

For detail, load only what the task needs:
- history: `docs/history/PROJECT_HISTORY.md`
- releases: `CHANGELOG.md`
- architecture decisions: `docs/adr/`
- task-specific guidance: `.ai/`
