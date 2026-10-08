# D-022 — Independent usage-context profile foundation

Date: 2026-10-09. Scope: approved Issue #46 / Implementation Brief v0.1.

Keep context definitions in a standalone shared-Rust module and versioned local JSON
document, separate from desktop settings v1, SQLite terminology data and future
My Terms storage. Use one document-level `schema_version` to avoid mixed per-record
versions. v1 is the first format; explicit dispatch rejects unsupported versions
without guessing migrations or overwriting the original document.

Ship seven stable reserved built-in IDs. General is the root; other built-ins inherit
General. Custom construction defaults to General, with explicit single-parent/root
overrides. Built-in identity and links are protected; display names/enabled flags may
change. Validate the whole snapshot before accepting edits/imports, including disabled
profiles, duplicate IDs, missing parents and cycles. Resolve specific-to-general;
disabled profiles/ancestors make a chain unavailable, never silently skipped.

Use bounded input and deterministic serialization plus a synced same-directory atomic
file replacement, reusing `tempfile` (now a normal Rust dependency). Bad existing files
block writes. The opt-in store does not choose a desktop directory or provide a
multi-process writer protocol; callers serialize edits. Future desktop integration
needs its own UI/storage-location scope.

Runtime API v1, localization selection, desktop settings, private dictionary schemas
and licence/ingestion gates remain unchanged. Context profiles are structure, not
approved terminology coverage. Issue #47 must separately reconcile inheritance with
term-ranking/protected-term behavior. No My Terms migration or later Alpha feature is
implied. See [Usage Context Profiles](../CONTEXT_PROFILES.md) for the implemented contract.
