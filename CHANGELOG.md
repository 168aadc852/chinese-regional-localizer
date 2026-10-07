# Changelog

## 2026-10-07 — Phase 3G signed release metadata authenticity

- Added Ed25519 detached signature envelope v1 using strict verification from a maintained Rust cryptography library.
- Added pinned trusted public-key records with key IDs derived from SHA-256 fingerprints of the exact public-key bytes and support for multiple pinned keys.
- Added separate signed-message domains for package manifests and release catalogs so a valid signature cannot be reused across metadata types.
- Added release catalog v1 with monotonic sequence rollback protection, signed expiry and unique package/version entries.
- Added exact package-manifest SHA-256 and identity binding between signed catalog entries and Phase 3F package manifests.
- Added deterministic unsigned release-catalog construction that re-validates Phase 3F package governance before cataloging a release.
- Added an offline Rust signing utility that reads a 32-byte Ed25519 seed from an explicitly supplied external base64 key file; production private keys are never stored in the repository or application bundle.
- Added regressions for payload tampering, wrong/unknown keys, cross-domain signature reuse, malformed signatures, expired catalogs, rollback sequences and package/catalog binding changes.
- Added unsigned catalog CI smoke plus committed/locked Rust crypto dependencies.
- Added `docs/RELEASE_SIGNING.md` and ADR D-018 documenting the pinned-key trust model and the rule that future network updates must pass both Phase 3G authenticity and Phase 3F package validation.
- Internet update discovery/download, remote trust-root rotation, HSM integration and installer signing remain out of scope for this phase.

## 2026-10-07 — Phase 3F versioned shared-database packages and rollback core

- Added `scripts/build_data_package.py` to create governed shared-database release directories with `package.json` plus immutable `regional.sqlite`.
- Added package manifest v1 fields for package/version identity, licence pack, minimum Runtime API, creation time, byte size, SHA-256 and current source/resource provenance.
- Added a release-time governance gate that re-checks `sources.yaml`, `ingest_allowed`, exact `ingest_resources` and licence-pack consistency before packaging.
- Added Rust package validation for manifest compatibility, safe relative filenames, size/SHA-256 integrity, SQLite integrity, required runtime tables and package/database pack agreement.
- Added a versioned Rust `PackageStore` using staging directories and immutable `<package_id>/<version>` installations.
- Added separate `current`/`previous` state and validated rollback; active databases are never overwritten in place.
- Added regressions for checksum mismatch, path traversal, pack mismatch, corrupt SQLite, failed-install safety and rollback.
- Added a CI smoke build of a governed package from the fixture demo database and switched Rust checks to the committed lockfile with `--locked`.
- Added `docs/DATA_PACKAGE_FORMAT.md`, updated licence-packaging guidance and ADR D-017.
- Network update discovery/download and publisher signing remain deliberately out of scope for this phase.

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
- Reduced `PROJECT_STATE.md` to current status only; moved compact historical context to `docs/history/PROJECT_HISTORY.md` while full detail remains in Git/closed Issues.
- Split architecture decisions into individual ADR files under `docs/adr/`; `docs/DECISIONS.md` is now a compatibility pointer.
- Added D-010 requiring on-demand AI context loading to reduce repeated token cost.
- Minimized Gemini, Claude and Copilot repository instructions so they defer to `AGENTS.md` without preloading additional files.

## 2026-10-06

- Created the initial governance-first repository structure.
- Added AI-agent handoff documentation.
- Added draft data policy, data-source registry, schema and roadmap.
- Expanded the data-source registry to 23 records after separating DATA.GOV.HK from the Combined DoJ Glossaries.
- Completed usable first-pass reviews for 20 of 23 source records.
- Added machine-readable `data-registry/sources.yaml` ingestion policy.
- Added source manifest schema and licence packaging model.
- Added SQLite schema v0.1 with provenance and licence-pack metadata.
- Implemented manifest-enforced Phase 0.5 SQLite PoC builder.
- Added non-authoritative CN/HK/TW test fixtures and automated tests.
- Added GitHub Actions CI; PoC unit tests and demo database build passed.
- Added Phase 1A real importer for the LSHK Jyutping Table with pinned revision, SHA-256 provenance, strict TSV validation, multi-reading support and CI tests.
- Added pronunciation schema migration and documentation.
- Added Phase 1B real OpenCC phrase importer pinned to commit `3ac34aa439a9908dd49fa92b5174b46314787ac2`.
- Added staged `STPhrases`, `HKPhrases` and `TWPhrases` support without flattening CN/HK/TW conversion semantics.
- Preserved OpenCC multi-candidate mappings, identity mappings, dictionary/line provenance and per-file SHA-256 source versions.
- Added OpenCC parser/importer tests covering HK/TW proper-name and terminology examples; CI passed.
- Added Phase 1C Wikidata EntityData importer for CC0 structured entity labels and aliases.
- Added stable QID identity, explicit entity types, locale-specific CN/HK/TW names, aliases, revision-aware provenance, retrieval timestamps and canonical per-entity SHA-256.
- Added a hard no-fabrication rule: generic Chinese labels are not silently promoted to missing regional labels.
- Added offline Wikidata fixtures and tests for a person and a film; CI passed after final provenance updates.
- Added Phase 2A deterministic localization/reference engine in `src/localizer_engine.py` and CLI in `scripts/localize_text.py`.
- Added entity-first longest-match resolution, entity-span protection, staged OpenCC-compatible term processing and no-guess conflict handling.
- Added structured provenance/explanation output for entity and term-rule decisions.
- Added `data/fixtures/evaluation_cases.json`, one-command fixture DB builder and integrated engine tests.
- Verified person CN→HK/TW localization, Taiwan terminology conversion, script-stage conversion, mixed entity+term conversion, entity protection, longest-match precedence, and ambiguous entity/rule review behavior in CI.
- Added `docs/LOCALIZER_ENGINE.md` and decision D-009 defining deterministic localization precedence and no-guess policy.
- Added Phase 2B OpenCC character/variant coverage: `STCharacters.txt`, `HKVariantsPhrases.txt`, `HKVariants.txt`, `TWVariantsPhrases.txt` and `TWVariants.txt`.
- Added dictionary-level base priorities reflecting reviewed OpenCC short-circuit order, separate from candidate rank.
- Expanded OpenCC fixtures/importer tests from 3 to 8 dictionaries.
- Added regression coverage for `见→見`, `檯→枱`, `爲→為` and Taiwan phrase-exception protection of `張棟樑` from the `樑→梁` character rule.
- Updated OpenCC documentation to state current forward coverage and remaining generated/reverse/runtime limitations; all Phase 2B tests passed CI.
