# MusicBrainz

Status: **approved_with_conditions**

## Intended use
Music-related entity identifiers, artists, aliases, releases/works and multilingual naming data where the fields belong to MusicBrainz Core Data.

## Region coverage
Global; multilingual aliases

## Official source
Data licence: https://musicbrainz.org/doc/About/Data_License
Publisher: MetaBrainz Foundation / MusicBrainz

## Licence review
The MusicBrainz database is split into two licensing components:

### Core Data — CC0
Officially licensed under **CC0**. MusicBrainz states that anyone may download and use Core Data in any way.

- Commercial use: **Yes**
- Modification: **Yes**
- Redistribution: **Yes**
- Attribution: **Not legally required under CC0, though source credit is recommended for provenance**
- Share-alike/copyleft: **No**

### Supplementary Data — CC BY-NC-SA 3.0
The remaining database portions are licensed under **CC BY-NC-SA 3.0**.

- Commercial use: **No** under that public licence
- Modification: **Yes, non-commercially, subject to attribution and ShareAlike**
- Redistribution: **Subject to CC BY-NC-SA 3.0**

## Packaging decision
Approved for this project **only for MusicBrainz fields explicitly classified as Core Data**.

Do not import Supplementary Data into the standard open-source/commercial-capable database pack.

Before implementing an importer, create a field/table allowlist based on the current MusicBrainz Core Data definition/schema documentation. Unknown or newly added fields default to excluded until reviewed.

Recommended provenance fields:
- MusicBrainz Identifier (MBID)
- entity type
- source dump date
- source table/field
- licence class (`core_cc0` vs excluded supplementary)

## Update method
Use official MusicBrainz database dumps / documented replication mechanisms. Record dump date and ensure importer only selects Core Data tables/columns.

Last reviewed: 2026-10-06
