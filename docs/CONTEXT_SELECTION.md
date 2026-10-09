# Deterministic Context-Aware Terminology — Issue #47

Date: 2026-10-09. This is a core/runtime capability, not a desktop context picker
or a claim of governed terminology coverage for every context.

Implementation baseline: [approved design](https://github.com/168aadc852/chinese-regional-localizer/issues/47#issuecomment-6064744646),
[approval](https://github.com/168aadc852/chinese-regional-localizer/issues/47#issuecomment-6064930045)
and [Implementation Brief v0.1](https://github.com/168aadc852/chinese-regional-localizer/issues/47#issuecomment-6064966605).

## Selecting a context

Pass a stable profile ID through Runtime API v1's existing optional JSON context:

```json
{"usage_context_id": "technology-software"}
```

Rust `Runtime`, `LocalizerEngine` and `UserLocalizer` accept a validated immutable
`ContextProfiles` snapshot through `with_context_profiles`. Without one, the seven
[built-ins](CONTEXT_PROFILES.md) apply. Resolution reuses the #46 resolver; the
scanner receives only its specific-to-general chain. No request-supplied path,
raw profile definitions, file I/O, inference or LLM is involved.

Python's reference engine accepts `context_profiles=ContextProfiles(...)` using
the immutable adapter in `src/context_profiles.py`. It mirrors #46's v1 validation
for reference/parity use; production Desktop/CLI/MCP clients reuse the Rust core.
The adapter neither implements a second local store nor changes settings files.

Omitting `usage_context_id` preserves legacy behavior. An explicit ID must be a
nonempty string resolving to an enabled chain. Unknown/disabled profiles and
disabled ancestors fail clearly before localization, even for empty input or
fully user-protected text. There is no silent fallback to General.
Old `domain` values are independent: `technology` is not automatically mapped
to `technology-software`.

## Rule conditions and selection

The shared SQLite schema stays v0.2. Use the existing `context_constraint` JSON:

```json
{"constraints": {"usage_context_id": "technology-software", "domain": "technology"}}
```

The usage condition is one stable ID, not a list, wildcard or inferred topic.
Both usage and legacy domain must match when both are present. Existing spatial
and word-boundary conditions still apply. Invalid JSON/condition structures or
invalid usage conditions fail closed; they do not become unrestricted rules.

For each unprotected text position in a regional terminology stage:

1. Filter by route/source/target locale, active/current data and all ordinary conditions.
2. Find the longest source phrase with an eligible rule for the resolved chain.
3. For that phrase, take the first eligible level: exact context, nearest parent,
   further ancestors, General **only when in the chain**, then unscoped fallback.
4. Compare numeric priority only within that level.
5. Same-priority records with one target are safe; conflicting targets preserve the
   stage's source phrase, protect it and set `review_needed`. Do not fall through
   to parent/General/unscoped rules to hide the conflict.

An exact-context priority 10 beats inherited General priority 1000 for the **same
phrase**. A valid longer General phrase still beats a shorter exact-context phrase.
An ineligible longer phrase permits a shorter eligible match.

A custom root with `parent_context_id: null` does not inherit General. Its own
rules can fall back to unscoped regional rules, but not unrelated scoped contexts.
General and unscoped are distinct, including when General itself is selected.

## Preserved boundaries

- Existing user protected terms and fixed overrides remain ahead of shared rules;
  private dictionary schema/context preferences do not change.
- Conservative entity resolution runs first in shared processing. Resolved and
  ambiguous entity spans remain protected from context terminology.
- Hong Kong and Taiwan candidates never cross target regions.
- Actual routes stay `zh-CN → zh-Hant → zh-HK/zh-TW` or `zh-Hant → zh-HK/zh-TW`.
  The initial script stage remains context-neutral; usage-scoped rules do not run
  in that stage. A regional conflict preserves the text entering the regional
  stage, which may already have undergone script conversion.
- Existing Runtime API v1 outer fields and their meanings stay unchanged.
- Context profiles authorize no new data. Source/ingestion/licence gates stay intact.
- No #48 candidate-choice UI, #49 My Terms migration, #50 desktop controls/modes,
  new routes, packaging or automatic context detection is implemented.

## Explanation and determinism

Regional change/review events with an explicit selection may include optional
`context_selection`: requested ID, matched ID, chain distance and the level
`exact`, `parent`, `general` or `unscoped`. Unscoped uses null matched ID/distance.
Other event types and legacy requests retain their existing fields.

Actual winning rule identity is carried into Runtime provenance enrichment so a
different-context, higher-priority record cannot claim the decision. Multiple
same-target winning records are not a wording conflict; a stable actual winning
record supplies existing provenance. Conflicting targets get no invented winner.

Output/review decisions depend only on input, route/context, validated profile
snapshot, shared data version and user dictionary state. Repeat calls and different
SQLite insertion orders must agree. This is not the future #48 candidate-list contract.

## Validation

`data/fixtures/context_selection_cases.json` is a project-authored shared contract
with 50 cases. Python and Rust both check the same expected results, repeated runs,
reversed insertion order, protection, inheritance, context-neutral scripts and
locale isolation. Rust also executes the Python reference to compare live results.
`CRL_TEST_PYTHON` can select that test interpreter; otherwise `python` is used.

Focused commands:

```text
python -m unittest discover -s tests -p test_context_selection.py -v
cargo test --locked --manifest-path rust/Cargo.toml --test context_selection
```

Separate regressions cover malformed conditions, adapter validation/immutability,
true rule provenance and the unchanged nine-field Runtime v1 response. Existing
#46 profile/store, user dictionary, corpus, governance and Tauri tests remain required.
See [ADR D-023](adr/D-023-context-aware-regional-terminology-selection.md).

Local Windows checks passed: 58 Python tests, 69 Rust tests (including the Windows
#46 locked-file case), 25 Tauri tests, Python compile/fatal lint, Rust/Tauri format
and warnings-denied Clippy, Tauri all-target compile and Windows debug build,
manifest/demo/package/catalog smoke, 8/8 realistic corpus cases and deterministic
Python/Rust benchmark smoke. Protected-main CI/review/merge remain the PR closeout gate.
