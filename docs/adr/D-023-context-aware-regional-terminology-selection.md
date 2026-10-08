# D-023 — Explicit context ranking for regional terminology

Date: 2026-10-09

Status: Accepted under [Issue #47 design approval](https://github.com/168aadc852/chinese-regional-localizer/issues/47#issuecomment-6064930045)
and [Implementation Brief v0.1](https://github.com/168aadc852/chinese-regional-localizer/issues/47#issuecomment-6064966605).

## Context

[D-009](D-009-localization-precedence.md) defines longest applicable matching,
protection and no-guess ties. [D-012](D-012-user-local-precedence.md) puts user
protected terms/fixed overrides before the shared engine. [D-022](D-022-independent-context-profile-foundation.md)
provides immutable validated single-parent snapshots, independently stored.
Issue #47 connects explicit usage-context selection without replacing those contracts.

## Decision

Reuse `context.usage_context_id` inside Runtime API v1's existing optional JSON.
Configure a validated #46 snapshot internally, defaulting to built-ins. Resolve
the requested chain once through that model before any localization; malformed,
unknown or unavailable contexts fail rather than silently choosing General.
No request paths, raw definitions, store I/O or automatic context detection.

Add one single-ID condition under the existing rule `context_constraint.constraints`.
Keep legacy domain/spatial restrictions independent, filtering before ranking.
No SQLite or private-dictionary migration is needed.

For shared regional terminology, use longest eligible phrase **before** context
specificity. For that phrase, choose exact, nearest inherited parent(s), explicit
General when inherited, then unscoped fallback. Within that level use priority.
Conflicting top targets preserve stage text, protect it and request review without
falling through. Same-target evidence is safe; rule IDs never break a wording tie.
Custom roots do not acquire an invented General ancestor.

Keep actual staged routes and context-neutral script conversion. Usage-scoped
rules do not run in generic script stages. Keep entity/user protections and strict
locale/current-data filtering. Add only optional context explanation metadata
and carry actual winning rule identity into provenance enrichment; do not invent
a winner for unresolved conflicts. Runtime API v1 outer fields remain unchanged.

Python mirrors the same behavior through an isolated immutable reference adapter;
the localizers do not embed inheritance graph logic. Production clients continue
to call the same Rust core/runtime. Common fixtures and live cross-language tests
guard selection parity, determinism and legacy requests.

## Consequences and exclusions

Context specificity can beat a broader rule's higher priority for the same phrase,
but cannot split a longer eligible phrase or overwrite protected text. The context
store/settings schemas remain independent. No new terminology source is approved.

General is not an unscoped alias. A CN-route regional conflict preserves the text
after the unchanged initial script stage, not a rollback of that stage.
Desktop profile loading/pickers, context-sensitive My Terms, alternative selection,
modes/UX, multiple parents, new routes and AI remain separately approved future work.
The exact executable contract and examples live in [Context Selection](../CONTEXT_SELECTION.md).
