# D-021 — Versioned desktop preferences remain outside the localization runtime

Date: 2026-10-08

Phase 3J / Issue #38 persists validated shared/user database selections and the
private dictionary enabled state in `settings.json` under Tauri's per-user app
configuration directory. The file is handled only by Rust; frontend commands take
no paths and receive only filenames, readiness, enabled state, and path-free messages.

The settings schema starts at v1 and contains only non-secret database preferences.
Startup uses bounded parsing and independent read-only compatibility checks, with
environment/demo fallbacks. Version dispatch is explicit. Unsupported versions or
fields are preserved and cannot be overwritten by this build. There is no prior
persisted schema to infer or migrate; future migrations require explicit code and
fixtures. Corrupt files can be replaced by a deliberate validated settings change.

Candidate configuration is published only after a same-directory temporary write,
file sync and atomic replace succeed, under the configuration lock. Disabled user
dictionaries retain their accepted selection but are excluded from runtime calls.
Re-enabling requires validation. Authenticated updates use the same database-path
acceptance boundary; package installation and desktop preference saving remain
separate operations, and a settings-save failure does not undo a package install.

The store does not hold text, secrets, update trust roots, terminology rules,
context profiles, or My Terms preferences. It neither changes Runtime API v1 nor
defines a future context model. Future Alpha contexts and user-term preferences
need separately approved schemas/issues and can evolve without coupling their
model to this small desktop settings document. Keep the existing app identifier
stable unless a later issue defines an explicit settings-location migration.
