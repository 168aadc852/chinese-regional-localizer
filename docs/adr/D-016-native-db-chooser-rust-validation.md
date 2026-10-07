# D-016 — Native database chooser with Rust-side validation

Status: Accepted

## Context

The desktop application needs a beginner-friendly way to select a shared regional database and optional private user dictionary. Allowing frontend JavaScript to receive arbitrary filesystem access or directly open SQLite would weaken the thin-frontend boundary established in D-015.

## Decision

Database selection is owned by Rust.

- Native file dialogs are opened from narrow Tauri commands.
- Full selected paths remain in Rust-managed application state.
- Rust validates SQLite compatibility before replacing the active configuration.
- Frontend JavaScript receives only filename/status/error information.
- Localization reads the current accepted Rust-managed configuration.
- Shared and private databases remain separate files.

Phase 3E settings are session-scoped. Persistence, packaged database delivery, update checks and rollback are separate decisions.

## Consequences

This keeps filesystem and SQLite authority out of the frontend while still giving non-technical users a normal file-picker workflow. Future persistence must preserve the same boundary and must not turn the frontend into an arbitrary path or database interface.