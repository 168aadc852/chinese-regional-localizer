# PanLex

Status: **approved**

## Intended use
Multilingual lexical mappings, aliases and cross-language terminology matching, including Chinese terms linked to English and other languages.

## Official source
Website: https://panlex.org/
Licence: https://panlex.org/license
Publisher: PanLex / The Long Now Foundation

## Licence review
PanLex explicitly provides CSV and JSON snapshots of:
- the PanLex Database;
- PanLex Lite;
- PanLex Swadesh Lists;
under **CC0 1.0 Universal**.

The official licence page states that the Data may be copied, modified and distributed, including for commercial purposes, without permission.

- Commercial use: **Yes**
- Modification: **Yes**
- Redistribution: **Yes**
- Attribution: **Not legally required under CC0; PanLex requests citation as good practice**
- Share-alike/copyleft: **No**
- Database-specific obligations: No downstream ShareAlike requirement identified for the official CC0 snapshots

## Packaging decision
Approved for the permissive/core candidate layer.

Recommended use:
- multilingual equivalent-expression lookup;
- alias generation;
- candidate discovery for regional Chinese terms;
- disambiguation support using language/variety identifiers.

Preserve provenance even though CC0 does not require attribution:
- PanLex source ID where available;
- language variety ID;
- snapshot date/version;
- upstream expression/meaning identifiers.

## Update method
Use official PanLex CSV/JSON snapshots rather than scraping the website. Record snapshot date/hash where practical.

Last reviewed: 2026-10-06
