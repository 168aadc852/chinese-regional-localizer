# Data Policy

## Fundamental rule

Public access is not the same as permission to ingest, transform or redistribute data.

## Review states

`approved`, `approved_with_conditions`, `reference_only`, `pending_review`, and `rejected` describe the human/legal review conclusion. They do not by themselves authorize an importer.

## Production ingestion gate

An implemented importer must require:

1. `ingest_allowed: true`;
2. an exact identifier in `ingest_resources`;
3. the expected licence `pack`;
4. format/resource identity validation;
5. source/version/URL/checksum/retrieval provenance.

If a source is broadly approved but has no machine-readable resource allow-list for the proposed importer, ingestion remains blocked until the manifest is narrowed from reviewed evidence.

`null` rights never mean permission. In particular, unresolved modification/adaptation rights must not be converted into permission for a derived terminology database.

## Licence-pack separation

Data is separated into core, attribution and share-alike packs. The SQLite build metadata records `pack_type`, and importers reject attempts to mix a source from a different pack into that database by default.

Reference-only, pending and rejected material is not packaged into redistributable data builds.

## Provenance and updates

Normalization or storage in SQLite does not remove upstream obligations. Records retain source/version provenance. Refreshes preserve historical versions but mark only the current resource snapshot active for runtime resolution.

## APIs and scraping

Use documented APIs/dumps/download mechanisms only after rights and operational conditions are reviewed. Technical accessibility is never the approval criterion.

## Contributions

Community data must be original under a compatible declared licence or traceable to an approved exact source resource. Unverifiable copied terminology is not accepted into canonical packs.

## Legal note

Repository reviews document project due diligence and engineering policy; they are not a substitute for professional legal advice where needed.
