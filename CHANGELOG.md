# Changelog

## 2026-10-10 — Issue #60 Desktop Alpha Stage A

- Added a human-editable UTF-8 CSV translation table and deterministic, offline validation/generation for four independent UI locales: zh-HK, zh-TW, zh-CN and en.
- Migrated the existing static shell to semantic keys, live UI language switching and self-labelled language options without flags; document source/target choices remain independent.
- Added semantic Light/Dark/E-ink / Mono tokens, platform Light/Dark resolution for System, visible keyboard focus, reduced-motion guards, non-colour-only status patterns and responsive zoom-friendly layout.
- Added separate typed presentation preferences to safely migrated desktop settings v2; preserved atomic-save, path privacy, concurrency and unsupported-document protection.
- Added translation release/build gates, zero-dependency Node presentation tests and translation/contrast/settings regressions. No localization engine, Runtime API v1, route, review-session or private-store semantics changed.
- This is a UI foundation, not the full #50 workflow, dictionary manager, new localization modes, installer or complete WCAG certification.

## 2026-10-10 — Issue #49 context-aware My Terms / My Dictionaries

- Added separate private SQLite v2 dictionary containers, reserved Personal/Legacy identities and target/context-scoped preferred terms without new numeric priority controls.
- Added atomic/idempotent v0.1 migration preserving term IDs, every legacy field, disabled state and null scopes; injected failure rolls back schema/data/version and leaves the old store usable.
- Filtered enabled dictionary/term, locale and validated context before longest matching; exact/nearest ancestor/all-context preferences precede unchanged shared processing. Same targets deduplicate; equal-level conflicts preserve original text.
- Added request-bound review remembering to Personal using original input spelling, current source/target locale and exact/all-context scope; stale/foreign candidates cannot write. Failed writes explicitly report nothing remembered while keeping valid one-time text/undo.
- Added optional fail-closed `choice.rememberable: false` for partial source expansions; existing occurrence edits/spans/undo remain valid and Runtime API v1's nine outer fields stay unchanged.
- Added reserved-safe dictionary activation/management, preferred-term edits and transactional CSV preview/import foundation; no third-party downloads or polished #50 UI.
- Added synthetic selection, migration/recovery, persistence/reopen, insertion-order, CSV and live Python/Rust parity regressions. Shared schema, supported routes, desktop settings and update/licence boundaries remain unchanged.

## 2026-10-10 — Issue #48 alternative terms and one-time occurrence review

- Added grouped deterministic regional candidates with Recommended, Also valid and Needs your decision states, retaining #47's longest eligible phrase, context ranking and no-guess decisions.
- Added optional nested `choice` metadata with session-local IDs, exact Unicode source/stage/output spans, frozen candidates and current-text anchors; Runtime API v1 still has the same nine outer response fields.
- Carried traversal/alignment positions through Rust stages and composed user-layer segment offsets explicitly, so repeated terms/replacements are independently tracked without substring guessing.
- Added in-memory Rust review sessions and matching Python reference behavior for validated this-time-only edits, later-span shifts, monotonic revisions and atomic stale/invalid-choice rejection.
- Added reversible LIFO undo, including adjacent/length-changing edits, repeated edits to one occurrence and choices with no visible text change.
- Added stable unsupported remember-intent hooks without writing preferences or changing dictionaries.
- Added 25 synthetic #48 scenarios alongside the existing 50 context cases, live candidate/ID/span/edit/undo parity, reversed-insertion golden checks and stale-state regressions.
- Preserved entity/user protection, script neutrality, locale/routes, schemas, desktop settings, data/licence gates and update/signing behavior. #49 persistence and #50 desktop choice UX remain separate work.

## 2026-10-09 — Issue #47 deterministic context-aware terminology selection

- Added optional `context.usage_context_id` using a validated immutable #46 profile snapshot, with built-in defaults and clear errors for unknown/disabled chains.
- Extended shared regional terminology ranking: longest eligible phrase, exact context, inherited parents, General only when inherited, then unscoped fallback; numeric priority applies within the winning level.
- Kept equal-level conflicting winners unresolved with protected original stage text and review-needed; retained actual winning rule provenance and optional context-selection explanations.
- Reused existing rule JSON constraints without schema migrations; legacy `domain`, spatial constraints, locale isolation, entities and private user protections remain intact.
- Preserved context-neutral script conversion and existing staged routes. Requests without a usage-context ID keep legacy behavior.
- Added a Python reference profile adapter, 50 project-authored shared cases, live Python/Rust parity and malformed-condition/API compatibility regressions.
- Kept Runtime API v1's outer contract, desktop/settings code, #46 storage, private dictionaries, ingestion/licence gates and later #48–#51 work unchanged.

## 2026-10-09 — Issue #46 usage-context profile foundation

- Added a standalone shared-Rust context-profile model with seven stable built-in IDs and human-readable names.
- Added custom profiles defaulting to General, single-parent chain resolution in specific-to-general order, and explicit disabled-ancestor unavailability.
- Rejected duplicate/reserved IDs, self-parenting, cycles, missing parents, invalid fields and incomplete documents before accepting a snapshot.
- Added an independent bounded JSON v1 store with explicit version dispatch, deterministic v1 round trips and synced atomic replacement; invalid/unsupported existing files are preserved.
- Added focused model/storage regressions and documented the approved Issue #46 boundary.
- Left Runtime API v1, localization selection, desktop settings v1, private dictionary schemas and UI unchanged. Context-aware terminology, My Terms and later Alpha features remain separate issues.

