# Changelog

## 2026-10-07 — Phase 3E safe desktop database settings

- Added Rust-managed mutable database configuration for the Tauri desktop shell.
- Added native Rust-side file pickers for the shared regional database and optional private user dictionary.
- Added read-only SQLite compatibility validation before a selected file can replace the active database configuration.
- Kept full local filesystem paths inside Rust-managed state; frontend JavaScript receives only filename/status/error metadata.
- Added a command to clear the optional private user dictionary without deleting its file.
- Made subsequent localization calls immediately use the currently accepted database configuration.
- Added a responsive Settings panel showing shared/user database readiness and selection controls.
- Preserved `CRL_SHARED_DB` and `CRL_USER_DB` as development startup fallbacks.
- Added Tauri unit tests for compatible/incompatible databases and path-privacy behavior.
- Added `docs/DESKTOP_DATABASE_SETTINGS.md` and ADR D-016 documenting the native-chooser/Rust-validation security boundary.
- Database chooser selections remain session-only; persistence and database download/update/rollback are deferred.

## 2026-10-07 — Phase 3D minimal Tauri desktop shell

- Added a Tauri 2 desktop application under `desktop/` with a static vanilla HTML/CSS/JavaScript frontend.
- Added one async `localize_text` Tauri command backed exclusively by Rust Runtime API v1.
- Kept shared/user SQLite paths in Rust managed state; frontend JavaScript cannot supply arbitrary database paths or query SQLite directly.
- Added development database configuration through `CRL_SHARED_DB` and optional `CRL_USER_DB` environment variables.
- Added a minimal responsive UI for source/target selection, input, localized output, review state and structured change/provenance display.
- Enabled Tauri's global JavaScript API so this first shell does not require Node.js, npm, Vite or a frontend framework.
- Added Tauri compile/format/Clippy validation to CI using the required Linux WebKitGTK dependencies.
- Added beginner Windows development guidance and ADR D-015 defining the thin-frontend/managed-database boundary.

## 2026-10-07 — Phase 3C Rust runtime API and explanations

- Added versioned Rust Runtime API v1 as the application-facing localization boundary.
- Unified shared-only and optional private `user_dictionary.sqlite` execution behind one request/response contract.
- Refactored the Rust CLI to delegate to Runtime API v1 instead of calling lower-level engines directly.
- Added shared entity explanation enrichment with concept metadata, Wikidata QID, confidence and current evidence records.
- Added shared term-rule enrichment with stage, rule/source/version/upstream provenance and confidence.
- Added UI-oriented original-input/final-output character spans where the lower-level event did not already provide them.
- Added Runtime API regression tests for versioning, entity evidence, term provenance and user-dictionary mode.
- Added a reproducible Rust benchmark binary using the existing realistic corpus and reporting determinism, throughput and latency.
- Added `docs/RUST_RUNTIME_API.md` and ADR D-014 requiring future Tauri/application code to use the Runtime API rather than duplicate localization logic.

## 2026-10-07 — Phase 3B Rust user-local control parity

- Added a Rust user-local control layer reading the existing separate `user_dictionary.sqlite` v0.1 database directly.
- Added protected terms and fixed overrides with the same precedence as Python: protected > user override > shared entity > regional terminology > generic/script conversion.
- Added longest user-surface matching, locale scoping, specificity ranking, priority handling, enabled/disabled filtering and no-guess ambiguity behavior.
- Added user explanation events with user rule IDs, notes, provenance and original/final character spans.
- Added global span offset handling for shared-engine events emitted between user-controlled segments.
- Added optional Rust CLI `--user-db` support.
- Added Rust user-local regression coverage corresponding to the existing Python scenarios, including conflict review and disabled-rule fallback.
- Added `rust/clippy.toml` with an eight-argument threshold for the test fixture insertion helper while keeping CI Clippy warnings denied.
- Added `docs/RUST_USER_LOCAL_CONTROL.md`.

