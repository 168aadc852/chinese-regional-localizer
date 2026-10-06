# Wikidata

Status: **approved**

## Intended use

Structured entity identifiers, locale-specific labels/aliases and selected structured metadata.

## Official source

https://www.wikidata.org/  
Publisher: Wikimedia Foundation / Wikidata community

## Licence review

Approved scope: structured data in CC0-covered namespaces/interfaces. Arbitrary project/help/talk prose is not covered by this importer approval.

Licence: **CC0-1.0** for the reviewed structured-data scope.

- Commercial use: yes.
- Modification: yes.
- Redistribution: yes.
- Legal attribution requirement under CC0: no, while project provenance is still mandatory for quality/auditability.
- Share-alike: no.

## Machine-readable ingest scope

Current selected-entity importer requires:

`wikidata:entity-json:item`

It accepts item EntityData JSON only. Missing regional labels are not synthesized.

## Update behavior

Each QID is a separate resource key (`wikidata:<QID>`). `lastrevid`, modified/retrieval time, URL and canonical entity SHA-256 are recorded. A new revision marks the previous QID source version non-current; runtime entity resolution uses current evidence only.

This prevents historical preferred labels/removed aliases from remaining active after refresh.

## Packaging

Core pack.

## Preferred future update methods

Use official JSON dumps/incremental feeds for scale; documented EntityData/API access remains appropriate for targeted refreshes. Operational limits/usage etiquette are separate from the CC0 licence decision.

Last reviewed: 2026-10-07
