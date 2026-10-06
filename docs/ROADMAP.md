# Roadmap

## Phase 0 — Data governance

- Data-source whitelist
- Licensing evidence registry
- Data policy
- Draft schema
- Initial test corpus

Exit condition: enough verified sources and schema clarity to build a small legal/traceable dataset prototype.

## Phase 1 — Data proof of concept

- Implement a small number of importers
- Normalize source records
- Build SQLite prototype
- Preserve provenance
- Validate regional lookup examples

## Phase 2 — Localization core

- Script/character conversion
- Regional term lookup
- Entity matching
- Rule priority/conflict handling
- Protected terms
- Explainable change records
- Automated tests

## Phase 3 — Minimal application

- Text input/output
- Auto/source selection
- CN/HK/TW targets
- Diff/review
- Source explanation cards
- User dictionary
- Local database update flow

## Phase 4 — File workflows

- TXT / Markdown
- subtitles
- CSV
- later: Office and EPUB

## Phase 5 — Cross-platform distribution

- Windows
- Android
- macOS
- iOS/iPadOS

## Phase 6 — Optional ambiguity intelligence

Only after deterministic methods are measured. Consider a local LLM for unresolved context-sensitive cases rather than for all conversions.
