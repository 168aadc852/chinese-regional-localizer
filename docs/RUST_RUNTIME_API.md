# Rust Runtime API v1

Phase 3C introduces the application-facing Rust runtime boundary that future Tauri commands should call instead of depending directly on the lower-level shared or user-local engines.

## Version

The current contract reports:

```text
api_version = "1"
```

Changing the meaning or shape of existing v1 fields should be treated as an API change rather than an internal refactor.

## Request

`RuntimeRequest` contains:

- `api_version`
- `text`
- `source_locale`
- `target_locale`
- optional JSON `context`

The supported routes remain the deterministic Phase 3A routes.

Issue #47 accepts additive `context.usage_context_id`, for example
`{"usage_context_id":"technology-software"}`. Omitting this key retains legacy behavior;
legacy `domain` remains independent. Unknown, malformed or unavailable context IDs
return an error, not an implicit General fallback. Requests do not supply profile paths
or definitions: Rust callers may configure `Runtime::with_context_profiles` with a
validated immutable #46 snapshot, otherwise built-in defaults apply. No profile-file
I/O occurs during localization. See [Context Selection](CONTEXT_SELECTION.md).

## Response

`RuntimeResponse` contains:

- `api_version`
- original `input`
- localized `output`
- source/target locale
- executed route stages
- structured `changes`
- `review_needed`
- `user_dictionary_applied`

The same entry point supports shared-only execution or shared + private `user_dictionary.sqlite` execution.

## Explanation enrichment

The runtime wrapper preserves the Phase 3A/3B localization decisions and enriches their UI-facing explanation data.

Entity events can include:

- concept ID and concept type;
- Wikidata QID where available;
- confidence;
- current source/evidence records;
- original-input and final-output character spans.

Term-rule events can include:

- route stage;
- rule ID, priority, rule type and domain;
- source/source-version IDs;
- upstream record ID and URL;
- version/revision/checksum provenance;
- confidence;
- original-input and final-output character spans.

User-local events retain the Phase 3B `user_term_id`, note, `user_dictionary` provenance and spans.

For explicit usage-context requests, shared regional term events may add optional
`context_selection` containing `usage_context_id`, `matched_usage_context_id`,
`context_distance` and `context_level` (`exact`, `parent`, `general`, `unscoped`).
Unscoped fallback has null matched ID/distance. Context-free requests, script events
and entity/user events do not acquire this metadata. Winning provenance identifies
an actual eligible rule; equal-level conflicting targets do not invent a winning rule.
These are additive explanation fields, not an alternatives-choice UI contract or
changes to the existing v1 outer fields.

## Occurrence choices — Issue #48

Shared regional term events may add optional nested `choice` with deterministic
session-local occurrence/candidate IDs, states, target texts, exact Unicode spans,
expected current text and selected candidate ID. The same nine outer response
fields and legacy request behavior remain; entities, user fixed/protected terms
and generic script events do not receive choices. See [Alternative Terms](ALTERNATIVE_TERMS.md)
for the exact contract and trust boundary. Identity recommendations may produce
an unapplied regional event so that an otherwise valid alternative remains reviewable.

`alternative_terms::ReviewSession::from_response(&response)` copies the trusted
core result into an in-memory review. `snapshot()` returns a detached view;
`apply_choice(revision, occurrence_id, candidate_id, ChoiceIntent::UseThisTimeOnly)`
and `undo(revision)` operate on this owned state without invoking the runtime again.
Invalid/stale requests return `ChoiceError` without changing text, revision or history.
Remember intents return `UnsupportedIntent`; no preference is stored.
This is an internal shared-core operation, not a new Tauri command, Runtime v2,
frontend engine, or #49 persistence workflow.

## CLI

The CLI is now a thin caller of the runtime API. Shared-only example:

```bash
cargo run --manifest-path rust/Cargo.toml -- \
  --db build/regional-demo.sqlite \
  --from zh-CN \
  --to zh-TW \
  --text "布拉德·皮特研究人工智能"
```

With private user rules:

```bash
cargo run --manifest-path rust/Cargo.toml -- \
  --db build/regional-demo.sqlite \
  --user-db path/to/user_dictionary.sqlite \
  --from zh-CN \
  --to zh-TW \
  --text "OpenAI人工智能"
```

## Rust benchmark

A reproducible benchmark binary uses the existing project-authored realistic corpus and expands it to a requested input size:

```bash
cargo run --release --manifest-path rust/Cargo.toml --bin benchmark -- \
  --db build/regional-demo.sqlite \
  --corpus data/fixtures/realistic_corpus.json \
  --target-chars 5000 \
  --iterations 5
```

It reports determinism, characters per second and mean/p50/p95/max latency as JSON.

GitHub-hosted runner measurements are regression/smoke signals only. They are not product performance promises and should not be compared as if runner hardware were fixed.

## Boundary for Tauri

Future Tauri commands should call this runtime API rather than reproduce localization logic. UI work must not bypass the deterministic/no-guess rules or directly reinterpret shared/user database precedence.
