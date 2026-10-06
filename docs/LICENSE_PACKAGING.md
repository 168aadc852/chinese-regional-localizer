# Data Licence Packaging Model

This document records the working packaging model emerging from Phase 0 source reviews.

## 1. Permissive / core layer
Suitable candidates include CC0, Apache-2.0, Unicode-3.0 and comparable permissive/public-domain sources, subject to each source record.

Examples:
- Wikidata structured data
- PanLex official snapshots
- OpenCC
- Unicode / Unihan / CLDR machine-readable Data Files
- Words.hk explicitly public-domain word-list/pronunciation dataset
- MusicBrainz Core Data
- THUOCL published repository package, subject to recorded provenance safeguards

## 2. Attribution-required non-ShareAlike layer
Keep attribution metadata and notices but no general ShareAlike requirement.

Examples:
- verified Taiwan Government OGL-Taiwan v1 datasets
- qualifying DATA.GOV.HK data within portal terms
- Rime Cantonese main CC BY 4.0 content

## 3. ShareAlike-isolated data packs
Do not silently merge these sources into a permissive-only database release.

Examples:
- Chinese Wikipedia-derived data — CC BY-SA 4.0
- Chinese Wiktionary text-derived data — CC BY-SA 4.0
- CC-CEDICT — CC BY-SA 4.0
- CC-Canto — CC BY-SA 3.0
- Rime `jyut6ping3.maps` — ODbL 1.0 database layer

## 4. Non-commercial / reference-only content
Exclude from the standard commercial-capable downloadable database unless separately licensed.

Examples:
- full Words.hk dictionary under Non-Commercial Open Data License
- MusicBrainz Supplementary Data under CC BY-NC-SA 3.0

## 5. Pending rights clarification
Do not ingest into redistributable derived databases yet.

Example:
- Combined DoJ Glossaries of Legal Terms

## Engineering rule
Every imported record should retain `source_id`, upstream URL/identifier, source version or revision, licence class, and transformation history. Importers must use explicit per-source and, where needed, per-file allowlists.
