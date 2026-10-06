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
