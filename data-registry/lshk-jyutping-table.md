# LSHK Jyutping Table

Status: **approved_with_conditions**

## Intended use
Cantonese character readings, Jyutping display/search, pronunciation validation and HKSCS-related character support.

## Official source
Repository: https://github.com/lshk-org/jyutping-table
Maintainer: Jyutping Workgroup, Linguistic Society of Hong Kong

## Licence review
The official repository explicitly states that the Cantonese Pronunciation List of the Characters for Computers is released under **Creative Commons Attribution 4.0 International (CC BY 4.0)**.

The current machine-friendly TSV (`list.tsv`) is maintained by the LSHK Jyutping Workgroup; historical JSON/PDF versions are also present.

- Commercial use: **Yes**
- Modification/adaptation: **Yes**
- Redistribution: **Yes**
- Attribution: **Required**
- Indicate changes: **Required under CC BY 4.0 when sharing adapted material**
- ShareAlike: **No**

## Packaging decision
Approved for an attribution-required Cantonese pronunciation layer.

Prefer the maintained `list.tsv` as the canonical import source.

Historical PDFs/JSON files may have additional provenance considerations; ingest the current maintained TSV first and review historical files only if needed.

## Update method
Track the official GitHub repository/versions directory. Record commit/date and retain LSHK/Jyutping Workgroup attribution.

Last reviewed: 2026-10-06
