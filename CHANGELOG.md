# Changelog

## 2026-10-06

- Created the initial governance-first repository structure.
- Added AI-agent handoff documentation.
- Added draft data policy, data-source registry, schema and roadmap.
- Expanded the data-source registry to 23 records after separating DATA.GOV.HK from the Combined DoJ Glossaries.
- Completed usable first-pass reviews for 20 of 23 source records.
- Added machine-readable `data-registry/sources.yaml` ingestion policy.
- Added source manifest schema and licence packaging model.
- Added SQLite schema v0.1 with provenance and licence-pack metadata.
- Implemented manifest-enforced Phase 0.5 SQLite PoC builder.
- Added non-authoritative CN/HK/TW test fixtures and automated tests.
- Added GitHub Actions CI; PoC unit tests and demo database build passed.
- Added Phase 1A real importer for the LSHK Jyutping Table with pinned revision, SHA-256 provenance, strict TSV validation, multi-reading support and CI tests.
- Added pronunciation schema migration and documentation.
- Added Phase 1B real OpenCC phrase importer pinned to commit `3ac34aa439a9908dd49fa92b5174b46314787ac2`.
- Imported-model support for staged `STPhrases`, `HKPhrases` and `TWPhrases` rules without flattening CN/HK/TW conversion semantics.
- Preserved OpenCC multi-candidate mappings, identity mappings, dictionary/line provenance and per-file SHA-256 source versions.
- Added OpenCC parser/importer tests covering HK/TW proper-name and terminology examples; CI passed.