## 2026-10-08 — Phase 3J desktop settings persistence

- Added Rust-only versioned non-secret settings under Tauri's per-user configuration directory.
- Persisted validated shared/private database choices and private dictionary enabled state; added re-enable control without forgetting the selection.
- Added bounded startup parsing, independent read-only compatibility/integrity validation and safe environment/demo fallbacks.
- Preserved unsupported versions/fields for explicit future migration rather than silently downgrading or overwriting them.
- Added synced temporary writes and atomic replacement before changing the working session; failed saves leave it unchanged.
- Persisted authenticated update database activation through the same acceptance boundary, without changing the update trust pipeline.
- Added restart, corruption, compatibility, future-version, failed-save, concurrency, path-privacy and Runtime API v1 toggle regressions.
- Kept Runtime API v1, localization precedence, user-term schema and future Alpha contexts/My Terms outside this change.

## 2026-10-07 — Phase 3I desktop authenticated data-update UI

- Added narrow Tauri commands for update status, explicit authenticated catalog checks and explicit package installation.
- Kept update origin, trusted-key file, package-store path and package identity in Rust-side startup configuration only; frontend JavaScript cannot submit arbitrary URLs or trust roots.
- Reused the Phase 3H authenticated catalog, signature, sequence rollback, signed manifest binding, bounded download, hash/size, SQLite/package validation and immutable PackageStore activation pipeline.
- Added structured frontend update status containing only configured/current/offered/availability/message fields, without exposing full local paths or generic network access.
- Added a Settings UI section for **Check for updates** and **Install update**.
- Made successful installs switch the desktop runtime to the newly activated shared database only after full package validation.
- Kept the existing active database unchanged when update discovery or installation fails.
- Added a gated Rust `test-support` feature used only by desktop tests to construct authenticated catalog fixtures; the production binary does not expose the test constructor.
- Added `docs/DESKTOP_DATA_UPDATES.md` and ADR D-020 documenting the Rust-only update trust boundary.
- Background scheduling, retry/resume, production hosting/CDN layout, application-binary updates and remote trust-root rotation remain out of scope.

## 2026-10-07 — Phase 3H authenticated network update discovery/download

- Added a Rust-side update transport boundary using HTTPS with redirects disabled and one configured release origin.
- Added bounded catalog, detached-signature, manifest and database downloads, including streamed byte limits independent of `Content-Length`.
- Added authenticated release discovery that verifies the existing pinned-Ed25519 catalog before trusting package identities.
- Persisted the highest trusted catalog sequence atomically in the local package store so restart cannot reset rollback protection.
- Added package-manifest signature verification plus exact signed-catalog hash/identity binding before database download.
- Added staged database download followed by signed size/SHA-256, SQLite/package validation and existing immutable `PackageStore` installation.
- Kept the active shared database unchanged on network, signature, binding, hash, SQLite or package-validation failure.
- Added a transport abstraction and fake-transport tests so network-update CI requires no public Internet.
- Added `docs/NETWORK_UPDATES.md` and ADR D-019 documenting the Rust-only authenticated network boundary.
- Background scheduling, update UI, arbitrary frontend networking, app-binary updates, remote trust-root rotation and production hosting remain out of scope.

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

- Added the standalone Rust reference crate under `rust/`.
- Added direct SQLite v0.2 reads for the current forward CN/HK/TW localization routes.
- Ported conservative entity matching, current-version filtering, staged longest-match term rules, context constraints and no-guess ambiguity behavior.
- Added a small Rust CLI for fixture/demo use.
- Added Rust parity coverage against the same short evaluation corpus and Phase 2E realistic corpus used by Python.
- Added ambiguity and short/common-entity false-positive Rust regressions.
- Added strict Rust format, Clippy and Cargo tests to CI while keeping all Python checks green.
- Added `docs/RUST_REFERENCE_ENGINE.md` and ADR D-013 requiring Python/Rust behavioral parity before optimization or UI work.

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
- Replaced the monolithic project-history behavior of `PROJECT_STATE.md` with a short current-state file plus `docs/history/PROJECT_HISTORY.md`.
- Replaced monolithic architecture-decision loading with a compact ADR index plus one file per decision under `docs/adr/`.
- Reduced Gemini, Claude and Copilot entry files to pointers to the canonical `AGENTS.md` router.
- Recorded D-010: load AI context on demand instead of preloading the documentation tree.

## 2026-10-07 — Phase 2B OpenCC character / regional variant coverage

- Expanded the reviewed OpenCC forward mapping from three phrase dictionaries to eight source dictionaries.
- Added `STCharacters.txt` for Simplified → Traditional character fallback.
- Added `HKVariantsPhrases.txt` / `HKVariants.txt` for Hong Kong phrase/character normalization.
- Added `TWVariantsPhrases.txt` / `TWVariants.txt` for Taiwan phrase/character normalization.
- Added deterministic dictionary-level priority bands that preserve phrase-exception-before-character-fallback behavior without flattening stages.
- Added regression coverage for `见 → 見`, `檯 → 枱`, `爲 → 為`, and the Taiwan `張棟樑` phrase exception.
- Updated importer documentation and project state while continuing to state that full OpenCC runtime parity is not yet claimed.
