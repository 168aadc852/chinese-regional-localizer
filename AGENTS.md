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

## Definition of done

A task is not complete until, where applicable:

- the requested work exists in the repository;
- relevant tests/checks have been run;
- documentation reflects material changes;
- `PROJECT_STATE.md` reflects meaningful progress;
- important decisions are recorded in `docs/DECISIONS.md`;
- unresolved licensing uncertainty is explicitly documented rather than guessed.

## Handoff rule

Before ending substantial work, leave enough information in the repository for a different AI tool with no chat history to continue safely.
