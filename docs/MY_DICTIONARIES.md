# My Dictionaries / My Terms — Issue #49 core foundation

Date: 2026-10-10. Approved core/private-storage work, implemented in the focused
#49 PR pending protected-main CI, review and merge. No #50 dictionary manager,
Desktop remember buttons, TXT import UI, cloud sync or terminology coverage claim.

Baseline: [Design Approval v0.1](https://github.com/168aadc852/chinese-regional-localizer/issues/49#issuecomment-6092942948)
and [Implementation Brief v0.1](https://github.com/168aadc852/chinese-regional-localizer/issues/49#issuecomment-6092948935).

## Three separate concepts

- Usage Context describes the writing context using the validated #46 profile snapshot.
- My Dictionaries are independent, enabled/disabled local private terminology sets.
- My Terms are protected/preferred records inside those dictionaries, not a second engine.

The reserved `personal` dictionary (display name Personal) stores remembered choices.
The reserved `legacy` dictionary (Legacy / Default) contains migrated records and
the existing management CLI's compatibility operations. Both are system-managed;
service operations prohibit their rename/deletion. They can be independently
enabled/disabled. Saving a preference does not silently re-enable a disabled dictionary.
User dictionary identity is stable and separate from its editable display name.

## SQLite v2 and migration

The private store uses `user_metadata.schema_version = '2'`,
[user-dictionary-v2.sql](../schema/user-dictionary-v2.sql), and
[the v0.1 migration](../schema/private-migrations/0001-v2.sql).
The historical v0.1 schema and shared SQLite v0.2 schema remain unchanged.

`user_dictionaries` stores stable ID, name, enabled flag, reserved marker and optional note.
`user_terms` preserves the old eleven fields/term IDs, adding dictionary ID, optional
`usage_context_id` and a legacy-compatibility marker. `kind` remains `protected` or
`override` (preferred term). New preferred records have an explicit target locale;
null context means all contexts **within that target locale**. New APIs have no priority
input. Historical priority is retained only to reproduce legacy semantics.

Explicit private-store open creates a fresh v2 or upgrades v0.1 inside one SQLite
IMMEDIATE transaction. The migration copies named original fields into the new
private table, preserving IDs, notes, timestamps, priorities, enabled flags and null
locale scopes; it does not invent contexts/locales. It assigns `legacy` and null
context, verifies row counts, integrity, foreign keys and reserved dictionaries,
then commits. Failure rolls back DDL/data/version together, leaving the old DB usable.
Migration/open is idempotent. Unsupported future versions/corrupt databases fail clearly
without overwriting them. Noncanonical legacy tables/views/term triggers are refused
rather than silently discarded. No shared-store migration is involved.
Early unversioned stores are accepted only when their eleven `user_terms` columns
exactly match the historical layout; version metadata is created inside the same
transaction. Missing metadata on an unfamiliar/newer layout is not assumed to be v0.1.

Rust `UserControlledLocalizer::open` and Python `UserDictionary.open` perform this
upgrade. The Python CLI's historical `--schema` argument remains accepted for caller
compatibility but is not re-executed on a v2 store. In-memory reference/test adapters
can still read historical v0.1 connections without upgrading them.

## Eligibility and deterministic precedence

Filter dictionary enabled, term enabled, source/target locale compatibility and
validated Usage Context chain **before** choosing the longest private surface.
For that surface protected terms still win. Preferred terms rank:

1. Exact selected context.
2. Nearest inherited ancestor, then further enabled ancestors (General only if inherited).
3. All-context scope.
4. Legacy locale-specificity/priority only when all records at the winning level are legacy.

One distinct preferred target at the winning level is applied and protected from
the shared pipeline. Duplicate same-target records do not create ambiguity. Different
targets at that level preserve original text and require review. Dictionary order,
row ID, insertion time and alphabetical order never choose wording. If a new
preference competes at the same level with legacy records, legacy numeric priority
does not silently break that conflict. With no private eligible match, #47 shared
entity/script/regional behavior is unchanged; stages/routes are not reordered.

New private preferred events add #48-compatible `choice` metadata with deduplicated
targets, exact character spans and frozen candidates. Optional `private_selection`
reports exact/ancestor/all-context and stable dictionary IDs; no path or numeric
priority is added. Existing protected/legacy events retain their established contract.

## Request-bound remembering and failure semantics

Rust `Runtime::review(&request)` and Python `UserControlledLocalizer.review(...)`
create an owned `PrivateReviewSession`. It keeps the original input, source/target
locale and validated current context internally. Edits send only revision,
occurrence ID, frozen candidate ID and intent, plus a trusted core store handle.
Never construct trusted review state from frontend-provided responses/candidates.

- Use this time only: the #48 in-memory edit; no private-store operation.
- Remember for this context: requires an explicit valid current context; saves to
  `personal` with that context and the current target locale.
- Remember for all contexts: saves to `personal` with null context and current target locale.

The saved source is the exact original-input interval, **not** the possibly
script-normalized regional-stage spelling. The source locale is also preserved,
so a choice for CN `测试词` can replay before shared conversion on future CN requests.
Normal script expansion/contraction remains supported when the selected stage phrase
covers complete original-source intervals. If it is only part of an expansion,
the core adds `choice.rememberable: false`; remembering rejects without mutation,
while occurrence-specific one-time editing/undo remains valid. This prevents saving
a partial output as the replacement for an entire original character. The optional
flag is omitted when true and travels with the frozen core occurrence.

The wrapper delegates revision, membership, anchors, bounds, overlap, span shifting
and undo to #48 without weakening those checks. One-time review remains standalone;
its original remember hooks are unsupported without this request-bound wrapper.
The old nine-field Runtime API v1 response remains unchanged. No new Tauri command.

A remembered upsert is a separate atomic SQLite transaction. Same Personal
dictionary/source/source-locale/target/context scope + same output is idempotent;
a newer explicit choice updates that scope in place. It never overwrites another
dictionary. If the text edit succeeds but storage fails, the wrapper returns an
explicit persistence error: **text choice applied, nothing remembered**. The current
snapshot/undo remains valid and the private DB is unchanged. Undo affects review
text only; it does not erase a previously committed remembered preference.

## Core operations and CSV foundation

Rust `private_store::PrivateStore` and Python reference `PrivateStore` expose create,
rename, delete and independent activation of dictionaries; validated preferred-term
upsert/edit-by-ID, term enable/disable and deletion. Edits preserve term IDs and
fail atomically on scope collisions. Protected rows remain readable/manageable through
the existing protected-term compatibility API; this is not a polished manager.

`preview_csv(text, target_default)` accepts UTF-8 text already read by the caller:

```csv
source_text,replacement
軟件,軟體
```

An explicit target default is required for rows without `target_locale`; no locale
is guessed. Optional columns: `target_locale`, `usage_context_id`, `note`.
Null/empty usage means all contexts. Standard quoted fields, escaped quotes,
embedded newlines and CRLF are supported. Extra/duplicate/missing headers and malformed
or invalid rows return a row/reason preview; no DB writes occur in preview.
`commit_csv(dictionary_id, preview)` refuses previews with errors, revalidates
every term and writes all rows in one transaction. A mutated preview cannot bypass
validation or partially commit. Equivalent scopes explicitly update that dictionary's
preferred term. Source-locale scope is available through term operations, not this CSV.
Different targets for a duplicate scope **within one CSV** are validation errors,
not a silent last-row-wins decision; identical duplicates are idempotent.

Import only content you have the right to use. Content stays local; it is neither
uploaded nor promoted into official packs. No third-party downloads/content shipped.
Export/backup is a future core hook for deterministic records or a consistent SQLite
backup; no export format/backup UX is claimed here. #50 owns that UI separately.

## Validation

Project-authored [private preference fixtures](../data/fixtures/private_preferences_cases.json)
cover 28 selection cases in both dictionary/term insertion orders. Live Rust/Python
parity compares complete owned snapshots plus this-time/context/all-context persistence,
restart replay and failed-write/undo traces. Additional tests cover old-field migration,
injected rollback/recovery, future/corrupt stores, reserved operations, explicit edits,
CSV atomicity, stale/foreign candidates, source expansions and Runtime v1 outer fields.

```text
python -m unittest discover -s tests -p test_private_preferences.py -v
cargo test --locked --manifest-path rust/Cargo.toml --test private_preferences
```

Full #46–#48, legacy user, governance, runtime, Tauri and Windows build regressions
remain required. Python is reference/parity only; Desktop/CLI/MCP reuse the same Rust core.
See [ADR D-025](adr/D-025-versioned-private-context-preferences.md).

Local Windows closeout: 74 Python tests, 86 Rust tests (including the Windows-only
profile-store case), 25 Tauri tests; Python compile/fatal lint, Rust/Tauri formatting,
warnings-denied Clippy, Tauri compile/check and actual Windows debug executable build
passed. Demo/package/catalog, 8/8 realistic corpus cases, deterministic Python/Rust
benchmark smoke, diff whitespace and changed-document local links also passed.
Protected-main CI/review/merge remain the PR gate; no automatic merge.
