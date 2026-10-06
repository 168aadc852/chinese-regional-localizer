# Chinese Regional Localizer

An open-source, offline-first Chinese regional localization project for converting and reviewing Chinese text across Mainland China (`zh-CN`), Hong Kong (`zh-HK`) and Taiwan (`zh-TW`).

> Project status: **Phase 0 — Data governance and source research**

## Goal

Build a local tool that goes beyond Traditional/Simplified Chinese character conversion and can handle regional terminology and proper names such as films, people, organisations, technology terms and other locale-specific names.

The project is intended to support desktop and mobile applications later, while keeping the translation/localization core and knowledge base reusable across platforms.

## Core principles

- Offline-first: user text should not need to leave the device.
- Explainable: important changes should have a reason and provenance.
- Region-aware: distinguish `zh-CN`, `zh-HK` and `zh-TW` instead of treating Chinese as only Simplified vs Traditional.
- Data provenance: every imported data source must have a recorded origin and licence status.
- Open development: GitHub is the project source of truth.
- AI-tool independent: ChatGPT, Codex, Gemini, Claude, OpenCode and other agents should be able to continue work from repository documentation.

## Current priority

The project is **not yet implementing the application UI**.

Current work:

1. Build a whitelist of candidate open data sources.
2. Verify licence and redistribution terms for each source.
3. Define the canonical terminology/entity data schema.
4. Define rules for separating permissive, attribution-required and share-alike data packs.
5. Build a small proof-of-concept dataset before application development.

## Repository map

- `AGENTS.md` — canonical instructions for AI coding/research agents.
- `PROJECT_STATE.md` — current project status and immediate next work.
- `docs/PROJECT_OVERVIEW.md` — product and problem definition.
- `docs/PRODUCT_REQUIREMENTS.md` — initial product requirements.
- `docs/DATA_POLICY.md` — rules governing what data can enter the project.
- `docs/DATA_SOURCES.md` — source registry index.
- `docs/DATABASE_SCHEMA.md` — draft data model.
- `docs/DECISIONS.md` — architectural and governance decisions.
- `docs/ROADMAP.md` — phased project plan.
- `data-registry/` — one review record per external source.

## For AI agents

Read `AGENTS.md` first, then `PROJECT_STATE.md`.

Important findings must not exist only inside chat history. Update the relevant repository document.

## Licensing

No single licence is currently asserted for all repository data.

Source code licensing and third-party data licensing will be handled separately. Each external source must be reviewed before its data is imported or redistributed.

See `docs/DATA_POLICY.md` and `docs/DATA_SOURCES.md`.
