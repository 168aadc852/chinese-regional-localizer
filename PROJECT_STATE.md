# Current Project State

Updated: 2026-10-09

## Status

**Phase 3J / Issue #38 desktop settings persistence is complete. Phase 0 source whitelist review now has a usable conclusion for all 24 machine source IDs.**

Desktop settings persistence covers database choices and the private-dictionary enabled state only.
General Alpha preferences are not implemented. Runtime API v1 remains unchanged.

Working product identity is HanContext; repository/application identifiers remain unchanged.

Issue #46's approved context-profile foundation is merged. Issue #47's approved
context-aware regional terminology selection is implemented in this focused PR;
closeout remains subject to protected-main CI, review and merge. It preserves
Runtime API v1 and adds no desktop context controls or context-sensitive My Terms.

The project currently has:
- governed, licence-aware source ingestion;
- a completed first-pass source whitelist with unresolved-rights sources locked to reference-only/non-ingest status;
- SQLite v0.2 shared data with version/provenance history;
- Python reference localization plus Rust shared-core/runtime support for CN/HK/TW routes;
- private user-dictionary support;
- versioned usage-context profiles with seven built-ins, validated single-parent inheritance and independent local storage;
- explicit context-aware shared terminology selection with Python/Rust parity, inherited fallback and no-guess ties;
- Tauri 2 desktop UI with validated database selection;
- versioned non-secret desktop database preferences, atomic saving, safe restart fallback and persisted dictionary enabled state;
- signed, authenticated shared-data update discovery/install/rollback;
- offline fixture-backed CI for Python, Rust, Tauri and package/catalog flows;
- protected `main` requiring PRs and the `test` status check.

## Open items

- OpenCC remains a partial/reference implementation rather than full upstream runtime parity.
- Desktop updates are manual only; retry/resume and production hosting are not implemented.
- Release trust-root rotation/delegation, key-vault/HSM operations and installer signing are not implemented.
- General Alpha preferences, desktop context controls, context-sensitive My Terms, alternatives and modes are not implemented.
- Production Windows/macOS installers and mobile packaging are not started.

## Likely next work

1. Review and merge the focused Issue #47 selection PR; details are in `docs/CONTEXT_SELECTION.md`.
2. Separately approve the next Alpha package before implementation.
   Remaining Alpha proposals (#48–#51) retain their dependencies and approval gates in
   `docs/HANCONTEXT_ALPHA_MVP.md`; none are implemented automatically.
3. Track remaining release hosting, retry/resume, key operations and production packaging work separately.

Planned order: Desktop Alpha → CLI → MCP Server → production Windows/macOS
packaging/signing → editor/plugin integrations → optional AI assistance. Each client
reuses the same Core / Runtime. Planning and issue creation do not authorize automatic
implementation of the Alpha feature set.

For detail, load only what the task needs:
- history: `docs/history/PROJECT_HISTORY.md`
- releases: `CHANGELOG.md`
- architecture decisions: `docs/adr/`
- task-specific guidance: `.ai/`
