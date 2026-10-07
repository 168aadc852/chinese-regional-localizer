# Phase 3E acceptance

Phase 3E is complete when the Tauri desktop shell can safely inspect and change its Rust-managed shared database and optional user dictionary configuration, rejects invalid database files, updates runtime localization after successful changes, and keeps filesystem/SQLite access behind narrow Rust commands rather than exposing it to frontend JavaScript.
