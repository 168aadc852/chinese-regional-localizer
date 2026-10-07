# Desktop Database Settings

Phase 3E adds a safe desktop settings layer for selecting the shared regional database and an optional private user dictionary.

## Security boundary

The frontend does not receive unrestricted filesystem access and does not query SQLite directly.

Database selection happens inside Rust through Tauri's native dialog plugin. Rust validates the selected file before accepting it, stores the full path only in managed application state, and returns only filename/status information to JavaScript.

## Shared database

A selected shared database must:

- exist as a local file;
- open successfully as SQLite in read-only mode;
- contain the required runtime tables: `concepts`, `localized_names`, `term_rules`, and `source_versions`.

If validation fails, the previous accepted shared database remains active.

## User dictionary

The private dictionary is optional and remains a separate SQLite file.

A selected user dictionary must:

- exist as a local file;
- open successfully as SQLite in read-only mode;
- contain the `user_terms` table.

The user can remove the active private dictionary from the session without deleting the underlying file.

## Session behavior

Accepted settings take effect immediately for subsequent localization calls.

Environment variables `CRL_SHARED_DB` and `CRL_USER_DB` remain development startup fallbacks.

Phase 3E does not persist chooser selections across app restarts. Persistent preferences, packaged database delivery, updates and rollback are separate future work.

## Frontend status contract

The settings UI receives only:

- shared database filename;
- shared database ready/error state;
- optional user-dictionary filename;
- user-dictionary ready/error state.

Full local paths remain inside Rust-managed state.