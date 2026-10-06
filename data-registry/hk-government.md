# Hong Kong Government Open Data (DATA.GOV.HK)

Status: **approved_with_conditions**, but **automated ingestion currently disabled**.

## Intended use

Potential use of individually reviewed official datasets distributed through DATA.GOV.HK.

## Official source

https://data.gov.hk/  
Publisher: Digital Policy Office / relevant publishing organisations.

## Terms review

The reviewed portal terms allow browsing, downloading, distributing, reproducing, hyperlinking to and printing qualifying Data for commercial/non-commercial purposes subject to conditions including attribution.

However, the wording reviewed did **not clearly grant general modification/adaptation rights** for building a copyright-dependent derived terminology database.

- Commercial use: yes.
- Reproduction/redistribution: yes under reviewed conditions.
- General modification/adaptation: unresolved from the wording reviewed.
- Attribution: required.
- Share-alike: no general clause identified.
- Dataset-specific terms may override portal-level assumptions.

## Machine policy

Because `modification_allowed` remains unresolved, `sources.yaml` now sets:

`ingest_allowed: false`

No production importer may ingest DATA.GOV.HK-derived terminology until an individual dataset is reviewed and given a narrower exact machine-readable resource approval.

This does not change the human conclusion that qualifying portal data can be reused within rights actually granted; it prevents the automated database pipeline from treating ambiguous adaptation rights as permission.

## Important boundary

This source never automatically covers every Hong Kong Government website or Department of Justice resource. The DoJ glossaries remain a separate pending review.

Last reviewed: 2026-10-07
