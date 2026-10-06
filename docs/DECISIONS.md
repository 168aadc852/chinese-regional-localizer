# Project Decisions

## D-001 — GitHub is the project source of truth

Date: 2026-10-06

Decision: Repository files and Issues will hold durable project context so different AI tools and human contributors can continue work without relying on a specific chat history.

## D-002 — Data governance precedes application development

Date: 2026-10-06

Decision: Build and verify the data-source whitelist, policy and schema before significant UI/application work.

Reason: Licensing and provenance are foundational to an open-source terminology/localization product.

## D-003 — Offline-first product direction

Date: 2026-10-06

Decision: Normal localization should be performed locally. Internet access is primarily for explicit database/application updates.

## D-004 — Three regional targets are first-class

Date: 2026-10-06

Decision: `zh-CN`, `zh-HK` and `zh-TW` are distinct targets. The project is not merely a Simplified/Traditional converter.

## D-005 — Separate code licences from data licences

Date: 2026-10-06

Decision: Do not apply one blanket repository licence to third-party datasets. Maintain separable data layers/packs where obligations differ.

## D-006 — AI agents must write back important context

Date: 2026-10-06

Decision: Significant findings, decisions and progress must be committed to repository documentation, not left only in AI chat history.

## D-007 — Current technical direction is cross-platform but not locked

Date: 2026-10-06

Direction: Tauri 2 + Rust core + SQLite is the leading architecture candidate for desktop/mobile reuse. This remains subject to proof-of-concept validation before implementation lock-in.

## D-008 — Machine-readable source manifest controls ingestion

Date: 2026-10-06

Decision: `data-registry/sources.yaml` is the machine-readable ingestion policy for external data sources.

Rules:

- Detailed evidence remains in individual `data-registry/*.md` review records.
- Automated import is permitted only where the relevant manifest entry has `ingest_allowed: true`.
- `null` permission fields are unresolved and must never be interpreted as permission.
- Importers must enforce `ingest_scope` and `excluded_scope` at source/file/subset level.
- Multi-licence sources should be split into separate manifest entries when practical.
- `core`, `attribution`, `sharealike`, `reference_only` and `pending` packs must remain separable so licensing obligations cannot silently contaminate other release layers.

Reason: This allows Codex, Gemini, Claude, OpenCode, future AI tools and production importers to follow the same verified data-policy decisions without re-interpreting prose or chat history.

## D-009 — Deterministic localization precedence and no-guess policy

Date: 2026-10-06

Decision: The core localization engine must remain deterministic and use the following precedence unless a later explicit decision supersedes it:

1. recognize exact/longest known entity names or aliases in the declared source locale;
2. if an entity resolves uniquely and has one preferred target-regional name, use it and protect the resulting span from generic rules;
3. if an entity is ambiguous or lacks a unique target-regional name, preserve the source span and mark it for review rather than guessing;
4. apply generic/script/regional term rules in explicit conversion stages rather than flattening all dictionaries together;
5. within a term-rule stage, longest source phrase wins before rule priority is considered;
6. for the same source phrase, the highest-priority target candidate is selected;
7. equal-priority conflicting targets are not guessed: preserve the span and mark it for review;
8. every applied or review-required decision must remain explainable through stored provenance.

The initial route model is:

- `zh-CN -> zh-HK`: entity pass, `zh-CN -> zh-Hant`, then `zh-Hant -> zh-HK`;
- `zh-CN -> zh-TW`: entity pass, `zh-CN -> zh-Hant`, then `zh-Hant -> zh-TW`;
- `zh-Hant -> zh-HK`: entity pass, then regional stage;
- `zh-Hant -> zh-TW`: entity pass, then regional stage.

Reason: Proper names must not be corrupted by general dictionaries, longest-match behavior reduces partial replacements, OpenCC's staged semantics should not be lost, and uncertain cases should be visible to the user instead of silently hallucinated.
