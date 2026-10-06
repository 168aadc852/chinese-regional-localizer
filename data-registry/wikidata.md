# Wikidata

Status: **approved**

## Intended use
Structured entities, identifiers, labels and aliases for people, works, organisations, places and other named concepts.

## Region coverage
Global; Chinese locale labels and aliases where available, including locale-specific Chinese labels when present.

## Official source
URL: https://www.wikidata.org/
Publisher: Wikimedia Foundation / Wikidata community

## Licence review
Licence name: Creative Commons CC0 1.0 for structured data
Licence evidence URLs:
- https://www.wikidata.org/wiki/Wikidata:Licensing
- https://www.wikidata.org/wiki/Wikidata:Database_download
- https://www.wikidata.org/wiki/Wikidata:Copyright

Scope verified:
- Structured data in the main, Property, Lexeme and EntitySchema namespaces is released under CC0.
- Text in other namespaces is not part of this approval and may instead be CC BY-SA.

- Commercial use: **Yes**
- Modification: **Yes**
- Redistribution: **Yes**
- Attribution: **Not legally required by CC0 for the approved structured-data scope**, though this project should still preserve provenance (`source_id`, QID, dump date / revision where practical) for auditability and quality control.
- Share-alike / copyleft: **No** for the approved structured-data scope.
- Database-specific obligations: **No CC0 share-alike or attribution obligation.** CC0 does not waive third-party trademark, privacy, publicity or similar rights; these remain outside the database licence.
- API/download/scraping restrictions: For bulk ingestion, use official dumps or documented data-access interfaces rather than scraping rendered pages. This approval is for the data licence; API operational limits and usage etiquette must still be respected separately.

## Packaging decision
**Approved for the core data layer, limited to CC0 structured data.**

Recommended uses:
- QIDs and entity types
- labels and aliases
- locale-specific Chinese labels / aliases when present
- cross-database identifiers
- structured statements useful for entity disambiguation

Do **not** automatically ingest arbitrary prose from project/help/talk/other non-structured namespaces into the CC0 core database.

## Phase 1C importer baseline

The first entity importer uses Wikidata's documented stable Linked Data interface:

`https://www.wikidata.org/wiki/Special:EntityData/<QID>.json`

Implementation:

- `scripts/import_wikidata_entities.py`
- `tests/test_wikidata_importer.py`
- `data/fixtures/wikidata/entities.json`
- `data/fixtures/wikidata/entity_types.json`
- `docs/WIKIDATA_ENTITY_IMPORTER.md`

Initial locale scope:

- `en`
- `mul`
- `zh`
- `zh-hans`
- `zh-hant`
- `zh-cn`
- `zh-hk`
- `zh-tw`

Regional locale codes are normalized to BCP-47-style casing in SQLite. Missing regional labels are **not synthesized** from generic Chinese labels.

Each entity records:

- stable Wikidata QID in `external_ids`;
- explicit project concept type;
- labels as preferred names;
- aliases as alias names;
- `lastrevid` and modified timestamp where present;
- retrieval time;
- revision-pinned upstream URL for real source data;
- canonical per-entity SHA-256;
- evidence rows for each imported label/alias.

The initial fixture uses Brad Pitt (`Q35332`) and Oppenheimer (2023 film, `Q108839994`) only to test the data model offline. Fixture revision numbers are deliberately non-authoritative and use `fixture://` provenance URLs.

## Update method
Preferred sources:
1. Official JSON dumps for full rebuilds. Wikidata states that JSON dumps are produced weekly and are the recommended stable dump format.
2. Official incremental / add-change dumps for daily changes where appropriate.
3. Documented Linked Data/API access for targeted entity enrichment, subject to operational limits.

For selected-entity refreshes, record `lastrevid` and use revision-specific EntityData URLs where practical. Record the dump date or extraction timestamp in build metadata so a released local database can be reproduced.

## Review evidence / rationale
Wikidata's official licensing page states that structured data in the main, Property, Lexeme and EntitySchema namespaces is released under CC0. Its official database-download page explicitly states that the databases may be used for personal or commercial use, backups or offline use, and documents both weekly full JSON dumps and incremental dumps.

Because the planned use is structured entities, labels, aliases and identifiers, it fits directly within the CC0 scope. This makes Wikidata suitable for the project's permissive/core entity layer.

Last reviewed: 2026-10-06
Review status: formal first-pass review complete; Phase 1C selected-entity importer implemented and CI-tested; re-check scope before first public data release.
