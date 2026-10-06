# Chinese Regional Localizer

Offline-first, open-source Chinese regional localization project for converting and localizing Chinese text across Mainland China (`zh-CN`), Hong Kong (`zh-HK`) and Taiwan (`zh-TW`).

The project is intended to go beyond Simplified/Traditional character conversion by combining deterministic conversion, regional terminology, named-entity localization, provenance, confidence/review workflows and an updateable local knowledge base.

## Current phase

**Phase 0 — Data governance and source research.**

Before building the application, this repository is defining:

- which external data sources may be used;
- what their licences permit;
- how sources must be attributed and separated;
- how terminology/entity records retain provenance;
- how future AI agents and developers continue the project safely.

See `PROJECT_STATE.md` for current progress.

## Planned product direction

The target product is an offline-first Chinese Regional Localization Assistant with:

- automatic source-locale detection;
- output to Mainland Chinese, Hong Kong Chinese or Taiwan Chinese;
- character/script conversion;
- regional terminology localization;
- film, person, organisation, place and other named-entity localization;
- explainable changes with source information;
- confidence/review for ambiguous changes;
- user dictionaries and protected terms;
- independently updateable terminology/entity databases;
- future desktop and mobile clients.

Current technical direction is Tauri 2 + Rust + SQLite, but implementation choices are not locked until the data proof of concept is complete.

## Repository workflow

GitHub is the project's single source of truth.

AI tools and human contributors should start with:

1. `AGENTS.md`
2. `PROJECT_STATE.md`
3. the relevant documents under `docs/`

Important findings must be written back to the repository rather than left only in chat history.

## Data-source policy

No external dataset should be assumed reusable because it is publicly visible.

Every external source must be reviewed under `data-registry/` before ingestion or redistribution. Allowed statuses include:

- `approved`
- `approved_with_conditions`
- `reference_only`
- `pending_review`
- `rejected`

See:

- `docs/DATA_POLICY.md`
- `docs/DATA_SOURCES.md`
- `data-registry/TEMPLATE.md`

## Development state

No production application has been implemented yet. Source data, schema and licensing are being defined first so later desktop/mobile development does not need to undo unsafe assumptions.

## Important note

This repository will contain materials under different licences. Do not assume a single repository-wide software licence applies to third-party data. Each data source must retain its own licensing and attribution requirements.