## 2026-10-07 — Phase 3A Rust shared-core parity

- Added the first Rust reference runtime under `rust/`, reading the existing SQLite v0.2 database directly.
- Added the current forward CN/HK/TW routes with conservative entity matching, current-version filtering, staged longest-match terminology rules and no-guess ambiguity behavior.
- Added a Rust CLI that returns structured JSON for demo/local testing.
- Added Rust parity tests against the same 9 short evaluation cases and 8 Phase 2E realistic synthetic cases used by the Python reference implementation.
- Added Rust regressions for ambiguous entities, equal-priority conflicting rules and short/common entity false positives.
- Added `Cargo.lock` and strict CI checks for `cargo fmt --check`, Clippy with warnings denied, and Cargo tests.
- Added `docs/RUST_REFERENCE_ENGINE.md` and ADR D-013 requiring behavioral parity before optimization or Tauri UI integration.
- Phase 3A deliberately leaves private user-dictionary parity and the full Python explanation/alignment payload for a later Rust phase.

## 2026-10-07 — Phase 2E evaluation and performance baselines

- Added a project-authored synthetic realistic corpus covering CN→HK, CN→TW and Hant→HK/TW behavior.
- Added exact-output corpus evaluation with category summaries and detailed failure diagnostics.
- Added throughput, p50/p95 latency, determinism and tracked-memory benchmark tooling.
- Added a short/common entity false-positive regression case.
- Added CI corpus evaluation and deliberately generous catastrophic-performance smoke limits.
- Added benchmark methodology and comparison guidance for the future Rust port.

## 2026-10-07 — Phase 2D user-local control

- Added separate private `user_dictionary.sqlite` storage.
- Added protected terms and locale-scoped fixed user overrides.
- Added deterministic precedence: protected term > user override > shared entity > regional terminology > generic/script conversion.
- Added user-dictionary management CLI and optional user DB integration in the localization CLI.
- Added persistence, overlap/longest-match, locale-scope and precedence regression tests.
- Added ADR D-012 and user-local-control documentation.

## 2026-10-07 — Phase 2C core hardening

- Added SQLite schema v0.2 and migration `0003_core_hardening.sql` while preserving v0.1 as the historical baseline.
- Added per-resource `source_versions.resource_key` and `is_current` lifecycle state.
- Made OpenCC/Wikidata/LSHK refreshes update-safe and repeat imports idempotent.
- Added manifest synchronization on every production import instead of only on database creation.
- Added exact machine-readable `ingest_resources` gates for implemented importers.
- Disabled automated DATA.GOV.HK ingestion while modification/adaptation rights remain unresolved.
- Added licence-pack enforcement through `build_metadata.pack_type` so core/attribution/share-alike data cannot silently mix.
- Added LSHK pinned Git-blob verification and current-pronunciation filtering.
- Updated Wikidata refresh semantics so superseded labels/aliases remain historical but no longer participate in runtime resolution.
- Added conservative entity-surface matching to reduce common-word false positives.
- Replaced per-call first-character matcher rebuilding with cached prefix tries.
- Added executable JSON context constraints for domain/neighbor/boundary-sensitive rules.
- Added original-to-final span alignment to change/review events.
- Corrected importer retrieval timestamps to default to current UTC instead of a fixed development date.
- Added regression coverage for update idempotency, source refreshes, scope gates, pack separation, false-positive entities, span alignment and pinned-blob rejection.
- Added Ruff/compile checks to CI.
- Added Apache-2.0 licensing for project-authored software while explicitly preserving separate third-party data licences.
- Updated README, roadmap, schema/importer/engine docs and source reviews for Phase 2C.

## 2026-10-07 — AI context refactor

- Refactored `AGENTS.md` from a preload checklist into a minimal task/context router.
- Added task-specific on-demand guides under `.ai/` for coding, data review, importers, database, engine, testing, docs and releases.
