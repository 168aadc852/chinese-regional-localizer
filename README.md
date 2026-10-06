# Chinese Regional Localizer

Offline-first, open-source Chinese regional localization project for converting and localizing text across Mainland China (`zh-CN`), Hong Kong (`zh-HK`) and Taiwan (`zh-TW`).

The project goes beyond Simplified/Traditional character conversion by combining deterministic script conversion, regional terminology, named-entity localization, provenance, conflict review and updateable SQLite data packs.

## Current state

**Phase 2C — Core hardening.**

Implemented foundations:

- machine-readable data-source policy and licence-pack separation;
- SQLite schema v0.2 with current/superseded source-version tracking;
- LSHK Jyutping importer with pinned Git-blob verification;
- OpenCC forward CN → Hant → HK/TW phrase/character/variant import;
- Wikidata regional entity-name importer with revision-aware refresh;
- deterministic reference localization engine with conservative entity matching, longest-match rules, protected spans and no-guess ambiguity handling;
- original-to-final span alignment for future diff/review UI;
- offline fixtures, regression tests and GitHub Actions CI.

Production Rust/Tauri clients have not started. The Python code is the reference implementation used to stabilize behavior first.

See `PROJECT_STATE.md` for the concise current status.

## Supported reference routes

- `zh-CN -> zh-HK`
- `zh-CN -> zh-TW`
- `zh-Hant -> zh-HK`
- `zh-Hant -> zh-TW`

Reverse routes are not yet implemented.

## Quick development demo

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

## Repository workflow

GitHub is the project source of truth. AI tools and human contributors should begin with `AGENTS.md`, then load only the task-specific context it routes to under `.ai/` and `docs/`.

Behavior changes require tests. Importers must preserve provenance, obey exact machine-readable ingest resources and keep incompatible licence packs separate.

## Data governance

Public visibility is not permission to ingest or redistribute data. Every external source is reviewed under `data-registry/` and represented in `data-registry/sources.yaml`.

Production importers require all of the following:

- `ingest_allowed: true`;
- an exact resource listed in `ingest_resources`;
- the expected licence pack;
- revision/version, URL and checksum provenance;
- input-format validation.

Pending, reference-only, rejected, or out-of-scope material is blocked.

## Licensing

Project-authored software is licensed under Apache License 2.0; see `LICENSE` and `LICENSE_SCOPE.md`.

Third-party data is **not** relicensed under the software licence. Each source keeps its own licence and attribution/share-alike obligations. See `docs/LICENSE_PACKAGING.md` and the corresponding `data-registry/*.md` review.

## Production direction

The planned application direction remains Rust + Tauri 2 + SQLite after reference behavior, update semantics and regression coverage are stable. User dictionaries/protected terms come before the Rust port; UI work follows the stable core rather than defining core behavior implicitly.
