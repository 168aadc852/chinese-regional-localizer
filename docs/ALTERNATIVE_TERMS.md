# Alternative Terms and One-Time Occurrence Review — Issue #48

Date: 2026-10-10. Core/runtime and Python reference only; no desktop picker or
clickable-choice UI, no remembered preferences and no new terminology coverage.

Baseline: [approved design](https://github.com/168aadc852/chinese-regional-localizer/issues/48#issuecomment-6086843519),
[approval](https://github.com/168aadc852/chinese-regional-localizer/issues/48#issuecomment-6086853168)
and [Implementation Brief v0.1](https://github.com/168aadc852/chinese-regional-localizer/issues/48#issuecomment-6086902626).

## Candidates do not change the localization decision

The engine first selects the longest **eligible** source phrase with the existing
[Issue #47 contract](CONTEXT_SELECTION.md). For that exact occurrence it collects
all otherwise eligible regional targets: same active route/stage/target locale,
current evidence and satisfied ordinary/context constraints. Shorter overlapping
phrases, unrelated contexts, wrong locales/stages, stale/inactive rows and malformed
conditions are not candidates. Generic script conversion is not a choice layer.
Entities and existing user protected/fixed terms remain above shared terminology.

Group evidence by target text, retaining each target's best rank. Order by context
level (exact, nearest inherited parent(s), General only if inherited, unscoped),
then priority descending, then target text in Unicode order. Without a usage-context
ID, only legacy unscoped rules are eligible and legacy priority ranking applies.
Priority remains technical metadata, not a field in the new user-facing candidates.

- One unique top target: `recommended`; it is applied as before.
- Other eligible targets: `also_valid`, including lower inherited-context levels.
- Multiple targets tied at the top level/priority: those targets are `needs_decision`;
  preserve stage source text and keep review-needed. Lower targets may still be
  `also_valid`, but never become an automatic fallback hiding the tie.

Duplicate evidence is not ambiguity. This adds choices without changing #47's
selected wording, longest-match boundaries, locale isolation or no-guess policy.
Same-text recommendations can carry a choice event with `applied: false` so an
alternative is still reviewable. Existing legacy explanation fields remain;
the new nested object, not old `alternatives`/`candidates`, is the #48 contract.

## Optional nested `choice` event contract

Only shared **regional term** change/review events acquire this optional object:

```json
{
  "occurrence_id": "occurrence-1",
  "state": "recommended",
  "source_text": "測試詞",
  "source_span": [1, 4],
  "stage_input_span": [1, 4],
  "output_span": [1, 4],
  "expected_text": "科技詞",
  "selected_candidate_id": "occurrence-1/candidate-1",
  "candidates": [
    {"candidate_id": "occurrence-1/candidate-1", "target_text": "科技詞", "status": "recommended"},
    {"candidate_id": "occurrence-1/candidate-2", "target_text": "短", "status": "also_valid"}
  ]
}
```

For an unresolved tie, `state` is `needs_decision`, selected ID is null and expected
text is the preserved source phrase. Runtime API v1 retains its nine outer response
fields and existing request fields. Legacy callers may ignore `choice`. No new
Tauri command or frontend engine is introduced.

All spans are half-open **Unicode scalar-character** indexes, not UTF-8 bytes,
UTF-16 code units or grapheme clusters. An emoji such as 😀 counts as one; combining
characters count separately. Frontend clients must not apply these as JavaScript
UTF-16 offsets without an explicit adapter in a separately approved integration.

- `source_span`: exact original-document interval from traversal/alignment.
- `source_text`: phrase entering the regional stage, possibly already script-normalized.
- `stage_input_span`: exact interval in that shared regional stage's input buffer.
  Under user-layer segmentation this is segment-local, not a fabricated global stage offset.
- `output_span`: exact current full-document interval used for occurrence editing.
- `expected_text`: required anchor at that output interval.

Rust composes original-character alignment across entities/script stages; Python
uses its existing alignment. Regional spans come directly from the traversal.
The user wrapper explicitly offsets original/output spans by the segment's input
and accumulated output positions. No candidate occurrence uses a first-substring
search. Existing non-choice explanation enrichment is not redesigned.

Occurrence IDs follow output traversal order; candidate IDs follow deterministic
target ranking within that occurrence. Identical input/settings/profile snapshot/
data/user state reproduce the contract, independently of SQLite insertion order.
IDs stay stable through edits/undo in one review session. They are **not durable**
preference keys or interchangeable between review sessions/reruns. A future client
must bind requests to the correct owned session, not reuse a cached ID with another one.

## Owned review state and trust boundary

Rust `alternative_terms::ReviewSession::from_response` copies a **trusted core-generated**
Runtime response into memory. Python's reference `ReviewSession(result)` matches it.
Constructing a session is an internal operation: do not accept a frontend-supplied
response/candidate array as trusted terminology. Future Desktop/CLI/MCP integrations
must retain the validated state in the core and send only IDs/revision/intent.
No API here reloads profiles, reopens a dictionary, relocalizes text or saves preferences.

`snapshot()` returns detached current text, monotonic `revision` and ordered occurrences.
Editing returned candidates/spans cannot mutate the owned state. The original
Runtime response is not changed; its other entity/user explanations remain separate
from the session's candidate-bearing review state.

`apply_choice(expected_revision, occurrence_id, candidate_id, intent)` checks:

1. caller revision equals current revision;
2. tracked spans are bounded, ordered and non-overlapping, and every current anchor matches;
3. occurrence exists and candidate belongs to its frozen validated set;
4. selection/state invariants remain consistent.

Only `use_this_time_only` executes. `remember_for_this_context` and
`remember_for_all_contexts` return unsupported/not-implemented without saving anything.
Only the intended interval is replaced. Later occurrences (including one starting
exactly at the previous end) shift by the exact character delta; earlier ones do not.
The occurrence's expected text/span/selected ID update, and state becomes `chosen`.
Candidate statuses remain frozen descriptions of the original engine ranking, not
new recommendations generated by the edit. Earlier accepted edits are preserved.
Choosing the same visible text still records/resolves a decision and increments revision.

Invalid/stale requests leave text, occurrences, revision and undo history unchanged.
Errors ask the caller to refresh/re-review, never search nearby for a guessed match.
There is no general document editor/external-text mutation API in this issue.

`undo(expected_revision)` validates current state and reverses only the most recent
accepted one-time choice. It restores prior selected/unresolved state/text/span and
later positions, preserving earlier history. Multiple edits of one occurrence undo
one step at a time. Revision increments rather than returning to an old number;
redo is not implemented. Revision exhaustion fails safely rather than wrapping.

## Exclusions and validation

No #49 My Terms schema/migration/categories/import or remembered preferences;
no #50 desktop redesign/modes; no new routes, entity alternatives, sources, LLM,
packaging or separate production-client engines. Shared/private schemas, settings v1,
source/licence gates and update/signing/network code are untouched.

`alternative_terms_cases.json` adds 25 project-authored synthetic scenarios to the
50 #47 cases. Golden tests cover candidates, protection, source/output spans, BMP/
non-BMP text, script expansion/contraction, repeated/adjacent occurrences and user
segment offsets. Tests repeat both insertion orders, compare edit traces with an
independent splice model and execute live Python/Rust parity for complete choice
objects and edit/undo snapshots. Separate tests cover atomic stale/unknown/wrong-
occurrence choices, lost candidates, corrupt anchors/overlaps/bounds, readonly
snapshots, unsupported remember intents, stale undo and revision exhaustion.

```text
python -m unittest discover -s tests -p test_alternative_terms.py -v
cargo test --locked --manifest-path rust/Cargo.toml --test alternative_terms
```

Rust live parity uses installed `python`, or `CRL_TEST_PYTHON` when explicitly set.
Full existing suites and CI remain required. See [ADR D-024](adr/D-024-occurrence-safe-one-time-alternatives.md).

Local Windows validation passed: 63 Python tests, 76 Rust tests (including existing
#46/#47 and Windows-only storage coverage), 25 Tauri tests, Python compile/fatal
lint, Rust/Tauri format and warnings-denied Clippy, Tauri all-target compile and
real Windows debug build. Manifest/demo/package/catalog smoke, 8/8 realistic corpus
cases, deterministic Python/Rust benchmark smoke, diff whitespace and documentation
local-link checks passed. Protected-main CI/review/merge remain the closeout gate.
