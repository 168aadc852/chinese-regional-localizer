# Architecture Decision Index

Read only decisions relevant to the current task.

- [D-001](D-001-github-source-of-truth.md) — GitHub is the project source of truth.
- [D-002](D-002-data-governance-first.md) — Data governance precedes application development.
- [D-003](D-003-offline-first.md) — Offline-first product direction.
- [D-004](D-004-regional-targets.md) — `zh-CN`, `zh-HK`, `zh-TW` are first-class targets.
- [D-005](D-005-separate-code-data-licences.md) — Keep code and data licences separate.
- [D-006](D-006-ai-writeback.md) — AI agents write material context back to the repo.
- [D-007](D-007-cross-platform-direction.md) — Tauri/Rust/SQLite is the leading, not fully locked, direction.
- [D-008](D-008-source-manifest-gate.md) — Manifest + exact resource IDs control automated ingestion and pack boundaries.
- [D-009](D-009-localization-precedence.md) — Deterministic shared localization precedence, conservative entity matching and no-guess policy.
- [D-010](D-010-on-demand-ai-context.md) — AI context is loaded on demand to reduce repeated token cost.
- [D-011](D-011-update-safe-source-versions.md) — Resource refreshes are idempotent, versioned and current/superseded aware.
- [D-012](D-012-user-local-precedence.md) — Private user rules stay separate and outrank shared entity/term rules.
- [D-013](D-013-rust-parity-before-ui.md) — Rust must match the Python behavioral contract before optimization or Tauri UI integration.
- [D-014](D-014-runtime-api-boundary.md) — The versioned Rust Runtime API is the application/Tauri boundary.
- [D-015](D-015-tauri-thin-frontend-managed-db-state.md) — Tauri keeps the frontend thin and database paths in managed Rust state.
- [D-016](D-016-native-db-chooser-rust-validation.md) — Native database selection and compatibility validation stay inside Rust; frontend receives only safe status metadata.
- [D-017](D-017-versioned-data-packages-rollback.md) — Shared data releases use immutable versioned packages, staged validation/activation and rollback instead of in-place replacement.
- [D-018](D-018-pinned-ed25519-release-authenticity.md) — Release metadata must be authenticated by pinned Ed25519 public keys, with domain separation, expiry and rollback protection before network updates.
