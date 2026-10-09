# D-024 — Occurrence-safe one-time alternatives without reruns

Date: 2026-10-10

Status: Accepted under [Issue #48 approval](https://github.com/168aadc852/chinese-regional-localizer/issues/48#issuecomment-6086853168)
and [Implementation Brief v0.1](https://github.com/168aadc852/chinese-regional-localizer/issues/48#issuecomment-6086902626).

## Context

[D-009](D-009-localization-precedence.md), [D-012](D-012-user-local-precedence.md)
and [D-023](D-023-context-aware-regional-terminology-selection.md) protect entities/
user choices, select the longest eligible phrase and avoid guessed ties. #48 needs
valid alternatives and edits for one occurrence without changing those decisions.
Repeated text cannot safely be located with a first-substring search.

## Decision

After choosing the phrase, group all eligible shared regional targets by text,
retain each target's best context/priority rank, then sort by context, descending
priority and target text. Label one unique top target recommended, lower targets
also valid, or equal top targets needs decision. Preserve ties exactly as #47 does.
Context-neutral script conversion and existing entity/user handling are not choosers.

Add optional nested `choice` metadata to regional events, not the outer Runtime v1
contract. Capture exact Unicode-character spans during traversal; compose original
alignment across stages and explicitly offset user-layer shared segments. Assign
deterministic session-local IDs after output ordering. Keep true winning provenance;
do not invent a winner for a conflict. Retain legacy explanation fields.

Construct owned Rust review state only from trusted core results, with Python
reference parity. Freeze candidate sets, expose detached snapshots, and accept only
expected revision, occurrence ID, candidate ID and choice intent for editing.
Validate all anchors/bounds/ordering/selection invariants before mutation. Replace
one interval, shift later positions, preserve earlier choices and record reversible
state. Reject stale/invalid requests atomically. A same-text choice is still a
decision. LIFO undo validates first and monotonically advances revision; no redo.

Only this-time-only intent executes. Remember-intent hooks are unsupported and
perform no storage. Future clients bind IDs to the correct core-owned session;
occurrence IDs are neither durable My Terms keys nor tokens usable across reruns.
No frontend-only engine or caller-supplied unvalidated candidate set is authorized.

## Consequences and exclusions

The selected wording/review decisions remain #47-compatible while explanations
gain alternatives across eligible inherited levels. Review snapshots are distinct
from the original Runtime response; candidate statuses describe its initial ranking,
and a user's accepted occurrence state is `chosen`. Span indexes are Unicode scalar
characters, requiring an explicit UTF-16 adapter for a future JavaScript integration.

No desktop/settings/schema change, persistent preferences, new routes/entities/
sources, LLM, packaging or later #49/#50 implementation. Common project-authored
fixtures, reversed ordering, splice/undo traces and live Python/Rust parity prove
the contract. Details live in [Alternative Terms](../ALTERNATIVE_TERMS.md).
