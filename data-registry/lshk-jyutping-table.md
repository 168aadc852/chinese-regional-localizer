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

## Phase 1A pinned importer baseline

First real importer baseline reviewed on 2026-10-06:

- upstream commit: `dad2dd6d6f02fc51138ecc6818f7b38eba5c2ad3`
- file: `list.tsv`
- Git blob SHA: `522f41701dd10c4da08d82923ef7ae40d14b9ffb`
- documented columns: `CH`, `UCODE`, `JP`, `INIT`, `FINL`, `TONE`, `DESC`, `DESC_JP`
- importer: `scripts/import_lshk_jyutping.py`
- schema migration: `schema/migrations/0002_pronunciations.sql`

The importer uses a commit-pinned raw URL when `--download` is explicitly requested, computes SHA-256 from the imported bytes, and records revision/URL/checksum in `source_versions`.

## Update method
Track the official GitHub repository/versions directory. Record commit/date and retain LSHK/Jyutping Workgroup attribution. Do not silently move a production build from one upstream commit to another; review and pin the new version first.

Last reviewed: 2026-10-06
