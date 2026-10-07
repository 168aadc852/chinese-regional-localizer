# Phase 3E — Desktop Database Settings / Chooser

## Goal

Add a safe desktop settings layer for choosing the shared regional database and optional private user dictionary without exposing arbitrary filesystem access or SQLite access to frontend JavaScript.

## Security boundary

- Frontend JavaScript never receives unrestricted filesystem APIs.
- Frontend JavaScript never opens SQLite directly.
- File selection is initiated through narrow Tauri commands.
- Rust validates selected files before accepting them into application state.
- The shared database is required; the user dictionary is optional and remains a separate file.
- Invalid or incompatible database files must be rejected with a user-readable error.

## Initial acceptance criteria

- Rust command returns current database configuration/status.
- Rust command can validate and set a shared database path selected by the desktop user.
- Rust command can validate and set or clear an optional user dictionary path.
- Localization uses the current Rust-managed configuration after a successful change.
- Frontend contains a simple Settings area showing configured database status and buttons for selection/clear actions.
- Tests cover valid/invalid shared DB, valid/invalid user DB, clearing user DB, and configuration persistence for the current app session.
- No direct SQLite or arbitrary path handling is added to frontend code.
