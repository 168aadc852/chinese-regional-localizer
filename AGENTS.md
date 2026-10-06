# Canonical AI Agent Instructions

This file is the canonical entry point for AI tools working on this repository.

## Before doing any work

Read, in this order:

1. `README.md`
2. `PROJECT_STATE.md`
3. `docs/PROJECT_OVERVIEW.md`
4. `docs/PRODUCT_REQUIREMENTS.md`
5. `docs/DECISIONS.md`

For any data, terminology, licensing, ingestion, scraping, API, database or knowledge-base work, also read:

6. `docs/DATA_POLICY.md`
7. `docs/DATA_SOURCES.md`
8. `docs/DATABASE_SCHEMA.md`
9. `data-registry/sources.yaml`
10. `docs/SOURCE_MANIFEST_SCHEMA.md`

## Source-of-truth rule

GitHub repository files are the project memory.

Do not leave an important decision, research result, licensing conclusion, schema change or implementation status only in chat history.

Write it back to the appropriate repository file.

## Data safety rules

- Never invent or guess licence terms.
- Publicly viewable does **not** automatically mean redistributable.
- Do not mark a source `approved` without verifiable licensing evidence.
- If rights are unclear, mark the source `pending_review` or `reference_only`.
- Keep source code licensing separate from third-party data licensing.
- Preserve provenance fields when transforming external data.
- Do not copy proprietary or restricted datasets into the repository.
- Do not bulk scrape a source until its terms and technical access method have been reviewed.
- Automated importers must treat `data-registry/sources.yaml` as the machine-readable ingestion policy.
- Only entries with `ingest_allowed: true` may be imported automatically.
- `null` permission fields mean unresolved, never permitted.
- Importers must obey both `ingest_scope` and `excluded_scope`; source-level approval does not imply every file or namespace is approved.

## Architecture rules

- Preserve the offline-first goal.
- Treat `zh-CN`, `zh-HK` and `zh-TW` as distinct localization targets.
- Do not reduce the product to only Simplified/Traditional character conversion.
- Keep deterministic conversion and terminology/entity lookup separate from optional future LLM assistance.
- Do not begin major UI implementation during Phase 0 unless `PROJECT_STATE.md` explicitly changes the current phase.
- Important architectural changes must be recorded in `docs/DECISIONS.md`.

## Working with external sources

Each source must have a record under `data-registry/` containing, where available:

- official name
- official URL
- data type
- region coverage
- licence name
- licence URL/evidence
- commercial-use status
- modification status
- redistribution status
- attribution requirement
- share-alike requirement
- packaging/republication implications
- update method
- intended use
- restrictions
- review date
- review status

Allowed statuses:

- `approved`
- `approved_with_conditions`
- `reference_only`
- `pending_review`
- `rejected`

When a source reaches a usable conclusion, keep its detailed review record and `data-registry/sources.yaml` consistent.

## Definition of done

A task is not complete until, where applicable:

- the requested work exists in the repository;
- relevant tests/checks have been run;
- documentation reflects material changes;
- `PROJECT_STATE.md` reflects meaningful progress;
- important decisions are recorded in `docs/DECISIONS.md`;
- source-review changes are reflected in `data-registry/sources.yaml`;
- unresolved licensing uncertainty is explicitly documented rather than guessed.

## Handoff rule

Before ending substantial work, leave enough information in the repository for a different AI tool with no chat history to continue safely.
