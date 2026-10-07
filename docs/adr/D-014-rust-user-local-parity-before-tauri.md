# D-014 — Rust user-local parity before Tauri

Status: Accepted

Date: 2026-10-07

## Decision

The Rust runtime must support the existing private user-local dictionary contract before Tauri UI integration begins.

Rust reads the existing separate `user_dictionary.sqlite` database rather than merging private preferences into the shared regional database. The runtime preserves the established precedence and no-guess behavior: protected terms > user overrides > shared entity localization > regional terminology > generic/script conversion.

The Rust user-local layer must also expose rule provenance and character spans sufficient for UI review/highlighting.

## Rationale

The user dictionary is part of the product behavior, not an optional UI-only feature. Starting Tauri before this behavior is available in Rust would force duplicated localization logic between Python and the application layer and would make later cross-platform packaging harder.

Keeping the user database separate preserves privacy, simplifies shared data releases and avoids contaminating upstream/licensed data packs with personal preferences.

## Consequences

- Tauri commands should call the Rust runtime rather than reimplementing user-rule precedence.
- Shared-only localization remains available when no user DB is supplied.
- Phase 3B remains read-only for the user database; editing APIs may be added later.
- Any future optimization must preserve the same parity tests and no-guess behavior.