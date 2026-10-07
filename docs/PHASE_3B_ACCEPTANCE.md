# Phase 3B acceptance

Phase 3B is complete when all of the following are true:

- Rust reads the existing separate `user_dictionary.sqlite` v0.1 database directly.
- Protected terms outrank shared entity and term/script conversion.
- Fixed overrides outrank shared entity and term/script conversion.
- Protected terms outrank overrides for the same surface.
- Locale scopes, specificity, priority, longest-surface matching and enabled state match the Python reference.
- Equal-ranked conflicting overrides are preserved unchanged and marked for review.
- User events expose provenance, rule IDs and character spans.
- Shared events after a user-controlled segment are offset to global input/output spans.
- The CLI accepts `--user-db` without changing shared-only behavior when it is omitted.
- Existing Phase 3A corpus parity remains green.
- `cargo fmt --check`, strict Clippy and Cargo tests pass in CI.
