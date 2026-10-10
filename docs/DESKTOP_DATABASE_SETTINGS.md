# Desktop Database Settings

Phase 3E introduced validated database selection. Phase 3J adds safe persistence
of those choices and the private dictionary's enabled/disabled state across restarts.

## Security boundary

The frontend does not receive unrestricted filesystem access and does not query SQLite directly.

Database selection happens inside Rust through Tauri's native dialog plugin. Rust
validates the selected file before accepting it, stores full paths only in managed
state and a local non-secret settings file, and returns only filename/status
information to JavaScript. The settings document is never sent to the frontend.

## Shared database

A selected shared database must:

- exist as a local file;
- open successfully as SQLite in read-only mode;
- pass SQLite `quick_check`;
- contain the tables and columns used by Runtime API v1: `concepts`,
  `localized_names`, `term_rules`, `source_versions`, `name_evidence`, and `external_ids`.

If validation fails, the previous accepted shared database remains active.

## User dictionary

The private dictionary is optional and remains a separate SQLite file.

A selected user dictionary must:

- exist as a local file;
- open successfully as SQLite in read-only mode;
- pass SQLite `quick_check`;
- contain the `user_terms` table with the columns used by the current user-local runtime.

The user can disable the private dictionary without deleting its file or forgetting
the accepted selection. The **Enable** button validates that selection again before
reactivating it; choosing a dictionary also enables it. The existing
`clear_user_database` command now persists the disabled state. When disabled,
localization uses the shared-only runtime even if a dictionary is still selected.

## Persistence and restart behavior

Accepted settings take effect immediately for subsequent localization calls.

Rust resolves `settings.json` under Tauri's per-user `app_config_dir()` for the
existing application identifier `org.chineseregionallocalizer.desktop`. On Windows
this is normally under `%APPDATA%` followed by that identifier. This is independent
of the working product name; no repository or application-identifier rename is made.

Phase 3J's v1 JSON document contained only `schema_version: 1`, `shared_db`, optional
`user_db`, and `user_enabled`. Issue #60 adds settings v2 with a separate
`presentation` object (`ui_locale`, `appearance`). Valid v1 documents migrate in
memory with zh-HK/System defaults; startup never rewrites them. Only an accepted
settings change writes v2. Presentation does not enter localization requests or
change database choices. See [UI foundation](DESKTOP_UI_FOUNDATION.md).
Paths are absolute and remain private to Rust-side
storage. Local paths are non-secret preferences, but the file is not encrypted;
it must never hold keys, tokens, text input, or release/update trust settings.

At startup:

- `CRL_SHARED_DB` and `CRL_USER_DB` remain development fallbacks; relative environment
  paths are resolved against the launch directory. Without a shared override, the
  fixture database under `build/regional-demo.sqlite` remains the default.
- Valid saved choices take precedence over those defaults. Each database is checked
  independently in read-only mode before restoration. The settings file is bounded
  to 64 KiB and is never rewritten merely by starting the app.
- Missing, corrupt, relative, or incompatible stored database choices retain the
  usable startup default for that field. An invalid startup user dictionary is
  omitted so shared-only localization remains available. If no shared database is
  usable, status reports it as unavailable rather than creating one.
- A saved disabled state takes precedence over an enabled environment dictionary.
  An invalid disabled selection is discarded but remains disabled.
- Malformed/truncated/incomplete settings fall back safely with a path-free warning.
  A deliberate validated chooser/toggle action can replace a corrupt document.
- Unsupported schema versions or unknown fields fall back safely and block writes
  for that session, preserving the original document for an explicit future
  migration. To use this build with that file, close the app and back up/remove it
  locally before restarting. v0 remains unsupported rather than guessed into v1/v2;
  settings versions newer than v2 and unknown fields (including presentation fields)
  remain write-protected for explicit future migration.

Changes are serialized under one Rust lock. Rust validates a candidate, writes a
same-directory temporary file, syncs its contents, and atomically replaces the old
document before publishing the new session configuration. Windows replacement uses
`tempfile`'s replace-existing implementation. If saving fails, the working session
configuration stays unchanged and errors contain no filesystem paths.

Successful authenticated updates persist the validated active shared-database path
through the same acceptance boundary. If saving then fails, the desktop keeps its
previous configuration; the package-store install may already have completed and
is not rolled back by the settings layer. The existing update trust pipeline is
unchanged.

Runtime API v1, user-term schemas, terminology precedence, usage contexts and other
Alpha features are unchanged. See ADR D-021 for the migration boundary.

## Frontend status contract

The settings UI receives only:

- shared database filename;
- shared database ready/error state;
- optional user-dictionary filename;
- user-dictionary ready/error state.
- user-dictionary enabled state;
- optional path-free settings restoration warning.

Full local paths remain inside Rust-managed state and storage.
