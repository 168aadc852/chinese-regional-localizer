# LSHK Jyutping Table

Status: **approved_with_conditions**

## Intended use

Cantonese character readings, Jyutping display/search and pronunciation validation.

## Official source

Repository: https://github.com/lshk-org/jyutping-table  
Maintainer: Jyutping Workgroup, Linguistic Society of Hong Kong

## Licence review

Licence: **CC BY 4.0** for the reviewed current pronunciation table.

- Commercial use: yes.
- Modification/adaptation: yes.
- Redistribution: yes.
- Attribution: required.
- Indicate changes: required when sharing adapted material.
- ShareAlike: no.

## Machine-readable ingest scope

Current importer allows exactly:

`lshk:list.tsv`

Historical/third-party material remains excluded unless separately reviewed.

## Pinned production baseline

- commit: `dad2dd6d6f02fc51138ecc6818f7b38eba5c2ad3`
- file: `list.tsv`
- Git blob: `522f41701dd10c4da08d82923ef7ae40d14b9ffb`

The importer now verifies the Git blob when the pinned production revision is claimed, then records SHA-256 of the exact imported bytes.

## Update behavior

`lshk:list.tsv` is one resource lifecycle. Identical repeat imports are idempotent. New revisions mark the old source version non-current; `current_pronunciations` exposes current rows while history remains auditable.

## Packaging

Attribution pack. Importing it into a core SQLite pack is rejected by the pack guard.

Last reviewed: 2026-10-07
