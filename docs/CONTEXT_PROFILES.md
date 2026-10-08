# Usage Context Profiles — Issue #46 foundation

Date: 2026-10-09.

A usage context is a named folder for a writing situation, such as General or
Technology / Software. This foundation identifies folders and their parent links;
it does **not** yet change wording, choose terms or add desktop context controls.
Context-aware terminology selection belongs to separately approved Issue #47.

The implementation follows the [approved design](https://github.com/168aadc852/chinese-regional-localizer/issues/46#issuecomment-6063752375)
and [Implementation Brief v0.1](https://github.com/168aadc852/chinese-regional-localizer/issues/46#issuecomment-6063785354).

## Model and built-ins

`rust/src/context_profiles.rs` is a standalone module in the shared Rust library.
Desktop, a future CLI and a future MCP client can reuse this model rather than
implement different inheritance logic. Runtime API v1 is unchanged and does not
call this module. Python localization and existing private dictionaries are unchanged.

| Stable ID | Default display name | Parent |
| --- | --- | --- |
| `general` | General | None |
| `technology-software` | Technology / Software | `general` |
| `banking-finance` | Banking / Finance | `general` |
| `business-marketing` | Business / Marketing | `general` |
| `legal` | Legal | `general` |
| `education` | Education | `general` |
| `government-public-administration` | Government / Public Administration | `general` |

A profile has `context_id`, `display_name`, optional `parent_context_id`, `built_in`
and `enabled`. `schema_version` lives once at the document level, not on every row,
so one snapshot cannot contain contradictory format versions. All seven built-ins
must be present. Their IDs, built-in flags and parent links are protected; their
display names may change without changing identity, and they may be disabled.
Custom profiles cannot replace a reserved ID or claim to be a built-in.

IDs are 1–64 ASCII bytes, start with a lowercase letter and contain lowercase
letters/digits separated by single hyphens. Display names are nonempty, trimmed,
at most 128 Unicode characters, with no control characters. These bounds are format
validation, not terminology/licence approval.

## Creation, edits and inheritance

`ContextProfile::custom(id, name)` creates an enabled profile inheriting `general`.
A caller may explicitly choose another parent or `None` before validation.
Imports preserve explicit links; an omitted optional parent defaults to General,
while explicit `null` means no parent. A declared missing parent is never replaced
with General. The built-in General root must explicitly have no parent. No context
is automatically guessed from text.

`ContextProfiles::with_custom_profiles` combines custom profiles with the defaults.
`ContextProfiles::from_profiles` validates a full snapshot, including edits/imports.
Snapshots expose read-only views; callers build and validate a replacement before
publishing an edit. A single invalid definition rejects the whole candidate, leaving
the caller's previous snapshot intact. Missing built-ins are not silently reinserted.

`resolve_chain(id)` returns the selected context first, then its parent, and so on:
`hi-fi-audio → audio → general`. Input/list order never changes this result.
Every profile is included once. Self-parenting, indirect cycles, duplicate IDs and
missing parents are rejected even for disabled profiles. There is no guessed fallback.

Disabled profiles remain stored and identifiable. Resolving a disabled profile, or
a child whose chain contains a disabled ancestor, returns `DisabledContext` and no
partial chain. It does not silently skip the disabled ancestor. Re-enabling requires
building a new validated snapshot. This is availability behavior, not a term-ranking
rule; reconciliation with terminology precedence belongs to Issue #47.

## Independent storage and version handling

The v1 JSON document contains `schema_version: 1` and a `profiles` array containing
all built-ins and custom profiles. Each profile uses the five fields listed above.
Unknown fields and duplicate JSON fields are rejected. No terms, locale preferences,
source datasets, priorities or raw dictionaries belong in this document.

`ContextProfileStore::new(directory)` uses the fixed filename `context-profiles.json`
in a caller-owned directory. This is an opt-in library store: there is no automatic
desktop startup integration or new application-config path in this issue. It never
reads/writes `settings.json` or SQLite dictionaries. A missing document returns
built-in defaults without creating a file or directory; other read/validation errors
are reported, not repaired. Loads are bounded to 1 MiB and 1,024 total profiles.

Saving first checks any existing document, serializes a validated snapshot in
canonical ID order, syncs a same-directory temporary file and atomically replaces
the old file using `tempfile`, including on Windows. Existing corrupt, unknown-field
or unsupported-version documents are not overwritten. Repair requires an explicit
caller/user decision outside this API. Failed staging/replacement is an error;
temporary files are cleaned up. Caller-owned edit serialization is required; this
foundation does not provide a multi-process writer protocol or a UI recovery flow.
File-content sync/atomic replacement is not a guarantee against every power-loss scenario.

`from_json` is the explicit version-dispatch/migration seam. v1 is the first persisted
context format: loading/resaving v1 is deterministic, with no semantic migration.
v0 and future versions return `UnsupportedVersion`; missing/non-integer versions,
malformed and incomplete data fail safely. Future migrations require a separately
approved source format, explicit transformation and validation tests; no fake legacy
format or private-dictionary migration is invented here.

## Scope and checks

These names do not promise terminology coverage for the Chinese Mainland, Hong Kong
or Taiwan. Existing source, ingestion and licence-pack gates remain in force. Profiles
contain no rules and cannot bypass those gates.

Focused checks: `cargo test --locked --manifest-path rust/Cargo.toml --test context_profiles`.
The regression suite covers built-ins, custom/default parents, deep chains, stable
ordering, invalid graphs/identity, disabled ancestors, malformed/versioned data,
deterministic round trips, storage preservation and separation from existing files.
Existing Python/Rust/Tauri suites still cover runtime behavior, data governance and
desktop settings. See [ADR D-022](adr/D-022-independent-context-profile-foundation.md).

Local Windows validation: 22 focused profile tests (including failed locked-file
replacement), 65 Rust tests in total, 54 Python tests and 25 Tauri tests passed.
Rust/Tauri formatting, Clippy with warnings denied, Tauri all-target compile check and Windows debug build,
demo/package/catalog smoke, corpus evaluation and Python/Rust benchmark smoke passed.
GitHub CI and protected-main review/merge remain the PR closeout gate.

No term-selection changes, multiple inheritance, My Terms migration, alternative-term
UI, Alpha redesign, LLM classification, packaging or repository/application rename
is part of this foundation.
