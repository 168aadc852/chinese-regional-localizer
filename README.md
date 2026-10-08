# Chinese Regional Localizer

Offline-first, open-source Chinese regional localization project for converting and localizing text across the Chinese Mainland (`zh-CN`), Hong Kong (`zh-HK`) and Taiwan (`zh-TW`).

The project goes beyond Simplified/Traditional character conversion by combining deterministic script conversion, regional terminology, named-entity localization, user overrides, provenance, ambiguity review and updateable SQLite data packs.

## Current state

**Phase 3I desktop authenticated data-update UI is complete. Phase 0 source-whitelist review has a usable first-pass conclusion for all 24 machine source IDs.**

Phase 3J / Issue #38 adds tested desktop settings persistence in this change;
issue closure remains subject to PR review, CI and protected-main merge.

Implemented foundations include:

- governed, licence-aware source ingestion with machine-readable allow-lists;
- SQLite schema v0.2 with source/version/provenance history;
- deterministic Python reference localization engine;
- Rust shared core and versioned Runtime API v1 for supported CN/HK/TW routes;
- private user-dictionary support and protected/user-preferred terms;
- Tauri 2 desktop shell with Rust-managed validated database selection;
- versioned non-secret desktop database preferences with safe restart validation and atomic saves;
- full local database paths kept inside Rust rather than exposed to frontend JavaScript;
- immutable versioned shared-data packages with staging, activation and rollback;
- pinned Ed25519 signatures for release catalogs and package manifests;
- authenticated HTTPS update discovery/download with rollback protection and bounded downloads;
- desktop controls for explicit authenticated shared-data update checks and installs;
- offline fixture-backed CI covering Python, Rust, Tauri and package/catalog flows;
- protected `main` requiring pull requests and the `test` status check.

See `PROJECT_STATE.md` for the concise current status and next work.

## Supported localization routes

Current supported forward routes include:

- `zh-CN -> zh-HK`
- `zh-CN -> zh-TW`
- `zh-Hant -> zh-HK`
- `zh-Hant -> zh-TW`

Reverse routes are not yet implemented. OpenCC behavior remains a documented partial/reference implementation rather than full upstream runtime parity.

## Desktop application status

A Tauri 2 desktop application now exists and uses the Rust runtime rather than frontend-side SQLite access.

Current desktop capabilities include:

- text localization through Runtime API v1;
- validated shared-database selection through a native chooser;
- optional private user-dictionary selection/clearing;
- persisted shared/private database choices and private dictionary enabled state;
- structured database status without exposing full filesystem paths to JavaScript;
- explicit authenticated shared-data update discovery and installation;
- safe failure behavior where invalid downloads or packages do not replace the active database.

Current desktop limitations:

- persisted settings cover database choices and dictionary enabled state only;
- shared-data update checks are manual rather than background scheduled;
- interrupted-download retry/resume and production hosting/CDN configuration are not complete;
- production Windows/macOS installers and platform signing are not complete;
- mobile packaging has not started.

Relevant documentation:

- `docs/DESKTOP_DATABASE_SETTINGS.md`
- `docs/DESKTOP_DATA_UPDATES.md`
- `docs/NETWORK_UPDATES.md`
- `docs/DATA_PACKAGE_FORMAT.md`

## Quick development demo

The Python reference path remains useful for local development and regression checks:

```bash
python -m pip install -r requirements-dev.txt
python scripts/build_demo_database.py
python scripts/localize_text.py \
  --db build/regional-demo.sqlite \
  --from zh-CN \
  --to zh-TW \
  --text "布拉德·皮特研究人工智能"
```

The fixture database is for tests/development only and is not an authoritative terminology release.

## Data governance

Public visibility is not permission to ingest, transform or redistribute data. External sources are reviewed under `data-registry/` and represented in `data-registry/sources.yaml`.

Production importers require all of the following:

- `ingest_allowed: true`;
- an exact resource listed in `ingest_resources`;
- the expected licence pack;
- revision/version, URL and checksum provenance;
- input-format and resource-identity validation.

Reference-only, pending, rejected or out-of-scope material is blocked from redistributable data builds.

The first-pass whitelist review now has a usable conclusion for all 24 machine source IDs. Sources with unresolved rights are kept reference-only/non-ingest rather than being guessed into an approved state.

See:

- `docs/DATA_POLICY.md`
- `docs/DATA_SOURCES.md`
- `docs/REVIEW_PROGRESS.md`
- `docs/LICENSE_PACKAGING.md`

## Repository workflow

GitHub is the project source of truth. AI tools and human contributors should begin with `AGENTS.md`, then load only the task-specific context routed under `.ai/` and `docs/`.

Behavior changes require tests. Importers must preserve provenance, obey exact machine-readable ingest resources and keep incompatible licence packs separated.

Current-state detail belongs in `PROJECT_STATE.md`; historical detail belongs in `docs/history/`, `CHANGELOG.md` and ADRs under `docs/adr/`.

## Product requirements

The canonical editable requirements are in `docs/PRODUCT_REQUIREMENTS.md`.

A dated frozen backup is also kept at:

- `docs/history/PRODUCT_REQUIREMENTS_BACKUP_2026-10-07.md`

The backup is for recovery/reference only and does not supersede the canonical requirements file.

## Licensing

Project-authored software is licensed under Apache License 2.0; see `LICENSE` and `LICENSE_SCOPE.md`.

Third-party data is **not** relicensed under the software licence. Each source keeps its own licence and attribution/share-alike obligations.

## Next planned work

**Phase 3J / Issue #38** implements persisted non-secret desktop settings and awaits
review/CI/merge before being treated as a closed phase.

Validated shared-database and optional user-dictionary choices now survive app
restarts, preserving the Rust-only filesystem-path boundary, safe fallback behavior
and Runtime API v1 compatibility.

After this focused PR is opened, the next step is planning the HanContext Alpha MVP
and its small reviewable issues. New Alpha features are not implemented here.
