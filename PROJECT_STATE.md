# Current Project State

Updated: 2026-10-10

## Status

**Phase 3J / Issue #38 desktop settings persistence is complete. Phase 0 source whitelist review now has a usable conclusion for all 24 machine source IDs.**

Phase 3J persisted database choices and the private-dictionary enabled state only.
Issue #60 Stage A is implemented on `feat/desktop-alpha-stage-a` for PR review:
four UI locales, System/Light/Dark/E-ink / Mono, accessibility baseline patterns,
and separate persisted presentation preferences in desktop settings v2.
Localization defaults/context/modes are not persisted or exposed by Stage A.
Runtime API v1 remains unchanged. See `docs/DESKTOP_UI_FOUNDATION.md`.

Working product identity is HanContext; repository/application identifiers remain unchanged.

Issues #46–#49 are merged. Issue #49 adds private-store v2, My Terms/multiple
dictionaries and request-bound remembered choices while preserving Runtime API v1
and #47/#48 protections. No #50 dictionary manager or desktop choice/context/remember controls are implemented yet.

The project currently has:
- governed, licence-aware source ingestion;
- a completed first-pass source whitelist with unresolved-rights sources locked to reference-only/non-ingest status;
- SQLite v0.2 shared data with version/provenance history;
- Python reference localization plus Rust shared-core/runtime support for CN/HK/TW routes;
- versioned private-store v2 with atomic legacy migration, multiple dictionaries, context/locale-scoped preferences, dictionary activation and validated CSV core operations;
- versioned usage-context profiles with seven built-ins, validated single-parent inheritance and independent local storage;
- explicit context-aware shared terminology selection with Python/Rust parity, inherited fallback and no-guess ties;
- deterministic alternative candidates, exact occurrence spans, in-memory one-time edits and revision-checked LIFO undo;
- request-bound remembered preferences with exact-context/all-context scope and target-locale isolation;
- Tauri 2 desktop UI with validated database selection;
- versioned non-secret desktop database preferences, atomic saving, safe restart fallback and persisted dictionary enabled state;
- signed, authenticated shared-data update discovery/install/rollback;
- offline fixture-backed CI for Python, Rust, Tauri and package/catalog flows;
- protected `main` requiring PRs and the `test` status check.

## Open items

- OpenCC remains a partial/reference implementation rather than full upstream runtime parity.
- Desktop updates are manual only; retry/resume and production hosting are not implemented.
- Release trust-root rotation/delegation, key-vault/HSM operations and installer signing are not implemented.
- Desktop context/alternative/remember controls, polished dictionary management and localization modes are not implemented; #49's core/storage implementation is separate from desktop exposure.
- Production Windows/macOS installers and mobile packaging are not started.

## Likely next work

1. Review/merge Issue #60 Stage A before separately approving the next #50
   Desktop/Core bridge and workflow stage; the #50 Pre-UX contracts are approved.
2. Keep #50 and #51 as separately approved, focused work packages under
   `docs/HANCONTEXT_ALPHA_MVP.md`; do not implement them automatically.
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
